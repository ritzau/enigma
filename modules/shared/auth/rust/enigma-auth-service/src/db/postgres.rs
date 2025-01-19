use crate::db::AuthDatabase;
use chrono::Duration;
use chrono::Utc;
use enigma_auth::{AccessToken, PasswordHash, RefreshToken, UserId, UserName};
use sqlx::{Pool, Postgres};
use std::env;
use std::error::Error;
use std::net::IpAddr;
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

        let sql = [
            "CREATE EXTENSION IF NOT EXISTS pgcrypto;",
            "CREATE TABLE IF NOT EXISTS accounts (
                user_id BIGSERIAL PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                hash TEXT NOT NULL
            );
            ",
            "CREATE INDEX IF NOT EXISTS idx_username ON accounts(username);",
            "CREATE TABLE IF NOT EXISTS sessions (
                session_id BIGSERIAL PRIMARY KEY,
                user_id BIGINT NOT NULL,
                access_token UUID NOT NULL DEFAULT gen_random_uuid(),
                refresh_token UUID NOT NULL DEFAULT gen_random_uuid(),
                access_token_expiry TIMESTAMPTZ NOT NULL,
                refresh_token_expiry TIMESTAMPTZ NOT NULL,
                ip_address VARCHAR(39),
                created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
                CONSTRAINT fk_user_id FOREIGN KEY (user_id) REFERENCES accounts(user_id)
            );
            ",
            "CREATE INDEX IF NOT EXISTS idx_user_id ON sessions(user_id);",
            "CREATE INDEX IF NOT EXISTS idx_access_token ON sessions(access_token);",
            "CREATE INDEX IF NOT EXISTS idx_refresh_token ON sessions(refresh_token);",
        ];

        for s in sql {
            sqlx::query(s).execute(&pool).await?;
        }

        Ok(Self { pool })
    }
}

#[async_trait]
impl AuthDatabase for PostgresAuthDatabase {
    #[instrument(skip_all, err, fields(%username))]
    async fn create_account(
        &self,
        username: &UserName,
        hash: &PasswordHash,
    ) -> Result<UserId, Box<dyn Error>> {
        let id = sqlx::query_scalar!(
            "INSERT INTO accounts (username, hash) VALUES ($1, $2) RETURNING user_id",
            username.as_str(),
            hash.as_str(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id.into())
    }

    #[instrument(skip_all, err, fields(%username))]
    async fn user_id(&self, username: &UserName) -> Result<UserId, Box<dyn Error>> {
        let row = sqlx::query!(
            "SELECT user_id FROM accounts WHERE username = $1",
            username.as_str()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.user_id.into())
    }

    #[instrument(skip_all, err, fields(%user_id))]
    async fn delete_user(&self, user_id: &UserId) -> Result<(), Box<dyn Error>> {
        sqlx::query!("DELETE FROM accounts WHERE user_id = $1", user_id.value())
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    #[instrument(skip_all, err, fields(%user_id))]
    async fn set_hash(&self, user_id: &UserId, hash: &PasswordHash) -> Result<(), Box<dyn Error>> {
        sqlx::query!(
            "UPDATE accounts SET hash = $1 WHERE user_id = $2",
            hash.as_str(),
            user_id.value()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip_all, err, fields(%user_id))]
    async fn hash(&self, user_id: &UserId) -> Result<PasswordHash, Box<dyn Error>> {
        let row = sqlx::query!(
            "SELECT hash FROM accounts WHERE user_id = $1",
            user_id.value()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.hash.into())
    }

    #[instrument(skip_all, err)]
    async fn list_accounts(&self) -> Result<Vec<(UserId, UserName)>, Box<dyn Error>> {
        let rows = sqlx::query!("SELECT user_id, username FROM accounts")
            .fetch_all(&self.pool)
            .await?;

        let users = rows
            .into_iter()
            .map(|row| (UserId::from(row.user_id), UserName::from(row.username)))
            .collect();

        Ok(users)
    }

    #[instrument(skip_all, err, fields(%user_id, ttl))]
    async fn create_session(
        &self,
        user_id: &UserId,
        access_ttl: Duration,
        refresh_ttl: Duration,
        remote_ip: Option<IpAddr>,
    ) -> Result<(AccessToken, RefreshToken), Box<dyn Error>> {
        let now = Utc::now();
        let access_token_expiry =
            OffsetDateTime::parse(&(now + access_ttl).to_rfc3339(), &Rfc3339)?;
        let refresh_token_expiry =
            OffsetDateTime::parse(&(now + refresh_ttl).to_rfc3339(), &Rfc3339)?;

        let record = sqlx::query!(
            "\
            INSERT INTO sessions (
                user_id,
                access_token_expiry,
                refresh_token_expiry,
                ip_address
            ) VALUES ($1, $2, $3, $4)
            RETURNING access_token, refresh_token;
            ",
            user_id.value(),
            access_token_expiry,
            refresh_token_expiry,
            remote_ip.map_or(String::default(), |ip| ip.to_string())
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((
            AccessToken::from(record.access_token),
            RefreshToken::from(record.refresh_token),
        ))
    }

    #[instrument(skip_all, err, fields(%access_token))]
    async fn session(
        &self,
        access_token: &AccessToken,
    ) -> Result<(UserId, OffsetDateTime), Box<dyn Error>> {
        let row = sqlx::query!(
            "SELECT user_id, access_token_expiry FROM sessions WHERE access_token = $1 AND access_token_expiry > NOW()",
            access_token.value()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((UserId::from(row.user_id), row.access_token_expiry))
    }

    #[instrument(skip_all, err)]
    async fn refresh_session(
        &self,
        refresh_token: &RefreshToken,
        access_ttl: Duration,
        refresh_ttl: Duration,
        remote_ip: &Option<IpAddr>,
    ) -> Result<(AccessToken, RefreshToken), Box<dyn Error>> {
        let now = Utc::now();
        let refresh_token_expiry = now + refresh_ttl;
        let refresh_token_expiry =
            OffsetDateTime::parse(&refresh_token_expiry.to_rfc3339(), &Rfc3339)?;
        let access_token_expiry = now + access_ttl;
        let access_token_expiry =
            OffsetDateTime::parse(&access_token_expiry.to_rfc3339(), &Rfc3339)?;

        let record = sqlx::query!(
            "UPDATE sessions
            SET access_token = gen_random_uuid(),
                access_token_expiry = $2,
                refresh_token = gen_random_uuid(),
                refresh_token_expiry = $3,
                ip_address = $4,
                updated_at = NOW()
            WHERE refresh_token = $1
            RETURNING user_id, access_token, refresh_token",
            refresh_token.value(),
            access_token_expiry,
            refresh_token_expiry,
            remote_ip.map_or(String::default(), |ip| ip.to_string())
        )
        .fetch_one(&self.pool)
        .await?;

        Ok((
            AccessToken::from(record.access_token),
            RefreshToken::from(record.refresh_token),
        ))
    }

    #[instrument(skip_all, err, fields(%now))]
    async fn purge_expired_sessions(&self, now: OffsetDateTime) -> Result<u64, Box<dyn Error>> {
        let result = sqlx::query!("DELETE FROM sessions WHERE refresh_token_expiry < $1", now)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }
}
