use enigma_auth::{AccessToken, RefreshToken, UserId, UserName};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct Session {
    user_id: UserId,
    user_name: UserName,
    access_token: AccessToken,
    refresh_token: RefreshToken,
}

impl Session {
    pub fn new(
        user_id: UserId,
        user_name: UserName,
        access_token: AccessToken,
        refresh_token: RefreshToken,
    ) -> Self {
        Self {
            user_id,
            user_name,
            access_token,
            refresh_token,
        }
    }

    pub fn store(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        std::fs::write(path, toml::to_string(self)?)?;
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str::<Self>(&contents)?)
    }

    pub fn user_id(&self) -> &UserId {
        &self.user_id
    }

    pub fn user_name(&self) -> &UserName {
        &self.user_name
    }

    pub fn access_token(&self) -> &AccessToken {
        &self.access_token
    }

    pub fn refresh_token(&self) -> &RefreshToken {
        &self.refresh_token
    }

    pub fn set(&mut self, access_token: AccessToken, refresh_token: RefreshToken) {
        self.access_token = access_token;
        self.refresh_token = refresh_token;
    }
}
