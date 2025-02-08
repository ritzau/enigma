use crate::db::AuthDatabase;
use crate::EnigmaAuthService;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHasher, PasswordVerifier};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use enigma_auth::{AccessToken, PasswordHash, RefreshToken, UserId, UserName};
use std::error::Error;
use std::net::IpAddr;

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

    async fn change_password(
        &self,
        user_id: &UserId,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), Box<dyn Error>> {
        let password_hash = self.db.hash(user_id).await?;

        if verify_password(&password_hash, old_password).unwrap_or(false) {
            let new_hash =
                hash(new_password).map_err(|e| format!("Failed to hash password: {}", e))?;
            self.db.set_hash(user_id, &new_hash).await?;
            Ok(())
        } else {
            Err("Invalid password".into())
        }
    }

    async fn add_role(&self, user_id: &UserId, role: &str) -> Result<(), Box<dyn Error>> {
        self.db.add_role(user_id, role).await
    }

    async fn remove_role(&self, user_id: &UserId, role: &str) -> Result<(), Box<dyn Error>> {
        self.db.remove_role(user_id, role).await
    }

    async fn refresh_session(
        &self,
        refresh_token: &RefreshToken,
        remote_ip: &Option<IpAddr>,
    ) -> Result<(AccessToken, RefreshToken), Box<dyn Error>> {
        let (access_token, refresh_token) = self
            .db
            .refresh_session(
                refresh_token,
                Duration::minutes(1),
                Duration::days(28),
                remote_ip,
            )
            .await?;

        Ok((access_token, refresh_token))
    }

    async fn login(
        &self,
        username: &str,
        password: &str,
        remote_ip: Option<IpAddr>,
    ) -> Result<(UserId, AccessToken, RefreshToken), Box<dyn Error>> {
        let user_id = self.db.user_id(&username.into()).await?;

        let hash = self.db.hash(&user_id).await?;

        if verify_password(&hash, password).unwrap_or(false) {
            let (access_token, refresh_token) = self
                .db
                .create_session(
                    &user_id,
                    Duration::seconds(60),
                    Duration::weeks(4),
                    remote_ip,
                )
                .await?;

            Ok((user_id, access_token, refresh_token))
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

    async fn get_user_info(
        &self,
        user_id: &UserId,
    ) -> Result<(UserId, UserName, Vec<String>), Box<dyn Error>> {
        self.db.get_user_info(user_id).await
    }

    async fn get_session(
        &self,
        access_token: &AccessToken,
    ) -> Result<(bool, Option<UserId>), Box<dyn Error>> {
        if let Ok((user_id, expires_at)) = self.db.session(access_token).await {
            Ok((expires_at > Utc::now(), Some(user_id)))
        } else {
            Ok((false, None))
        }
    }

    async fn purge_expired_sessions(&self) -> Result<u64, Box<dyn Error>> {
        Ok(self.db.purge_expired_sessions(Utc::now()).await?)
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
    let parsed_hash = argon2::PasswordHash::new(hash.as_str())?;
    Ok(argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
