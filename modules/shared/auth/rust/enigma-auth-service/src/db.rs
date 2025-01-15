use enigma_auth::{AccessToken, PasswordHash, UserId, UserName};
use sqlx::types::Uuid;
use std::error::Error;
use std::time::Duration;
use time::OffsetDateTime;
use tonic::async_trait;

pub mod postgres;

#[async_trait]
pub trait AuthDatabase: Send + Sync {
    async fn create_account(
        &self,
        username: &UserName,
        hash: &PasswordHash,
    ) -> Result<UserId, Box<dyn Error>>;
    async fn user_id(&self, username: &UserName) -> Result<UserId, Box<dyn Error>>;
    async fn delete_user(&self, user_id: &UserId) -> Result<(), Box<dyn Error>>;
    async fn set_hash(
        &mut self,
        user_id: &UserId,
        hash: &PasswordHash,
    ) -> Result<(), Box<dyn Error>>;
    async fn hash(&self, user_id: &UserId) -> Result<PasswordHash, Box<dyn Error>>;
    async fn list_accounts(&self) -> Result<Vec<(UserId, UserName)>, Box<dyn Error>>;
    async fn create_session(&self, user_id: &UserId, ttl: Duration)
        -> Result<Uuid, Box<dyn Error>>;
    async fn session(
        &self,
        access_token: &AccessToken,
    ) -> Result<(UserId, OffsetDateTime), Box<dyn Error>>;
    async fn purge_expired_sessions(&self, now: OffsetDateTime) -> Result<u64, Box<dyn Error>>;
}
