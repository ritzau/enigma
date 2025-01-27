use crate::db::AuthDatabase;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use enigma_auth::{AccessToken, PasswordHash, RefreshToken, UserId, UserName};
use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::net::IpAddr;
use tokio::sync::RwLock;
use uuid::Uuid;

struct Maps {
    accounts: HashMap<UserName, (UserId, PasswordHash, BTreeSet<String>)>,
    sessions: HashMap<AccessToken, (UserId, DateTime<Utc>)>,
    refresh_tokens: HashMap<RefreshToken, (AccessToken, RefreshToken)>,
}

pub struct MockAuthDatabase {
    maps: RwLock<Maps>,
}

impl MockAuthDatabase {
    pub fn new() -> Self {
        MockAuthDatabase {
            maps: RwLock::new(Maps {
                accounts: HashMap::new(),
                sessions: HashMap::new(),
                refresh_tokens: HashMap::new(),
            }),
        }
    }
}

impl Default for MockAuthDatabase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[async_trait]
impl AuthDatabase for MockAuthDatabase {
    async fn create_account(
        &self,
        username: &UserName,
        hash: &PasswordHash,
    ) -> Result<UserId, Box<dyn Error>> {
        let mut maps = self.maps.write().await;

        let user_id = UserId::from(maps.accounts.len() as i64 + 1);
        maps.accounts.insert(
            username.clone(),
            (user_id.clone(), hash.clone(), BTreeSet::new()),
        );
        Ok(user_id)
    }

    async fn user_id(&self, username: &UserName) -> Result<UserId, Box<dyn Error>> {
        let maps = self.maps.read().await;

        maps.accounts
            .get(username)
            .map(|(id, ..)| id.clone())
            .ok_or_else(|| "User not found".into())
    }

    async fn delete_user(&self, user_id: &UserId) -> Result<(), Box<dyn Error>> {
        let mut maps = self.maps.write().await;
        maps.accounts.retain(|_, (id, ..)| id != user_id);
        Ok(())
    }

    async fn set_hash(&self, user_id: &UserId, hash: &PasswordHash) -> Result<(), Box<dyn Error>> {
        let mut maps = self.maps.write().await;

        for (_, (id, stored_hash, ..)) in maps.accounts.iter_mut() {
            if id == user_id {
                *stored_hash = hash.clone();
                return Ok(());
            }
        }
        Err("User not found".into())
    }

    async fn hash(&self, user_id: &UserId) -> Result<PasswordHash, Box<dyn Error>> {
        let maps = self.maps.read().await;

        maps.accounts
            .values()
            .find(|(id, ..)| id == user_id)
            .map(|(_, hash, ..)| hash.clone())
            .ok_or_else(|| "User not found".into())
    }

    async fn add_role(&self, user_id: &UserId, role: &str) -> Result<(), Box<dyn Error>> {
        let mut maps = self.maps.write().await;

        for (_, (id, _, roles)) in maps.accounts.iter_mut() {
            if id == user_id {
                roles.insert(role.to_string());
                return Ok(());
            }
        }
        Err("User not found".into())
    }

    async fn remove_role(&self, user_id: &UserId, role: &str) -> Result<(), Box<dyn Error>> {
        let mut maps = self.maps.write().await;

        for (_, (id, _, roles)) in maps.accounts.iter_mut() {
            if id == user_id {
                roles.remove(role);
                return Ok(());
            }
        }
        Err("User not found".into())
    }

    async fn list_accounts(&self) -> Result<Vec<(UserId, UserName)>, Box<dyn Error>> {
        let maps = self.maps.read().await;

        Ok(maps
            .accounts
            .iter()
            .map(|(username, (id, ..))| (id.clone(), username.clone()))
            .collect())
    }

    async fn get_user_info(
        &self,
        user_id: &UserId,
    ) -> Result<(UserId, UserName, Vec<String>), Box<dyn Error>> {
        let maps = self.maps.write().await;

        for (username, (id, _, roles)) in maps.accounts.iter() {
            if id == user_id {
                return Ok((
                    user_id.clone(),
                    username.clone(),
                    roles.iter().cloned().collect(),
                ));
            }
        }

        Err("User not found".into())
    }

    async fn create_session(
        &self,
        user_id: &UserId,
        access_ttl: Duration,
        _refresh_ttl: Duration,
        _remote_ip: Option<IpAddr>,
    ) -> Result<(AccessToken, RefreshToken), Box<dyn Error>> {
        let mut maps = self.maps.write().await;

        let access_token = AccessToken::from(Uuid::new_v4());
        let refresh_token = RefreshToken::from(Uuid::new_v4());
        let expires_at = Utc::now() + access_ttl;
        maps.sessions
            .insert(access_token.clone(), (user_id.clone(), expires_at));
        maps.refresh_tokens.insert(
            refresh_token.clone(),
            (access_token.clone(), refresh_token.clone()),
        );
        Ok((access_token, refresh_token))
    }

    async fn session(
        &self,
        access_token: &AccessToken,
    ) -> Result<(UserId, DateTime<Utc>), Box<dyn Error>> {
        let maps = self.maps.read().await;

        let (user_id, date_time) = maps
            .sessions
            .get(access_token)
            .cloned()
            .ok_or_else(|| Box::<dyn Error>::from("Session not found"))?;

        Ok((user_id, date_time))
    }

    async fn refresh_session(
        &self,
        refresh_token: &RefreshToken,
        access_ttl: Duration,
        _refresh_ttl: Duration,
        _remote_ip: &Option<IpAddr>,
    ) -> Result<(AccessToken, RefreshToken), Box<dyn Error>> {
        let mut maps = self.maps.write().await;

        let (old_access_token, _) = match maps.refresh_tokens.get(refresh_token) {
            Some(token) => token.clone(),
            None => return Err("Invalid refresh token".into()),
        };

        maps.sessions.remove(&old_access_token);
        let new_access_token = AccessToken::from(Uuid::new_v4());
        let new_refresh_token = RefreshToken::from(Uuid::new_v4());
        let expires_at = Utc::now() + access_ttl;
        let old_user_id = maps.sessions[&old_access_token].0.clone();
        maps.sessions
            .insert(new_access_token.clone(), (old_user_id, expires_at));
        maps.refresh_tokens.insert(
            new_refresh_token.clone(),
            (new_access_token.clone(), new_refresh_token.clone()),
        );
        Ok((new_access_token, new_refresh_token))
    }

    async fn purge_expired_sessions(&self, now: DateTime<Utc>) -> Result<u64, Box<dyn Error>> {
        let mut maps = self.maps.write().await;
        let initial_len = maps.sessions.len();
        maps.sessions.retain(|_, (_, expires_at)| *expires_at > now);
        Ok((initial_len - maps.sessions.len()) as u64)
    }
}
