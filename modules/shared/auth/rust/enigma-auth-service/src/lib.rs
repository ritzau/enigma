use async_trait::async_trait;
use enigma_auth::{AccessToken, RefreshToken, UserId, UserName};
use std::error::Error;
use std::net::IpAddr;

pub mod db;
pub mod default;

#[cfg(feature = "grpc")]
pub mod grpc;

#[async_trait]
pub trait EnigmaAuthService: Send + Sync {
    async fn create_account(&self, username: &str, password: &str) -> Result<i64, Box<dyn Error>>;

    async fn delete_account(&self, user_id: &UserId) -> Result<(), Box<dyn Error>>;

    async fn change_password(
        &self,
        user_id: &UserId,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), Box<dyn Error>>;

    async fn add_role(&self, user_id: &UserId, role: &str) -> Result<(), Box<dyn Error>>;

    async fn remove_role(&self, user_id: &UserId, role: &str) -> Result<(), Box<dyn Error>>;

    async fn refresh_session(
        &self,
        refresh_token: &RefreshToken,
        x: &Option<IpAddr>,
    ) -> Result<(AccessToken, RefreshToken), Box<dyn Error>>;

    async fn login(
        &self,
        username: &str,
        password: &str,
        remote_ip: Option<IpAddr>,
    ) -> Result<(UserId, AccessToken, RefreshToken), Box<dyn Error>>;

    async fn list_accounts(&self) -> Result<Vec<(i64, String)>, Box<dyn Error>>;

    async fn get_user_info(
        &self,
        user_id: &UserId,
    ) -> Result<(UserId, UserName, Vec<String>), Box<dyn Error>>;

    async fn get_session(
        &self,
        access_token: &AccessToken,
    ) -> Result<(bool, Option<UserId>), Box<dyn Error>>;

    async fn purge_expired_sessions(&self) -> Result<u64, Box<dyn Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::default::DefaultAuthService;
    use db::mock::MockAuthDatabase;

    #[tokio::test]
    async fn test_create_account() {
        let db = MockAuthDatabase::new();
        let service = DefaultAuthService::new(db);

        let result = service.create_account("testuser", "password").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_account() {
        let db = MockAuthDatabase::new();
        let service = DefaultAuthService::new(db);

        let user_id = UserId::from(1);
        let result = service.delete_account(&user_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bad_password_change() {
        let db = MockAuthDatabase::new();
        let service = DefaultAuthService::new(db);

        let user_id = UserId::from(1);
        let result = service
            .change_password(&user_id, "old_password", "new_password")
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_bad_session_refresh() {
        let db = MockAuthDatabase::new();
        let service = DefaultAuthService::new(db);

        let refresh_token = RefreshToken::new_random();
        let result = service.refresh_session(&refresh_token, &None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login() {
        let db = MockAuthDatabase::new();
        let service = DefaultAuthService::new(db);
        service
            .create_account("testuser", "password")
            .await
            .unwrap();

        let result = service.login("testuser", "password", None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bad_login() {
        let db = MockAuthDatabase::new();
        let service = DefaultAuthService::new(db);

        let result = service.login("testuser", "password", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_accounts() {
        let db = MockAuthDatabase::new();
        let service = DefaultAuthService::new(db);

        let result = service.list_accounts().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_session() {
        let db = MockAuthDatabase::new();
        let service = DefaultAuthService::new(db);

        let access_token = AccessToken::new_random();
        let result = service.get_session(&access_token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_purge_expired_sessions() {
        let db = MockAuthDatabase::new();
        let service = DefaultAuthService::new(db);

        let result = service.purge_expired_sessions().await;
        assert!(result.is_ok());
    }
}
