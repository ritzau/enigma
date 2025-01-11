use async_trait::async_trait;
use sqlx::types::Uuid;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UserId(pub i64);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UserName(pub String);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UserHash(pub String);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AccessToken(pub Uuid);

#[async_trait]
pub trait FoorumAuthClient {
    async fn create_account(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<UserId, Box<dyn std::error::Error>>;
    async fn delete_account(&mut self, user_id: &UserId) -> Result<(), Box<dyn std::error::Error>>;
    async fn get_session(
        &mut self,
        access_token: &Uuid,
    ) -> Result<UserId, Box<dyn std::error::Error>>;
    async fn list_accounts(&mut self) -> Result<Vec<(UserId, String)>, Box<dyn std::error::Error>>;
    async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<String, Box<dyn std::error::Error>>;
    async fn purge_expired_sessions(&mut self) -> Result<u64, Box<dyn std::error::Error>>;
}
