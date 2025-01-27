use crate::db::{DatabaseError, EnigmaProfilesDatabase};
use async_trait::async_trait;
use enigma_auth::UserId;
use enigma_profiles::EnigmaUserProfile;
use sqlx::{Pool, Postgres};
use std::env;
use tracing::instrument;

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
        ];

        for s in sql {
            sqlx::query(s).execute(&pool).await.map_err(|e| {
                DatabaseError::IllegalState("Cannot setup the profiles DB", Some(e.into()))
            })?;
        }

        Ok(Self { pool })
    }
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
}
