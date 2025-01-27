use crate::db::{DatabaseError, EnigmaProfilesDatabase};
use async_trait::async_trait;
use enigma_auth::UserId;
use enigma_profiles::EnigmaUserProfile;
use sqlx::{Error, PgPool, Pool, Postgres, Transaction};
use std::env;
use tracing::instrument;
use uuid::Uuid;

pub struct PostgresProfilesDatabase {
    pool: Pool<Postgres>,
}

impl PostgresProfilesDatabase {
    #[instrument(err)]
    pub async fn new() -> Result<Self, DatabaseError> {
        dotenv::dotenv().expect("Failed to read .env file");

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = Pool::<Postgres>::connect(&database_url)
            .await
            .map_err(|e| {
                DatabaseError::IllegalState("Cannot connect to the profiles DB", Some(e.into()))
            })?;

        ensure_enum_exists(&pool)
            .await
            .map_err(|e| DatabaseError::IllegalState("Can't create enum", Some(e.into())))?;

        let sql = [
            "CREATE TABLE IF NOT EXISTS user_profiles (
                user_id BIGINT PRIMARY KEY,
                legal_name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                profile_picture_url TEXT NOT NULL,
                primary_email TEXT NOT NULL,
                date_of_birth DATE NOT NULL,
                FOREIGN KEY (user_id) REFERENCES accounts(user_id)
            )",
            "CREATE INDEX IF NOT EXISTS idx_birthdays ON user_profiles (EXTRACT(MONTH FROM date_of_birth), EXTRACT(DAY FROM date_of_birth))",
            "CREATE TABLE IF NOT EXISTS connections (
                connection_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                user_id BIGINT NOT NULL,
                peer_id BIGINT NOT NULL,
                connection_kind TEXT NOT NULL CHECK (connection_kind <> ''),
                status CONNECTION_STATUS NOT NULL DEFAULT 'requested',
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                status_updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (user_id) REFERENCES accounts(user_id) ON DELETE CASCADE,
                FOREIGN KEY (peer_id) REFERENCES accounts(user_id) ON DELETE CASCADE
            )",
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_connections_pair
            ON connections (user_id, peer_id)",
            "CREATE INDEX IF NOT EXISTS idx_user_id ON connections (user_id)",
        ];

        for s in sql {
            sqlx::query(s).execute(&pool).await.map_err(|e| {
                DatabaseError::IllegalState("Cannot setup the profiles DB", Some(e.into()))
            })?;
        }

        Ok(Self { pool })
    }

    #[instrument(err, skip(self))]
    async fn accept_connection_with_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>, // Reuse existing transaction
        user_id: &UserId,
        peer_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError> {
        // Lock the rows to prevent concurrent modifications
        sqlx::query!(
            "SELECT connection_id FROM connections
         WHERE (user_id = $1 AND peer_id = $2)
            OR (user_id = $2 AND peer_id = $1)
         FOR UPDATE",
            user_id.value(),
            peer_id.value()
        )
        .fetch_all(tx.as_mut())
        .await
        .map_err(|e| DatabaseError::IllegalState("Can't lock connection rows", Some(e.into())))?;

        // Update the requester's status to 'connected' (keep their original kind)
        sqlx::query!(
            "UPDATE connections SET status = 'connected', status_updated_at = NOW()
        WHERE user_id = $2 AND peer_id = $1",
            user_id.value(),
            peer_id.value()
        )
        .execute(tx.as_mut())
        .await
        .map_err(|e| {
            DatabaseError::IllegalState("Can't update connection status", Some(e.into()))
        })?;

        // Update the acceptor's status to 'connected' and set their chosen connection kind
        sqlx::query!(
        "UPDATE connections SET status = 'connected', connection_kind = $3, status_updated_at = NOW()
        WHERE user_id = $1 AND peer_id = $2",
        user_id.value(),
        peer_id.value(),
        kind
    )
            .execute(tx.as_mut())
            .await
            .map_err(|e| DatabaseError::IllegalState("Can't update connection status", Some(e.into())))?;

        Ok(())
    }
}

