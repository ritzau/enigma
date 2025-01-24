use async_trait::async_trait;
use chrono::Duration;
use enigma_auth::{AccessToken, PasswordHash, RefreshToken, UserId, UserName};
use std::error::Error;
use std::net::IpAddr;
use time::OffsetDateTime;

#[cfg(test)]
pub mod mock;

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

    async fn set_hash(&self, user_id: &UserId, hash: &PasswordHash) -> Result<(), Box<dyn Error>>;

    async fn hash(&self, user_id: &UserId) -> Result<PasswordHash, Box<dyn Error>>;

    async fn add_role(&self, user_id: &UserId, role: &str) -> Result<(), Box<dyn Error>>;

    async fn remove_role(&self, user_id: &UserId, role: &str) -> Result<(), Box<dyn Error>>;

    async fn list_accounts(&self) -> Result<Vec<(UserId, UserName)>, Box<dyn Error>>;

    async fn get_user_info(
        &self,
        user_id: &UserId,
    ) -> Result<(UserId, UserName, Vec<String>), Box<dyn Error>>;

    async fn create_session(
        &self,
        user_id: &UserId,
        access_ttl: Duration,
        refresh_ttl: Duration,
        remote_ip: Option<IpAddr>,
    ) -> Result<(AccessToken, RefreshToken), Box<dyn Error>>;

    async fn session(
        &self,
        access_token: &AccessToken,
    ) -> Result<(UserId, OffsetDateTime), Box<dyn Error>>;

    async fn refresh_session(
        &self,
        refresh_token: &RefreshToken,
        access_ttl: Duration,
        refresh_ttl: Duration,
        remote_ip: &Option<IpAddr>,
    ) -> Result<(AccessToken, RefreshToken), Box<dyn Error>>;

    async fn purge_expired_sessions(&self, now: OffsetDateTime) -> Result<u64, Box<dyn Error>>;
}
