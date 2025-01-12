use db::AuthDatabase;
use foorum_auth::{AccessToken, PasswordHash, UserId};
use sqlx::types::Uuid;
use std::error::Error;
use tonic::async_trait;

pub mod db;

#[async_trait]
pub trait FoorumAuthService: Send + Sync {
    async fn create_account(&self, username: &str, password: &str) -> Result<i64, Box<dyn Error>>;

    async fn delete_account(&self, user_id: &UserId) -> Result<(), Box<dyn Error>>;

    async fn login(&self, username: &str, password: &str) -> Result<Uuid, Box<dyn Error>>;

    async fn list_accounts(&self) -> Result<Vec<(i64, String)>, Box<dyn Error>>;

    async fn get_session(
        &self,
        access_token: &AccessToken,
    ) -> Result<(bool, Option<UserId>), Box<dyn Error>>;

    async fn purge_expired_sessions(&self) -> Result<u64, Box<dyn Error>>;
}

pub struct DefaultAuthService<T: AuthDatabase> {
    db: T,
}

impl<T: AuthDatabase> DefaultAuthService<T> {
    pub fn new(db: T) -> Self {
        DefaultAuthService { db }
    }
}

#[async_trait]
impl<T: AuthDatabase> FoorumAuthService for DefaultAuthService<T> {
    async fn create_account(&self, username: &str, password: &str) -> Result<i64, Box<dyn Error>> {
        let id = self
            .db
            .create_account(&username.into(), &password.into())
            .await?;

        Ok(id.value())
    }

    async fn delete_account(&self, user_id: &UserId) -> Result<(), Box<dyn Error>> {
        self.db.delete_user(user_id).await
    }

    async fn login(&self, username: &str, password: &str) -> Result<Uuid, Box<dyn Error>> {
        let user_id = self.db.user_id(&username.into()).await?;

        let hash = self.db.hash(&user_id).await?;

        if hash == PasswordHash::from(password) {
            Ok(self
                .db
                .create_session(&user_id, std::time::Duration::from_secs(60))
                .await?)
        } else {
            Err("Invalid password".into())
        }
    }

    async fn list_accounts(&self) -> Result<Vec<(i64, String)>, Box<dyn Error>> {
        Ok(self
            .db
            .list_accounts()
            .await?
            .into_iter()
            .map(|(id, name)| (id.value(), name.to_string()))
            .collect())
    }

    async fn get_session(
        &self,
        access_token: &AccessToken,
    ) -> Result<(bool, Option<UserId>), Box<dyn Error>> {
        if let Ok((user_id, expires_at)) = self.db.session(access_token).await {
            Ok((expires_at > time::OffsetDateTime::now_utc(), Some(user_id)))
        } else {
            Ok((false, None))
        }
    }

    async fn purge_expired_sessions(&self) -> Result<u64, Box<dyn Error>> {
        Ok(self
            .db
            .purge_expired_sessions(time::OffsetDateTime::now_utc())
            .await?)
    }
}
