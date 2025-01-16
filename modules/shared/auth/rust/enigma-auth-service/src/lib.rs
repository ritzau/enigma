use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHasher, PasswordVerifier};
use db::AuthDatabase;
use enigma_auth::{AccessToken, PasswordHash, UserId};
use sqlx::types::Uuid;
use std::error::Error;
use tonic::async_trait;

pub mod db;

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
impl<T: AuthDatabase> EnigmaAuthService for DefaultAuthService<T> {
    async fn create_account(&self, username: &str, password: &str) -> Result<i64, Box<dyn Error>> {
        let hash = hash(password).map_err(|e| format!("Failed to hash password: {}", e))?;

        let id = self.db.create_account(&username.into(), &hash).await?;

        Ok(id.value())
    }

    async fn delete_account(&self, user_id: &UserId) -> Result<(), Box<dyn Error>> {
        self.db.delete_user(user_id).await
    }

    async fn change_password(&self, user_id: &UserId, old_password: &str, new_password: &str) -> Result<(), Box<dyn Error>> {
        let password_hash = self.db.hash(user_id).await?;

        if verify_password(&password_hash, old_password).unwrap_or(false) {
            let new_hash = hash(new_password).map_err(|e| format!("Failed to hash password: {}", e))?;
            self.db.set_hash(&user_id, &new_hash).await?;
            Ok(())
        } else {
            Err("Invalid password".into())
        }
    }

    async fn login(&self, username: &str, password: &str) -> Result<Uuid, Box<dyn Error>> {
        let user_id = self.db.user_id(&username.into()).await?;

        let hash = self.db.hash(&user_id).await?;

        if verify_password(&hash, password).unwrap_or(false) {
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

fn argon_config() -> Argon2<'static> {
    Argon2::default()
}

fn hash(password: &str) -> Result<PasswordHash, password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = argon_config();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    let hash = PasswordHash::from(hash.as_str());
    assert!(verify_password(&hash, password)?);

    Ok(hash)
}

fn verify_password(hash: &PasswordHash, password: &str) -> Result<bool, password_hash::Error> {
    let argon2 = argon_config();
    let parsed_hash = argon2::PasswordHash::new(&hash.as_str())?;
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
