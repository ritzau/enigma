use crate::db::AuthDatabase;
use chrono::Utc;
use enigma_auth::{AccessToken, PasswordHash, UserId, UserName};
use sqlx::types::Uuid;
use sqlx::{Pool, Postgres};
use std::env;
use std::error::Error;
use std::time::Duration;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use tonic::async_trait;
use tracing::instrument;

pub struct PostgresAuthDatabase {
    pool: Pool<Postgres>,
}

impl PostgresAuthDatabase {
    #[instrument(err)]
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        dotenvy::dotenv()?;
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let pool = Pool::<Postgres>::connect(&database_url).await?;

        sqlx::query(
            "\
CREATE EXTENSION IF NOT EXISTS pgcrypto;
",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "\
CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    hash TEXT NOT NULL
);
",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "\
CREATE TABLE IF NOT EXISTS sessions (
  session_id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id    BIGINT NOT NULL,
  expires_at timestamptz NOT NULL,
  role       text
);
",
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl AuthDatabase for PostgresAuthDatabase {
    #[instrument(skip(self, hash), err)]
    async fn create_account(
        &self,
        username: &UserName,
        hash: &PasswordHash,
    ) -> Result<UserId, Box<dyn Error>> {
        let id = sqlx::query_scalar!(
            "INSERT INTO users (username, hash) VALUES ($1, $2) RETURNING id",
            username.as_str(),
            hash.as_str(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id.into())
    }

    #[instrument(skip(self), err)]
    async fn user_id(&self, username: &UserName) -> Result<UserId, Box<dyn Error>> {
        let row = sqlx::query!(
            "SELECT id FROM users WHERE username = $1",
            username.as_str()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.id.into())
    }

    #[instrument(skip(self), err)]
    async fn delete_user(&self, user_id: &UserId) -> Result<(), Box<dyn Error>> {
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id.value())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn set_hash(
        &mut self,
        user_id: &UserId,
        hash: &PasswordHash,
    ) -> Result<(), Box<dyn Error>> {
        sqlx::query!(
            "UPDATE users SET hash = $1 WHERE id = $2",
            hash.as_str(),
            user_id.value()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn hash(&self, user_id: &UserId) -> Result<PasswordHash, Box<dyn Error>> {
        let row = sqlx::query!("SELECT hash FROM users WHERE id = $1", user_id.value())
            .fetch_one(&self.pool)
            .await?;

        Ok(row.hash.into())
    }

    #[instrument(skip(self), err)]
    async fn list_accounts(&self) -> Result<Vec<(UserId, UserName)>, Box<dyn Error>> {
        let rows = sqlx::query!("SELECT id, username FROM users")
            .fetch_all(&self.pool)
            .await?;

        let users = rows
            .into_iter()
            .map(|row| (UserId::from(row.id), UserName::from(row.username)))
            .collect();

        Ok(users)
    }

    #[instrument(skip(self), err)]
    async fn create_session(
        &self,
        user_id: &UserId,
        ttl: Duration,
    ) -> Result<Uuid, Box<dyn Error>> {
        let expires_at = Utc::now() + ttl;
        let expires_at = OffsetDateTime::parse(&expires_at.to_rfc3339(), &Rfc3339)?;
        let session_id = sqlx::query_scalar!(
            "INSERT INTO sessions (user_id, expires_at) VALUES ($1, $2) RETURNING session_id",
            user_id.value(),
            expires_at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(session_id)
    }

    #[instrument(skip(self, access_token), err)]
    async fn session(
        &self,
        access_token: &AccessToken,
    ) -> Result<(UserId, OffsetDateTime), Box<dyn Error>> {
        let row = sqlx::query!(
            "SELECT user_id, expires_at FROM sessions WHERE session_id = $1",
            access_token.value()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((UserId::from(row.user_id), row.expires_at))
    }

    #[instrument(skip(self), err)]
    async fn purge_expired_sessions(&self, now: OffsetDateTime) -> Result<u64, Box<dyn Error>> {
        let result = sqlx::query!("DELETE FROM sessions WHERE expires_at < $1", now)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