pub async fn ensure_enum_exists(pool: &PgPool) -> Result<(), Error> {
    // Check if the enum already exists
    let exists: (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'connection_status');",
    )
    .fetch_one(pool)
    .await?;

    if !exists.0 {
        // Enum doesn't exist, so create it
        sqlx::query(
            "CREATE TYPE CONNECTION_STATUS AS ENUM (
                'request_sent',
                'requested',
                'connected',
                'denied',
                'follower',
                'followed',
                'ghost'
            )",
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

#[async_trait]
impl EnigmaProfilesDatabase for PostgresProfilesDatabase {
    #[instrument(err, skip_all, fields(?profile))]
    async fn create_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError> {
        sqlx::query!(
            "INSERT INTO user_profiles (user_id, legal_name, display_name, profile_picture_url, primary_email, date_of_birth) VALUES ($1, $2, $3, $4, $5, $6)",
            profile.user_id.value(),
            profile.legal_name,
            profile.display_name,
            profile.profile_picture_url,
            profile.primary_email,
            profile.date_of_birth
        ).execute(&self.pool).await.map_err(|e| {
            DatabaseError::IllegalState("Cannot create user profile", Some(e.into()))
        })?;

        Ok(())
    }

    #[instrument(err, skip(self))]
    async fn delete_profile(&self, user_id: &UserId) -> Result<(), DatabaseError> {
        sqlx::query!(
            "DELETE FROM user_profiles WHERE user_id = $1",
            user_id.value(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::IllegalState("Cannot delete user profile", Some(e.into())))?;

        Ok(())
    }

    #[instrument(err, skip_all, fields(%user_id))]
    async fn get_profile(&self, user_id: &UserId) -> Result<EnigmaUserProfile, DatabaseError> {
        let profile = sqlx::query_as!(
            EnigmaUserProfile,
            "SELECT * FROM user_profiles WHERE user_id = $1",
            user_id.value()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| DatabaseError::InvalidUser("Cannot fetch user profile", Some(e.into())))?
        .ok_or(DatabaseError::InvalidUser("User not found", None))?;

        Ok(profile)
    }

    #[instrument(err, skip(self))]
    async fn update_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError> {
        sqlx::query!(
            "UPDATE user_profiles
            SET
                legal_name = $2,
                display_name = $3,
                profile_picture_url = $4,
                primary_email = $5,
                date_of_birth = $6
            WHERE user_id = $1",
            profile.user_id.value(),
            profile.legal_name,
            profile.display_name,
            profile.profile_picture_url,
            profile.primary_email,
            profile.date_of_birth
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::IllegalState("Cannot update user profile", Some(e.into())))?;

        Ok(())
    }

    #[instrument(err, skip(self))]
    async fn request_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError> {
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                DatabaseError::IllegalState("Can't create transaction", Some(e.into()))
            })?;

        // Check if the reverse request already exists
        let existing_request = sqlx::query!(
        "SELECT status::TEXT AS status FROM connections WHERE user_id = $1 AND peer_id = $2 FOR UPDATE",
        peer_id.value(),
        user_id.value()
    )
            .fetch_optional(tx.as_mut())
            .await
            .map_err(|e| DatabaseError::IllegalState("Can't check for existing request", Some(e.into())))?;

        if existing_request.as_ref().and_then(|r| r.status.clone())
            == Some("request_sent".to_string())
        {
            // If mutual request detected, accept inside the same transaction
            self.accept_connection_with_tx(&mut tx, user_id, peer_id, kind)
                .await?;
            tx.commit().await.map_err(|e| {
                DatabaseError::IllegalState("Can't commit transaction", Some(e.into()))
            })?;
            return Ok(());
        }

        // Otherwise, insert a new connection request
        sqlx::query!(
            "INSERT INTO connections (connection_id, user_id, peer_id, connection_kind, status)
        VALUES ($1, $2, $3, $4, 'request_sent')",
            Uuid::new_v4(),
            user_id.value(),
            peer_id.value(),
            kind
        )
        .execute(tx.as_mut())
        .await
        .map_err(|e| DatabaseError::IllegalState("Can't insert connection", Some(e.into())))?;

        sqlx::query!(
            "INSERT INTO connections (connection_id, user_id, peer_id, connection_kind, status)
        VALUES ($1, $2, $3, $4, 'requested')",
            Uuid::new_v4(),
            peer_id.value(),
            user_id.value(),
            "?"
        )
        .execute(tx.as_mut())
        .await
        .map_err(|e| {
            DatabaseError::IllegalState("Can't insert reverse connection", Some(e.into()))
        })?;

        tx.commit()
            .await
            .map_err(|e| DatabaseError::IllegalState("Can't commit transaction", Some(e.into())))?;
        Ok(())
    }

    #[instrument(err, skip(self))]
    async fn accept_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError> {
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                DatabaseError::IllegalState("Can't create transaction", Some(e.into()))
            })?;

        self.accept_connection_with_tx(&mut tx, user_id, peer_id, kind)
            .await?;

        tx.commit()
            .await
            .map_err(|e| DatabaseError::IllegalState("Can't commit transaction", Some(e.into())))?;
        Ok(())
    }

    #[instrument(err, skip(self))]
    async fn reject_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
    ) -> Result<(), DatabaseError> {
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                DatabaseError::IllegalState("Can't create transaction", Some(e.into()))
            })?;

        // Lock the row to prevent concurrent modifications
        let existing_request = sqlx::query!(
            "SELECT status::TEXT AS status FROM connections
         WHERE user_id = $1 AND peer_id = $2
         FOR UPDATE",
            user_id.value(),
            peer_id.value()
        )
        .fetch_optional(tx.as_mut())
        .await
        .map_err(|e| DatabaseError::IllegalState("Can't lock connection row", Some(e.into())))?;

        if existing_request.is_none() {
            return Err(DatabaseError::NotFound("No pending request found", None));
        }

        let status = existing_request.unwrap().status;
        if status != Some("requested".to_string()) {
            return Err(DatabaseError::IllegalState(
                "Connection request is not pending",
                None,
            ));
        }

        // Update the peer's connection request status to 'denied'
        sqlx::query!(
            "UPDATE connections SET status = 'denied', status_updated_at = NOW()
         WHERE user_id = $1 AND peer_id = $2",
            user_id.value(),
            peer_id.value()
        )
        .execute(tx.as_mut())
        .await
        .map_err(|e| {
            DatabaseError::IllegalState("Can't update connection status", Some(e.into()))
        })?;

        tx.commit()
            .await
            .map_err(|e| DatabaseError::IllegalState("Can't commit transaction", Some(e.into())))?;
        Ok(())
    }

    #[instrument(err, skip(self))]
    async fn add_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "INSERT INTO connections (user_id, peer_id, connection_kind) VALUES ($1, $2, $3)",
            user_id.value(),
            connection_id.value(),
            kind
        )
        .execute(&self.pool)
        .await
        .map_err(|e| DatabaseError::IllegalState("Cannot add connection", Some(e.into())))?;

        Ok(())
    }

    #[instrument(err, skip_all)]
    async fn get_connections(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<(String, EnigmaUserProfile)>, DatabaseError> {
        let records = sqlx::query!(
            "SELECT connection_kind, peer_id, legal_name, display_name, profile_picture_url, primary_email, date_of_birth
            FROM connections
            INNER JOIN user_profiles ON connections.peer_id = user_profiles.user_id
            WHERE connections.user_id = $1",
            user_id.value()
        ).fetch_all(&self.pool).await.map_err(|e| DatabaseError::IllegalState("Cannot get connections", Some(e.into())))?;

        Ok(records
            .into_iter()
            .map(|r| {
                (
                    r.connection_kind,
                    EnigmaUserProfile {
                        user_id: r.peer_id.into(),
                        legal_name: r.legal_name,
                        display_name: r.display_name,
                        profile_picture_url: r.profile_picture_url,
                        primary_email: r.primary_email,
                        date_of_birth: r.date_of_birth,
                    },
                )
            })
            .collect())
    }

    #[instrument(err, skip(self))]
    async fn remove_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "DELETE FROM connections WHERE user_id = $1 AND peer_id = $2",
            user_id.value(),
            connection_id.value()
        )
        .execute(&self.pool)
        .await
        .expect("Cannot remove connection");

        Ok(())
    }

    #[instrument(err, skip(self))]
    async fn update_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError> {
        sqlx::query!(
            "UPDATE connections SET connection_kind = $3 WHERE user_id = $1 AND peer_id = $2",
            user_id.value(),
            connection_id.value(),
            kind
        )
        .execute(&self.pool)
        .await
        .expect("Cannot update connection");

        Ok(())
    }
}
