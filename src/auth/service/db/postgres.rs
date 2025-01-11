use crate::auth::service::db::AuthDatabase;
use crate::auth::{AccessToken, UserHash, UserId, UserName};
use chrono::Utc;
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::{Pool, Postgres};
use std::env;
use std::error::Error;
use std::time::Duration;
use time::format_description::well_known::Rfc3339;
use tonic::async_trait;

pub struct PostgresAuthDatabase {
    pool: Pool<Postgres>,
}

impl PostgresAuthDatabase {
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
    async fn create_account(
        &self,
        username: &UserName,
        hash: &UserHash,
    ) -> Result<UserId, Box<dyn Error>> {
        println!("Inserting user entry: {}/{}", username.0, hash.0);

        let id = sqlx::query_scalar!(
            "INSERT INTO users (username, hash) VALUES ($1, $2) RETURNING id",
            username.0,
            hash.0
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(UserId(id))
    }

    async fn user_id(&self, username: &UserName) -> Result<UserId, Box<dyn Error>> {
        let row = sqlx::query!("SELECT id FROM users WHERE username = $1", username.0)
            .fetch_one(&self.pool)
            .await?;

        Ok(UserId(row.id))
    }

    async fn delete_user(&self, user_id: &UserId) -> Result<(), Box<dyn Error>> {
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id.0)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn set_hash(&mut self, user_id: &UserId, hash: &UserHash) -> Result<(), Box<dyn Error>> {
        sqlx::query!(
            "UPDATE users SET hash = $1 WHERE id = $2",
            hash.0,
            user_id.0
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn hash(&self, user_id: &UserId) -> Result<UserHash, Box<dyn Error>> {
        let row = sqlx::query!("SELECT hash FROM users WHERE id = $1", user_id.0)
            .fetch_one(&self.pool)
            .await?;

        Ok(UserHash(row.hash))
    }

    async fn list_accounts(&self) -> Result<Vec<(UserId, UserName)>, Box<dyn Error>> {
        let rows = sqlx::query!("SELECT id, username FROM users")
            .fetch_all(&self.pool)
            .await?;

        let users = rows
            .into_iter()
            .map(|row| (UserId(row.id), UserName(row.username)))
            .collect();

        Ok(users)
    }

    async fn create_session(
        &self,
        user_id: &UserId,
        ttl: Duration,
    ) -> Result<Uuid, Box<dyn Error>> {
        let expires_at = Utc::now() + ttl;
        let expires_at = OffsetDateTime::parse(&expires_at.to_rfc3339(), &Rfc3339)?;
        let session_id = sqlx::query_scalar!(
            "INSERT INTO sessions (user_id, expires_at) VALUES ($1, $2) RETURNING session_id",
            user_id.0,
            expires_at,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(session_id)
    }

    async fn session(
        &self,
        access_token: &AccessToken,
    ) -> Result<(UserId, OffsetDateTime), Box<dyn Error>> {
        let row = sqlx::query!(
            "SELECT user_id, expires_at FROM sessions WHERE session_id = $1",
            access_token.0
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((UserId(row.user_id), row.expires_at))
    }

    async fn purge_expired_sessions(&self, now: OffsetDateTime) -> Result<u64, Box<dyn Error>> {
        let result = sqlx::query!("DELETE FROM sessions WHERE expires_at < $1", now)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
