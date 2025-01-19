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

use sqlx::types::Uuid;
use std::fmt::Display;
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
