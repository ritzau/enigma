//! This module defines the `EnigmaAuthClient` trait and related types for user authentication.
//!
//! The `EnigmaAuthClient` trait provides asynchronous methods for creating accounts, deleting accounts,
//! retrieving session information, listing accounts, logging in, and purging expired sessions.
//!
//! The following types are defined in this module:
//! - `UserId`: Represents a user ID.
//! - `UserName`: Represents a user name.
//! - `UserHash`: Represents a hashed user password.
//! - `AccessToken`: Represents an access token.

use std::fmt::Display;
use uuid::Uuid;

/// Represents a user ID.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UserId(i64);

impl UserId {
    /// Returns the user ID as an `i64`.
    pub fn value(&self) -> i64 {
        self.0
    }
}

impl Display for UserId {
    /// Formats the user ID.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for UserId {
    /// Converts an `i64` to a `UserId`.
    fn from(value: i64) -> Self {
        UserId(value)
    }
}

impl From<UserId> for i64 {
    /// Converts a `UserId` to an `i64`.
    fn from(value: UserId) -> Self {
        value.0
    }
}

/// Represents a user name.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UserName(String);

impl UserName {
    /// Returns the user name as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for UserName {
    /// Formats the user name.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for UserName {
    /// Converts a `&str` to a `UserName`.
    fn from(value: &str) -> Self {
        UserName(value.to_string())
    }
}

impl From<String> for UserName {
    /// Converts a `String` to a `UserName`.
    fn from(value: String) -> Self {
        UserName(value)
    }
}

/// Represents a hashed user password.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// Returns the password hash as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for PasswordHash {
    /// Formats the password hash.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.clone())
    }
}

impl From<&str> for PasswordHash {
    /// Converts a `&str` to a `PasswordHash`.
    fn from(value: &str) -> Self {
        PasswordHash(value.to_string())
    }
}

impl From<String> for PasswordHash {
    /// Converts a `String` to a `PasswordHash`.
    fn from(value: String) -> Self {
        PasswordHash(value)
    }
}

/// Represents an access token.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AccessToken(Uuid);

impl From<Uuid> for AccessToken {
    /// Converts a `Uuid` to an `AccessToken`.
    fn from(value: Uuid) -> Self {
        AccessToken(value)
    }
}

impl AccessToken {
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the access token as a `Uuid`.
    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl Display for AccessToken {
    /// Formats the access token.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Represents a refresh token.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RefreshToken(Uuid);

impl From<Uuid> for RefreshToken {
    /// Converts a `Uuid` to an `AccessToken`.
    fn from(value: Uuid) -> Self {
        RefreshToken(value)
    }
}

impl RefreshToken {
    pub fn new_random() -> Self {
        Self(Uuid::new_v4())
    }

    /// Returns the refresh token as a `Uuid`.
    pub fn value(&self) -> Uuid {
        self.0
    }
}

impl Display for RefreshToken {
    /// Formats the refresh token.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_user_id() {
        let user_id = UserId::from(42);
        assert_eq!(user_id.value(), 42);
        assert_eq!(user_id.to_string(), "42");

        let user_id_i64: i64 = user_id.into();
        assert_eq!(user_id_i64, 42);
    }

    #[test]
    fn test_user_name() {
        let user_name = UserName::from("test_user");
        assert_eq!(user_name.as_str(), "test_user");
        assert_eq!(user_name.to_string(), "test_user");

        let user_name_str: UserName = "test_user".into();
        assert_eq!(user_name_str.as_str(), "test_user");

        let user_name_string: UserName = "test_user".to_string().into();
        assert_eq!(user_name_string.as_str(), "test_user");
    }

    #[test]
    fn test_password_hash() {
        let password_hash = PasswordHash::from("hashed_password");
        assert_eq!(password_hash.as_str(), "hashed_password");
        assert_eq!(password_hash.to_string(), "hashed_password");

        let password_hash_str: PasswordHash = "hashed_password".into();
        assert_eq!(password_hash_str.as_str(), "hashed_password");

        let password_hash_string: PasswordHash = "hashed_password".to_string().into();
        assert_eq!(password_hash_string.as_str(), "hashed_password");
    }

    #[test]
    fn test_access_token() {
        let uuid = Uuid::new_v4();
        let access_token = AccessToken::from(uuid);
        assert_eq!(access_token.value(), uuid);
        assert_eq!(access_token.to_string(), uuid.to_string());

        let new_access_token = AccessToken::new_random();
        assert_ne!(new_access_token.value(), uuid);
    }

    #[test]
    fn test_refresh_token() {
        let uuid = Uuid::new_v4();
        let refresh_token = RefreshToken::from(uuid);
        assert_eq!(refresh_token.value(), uuid);
        assert_eq!(refresh_token.to_string(), uuid.to_string());

        let new_refresh_token = RefreshToken::new_random();
        assert_ne!(new_refresh_token.value(), uuid);
    }
}
