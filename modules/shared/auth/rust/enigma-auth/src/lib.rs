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
use tonic::async_trait;

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

/// The `EnigmaAuthClient` trait provides asynchronous methods for creating accounts, deleting accounts,
/// retrieving session information, listing accounts, logging in, and purging expired sessions.
#[async_trait]
pub trait EnigmaAuthClient {
    /// Asynchronously creates a new user account.
    ///
    /// # Parameters
    /// - `username: &str`: The username for the new account.
    /// - `password: &str`: The password for the new account.
    ///
    /// # Returns
    /// - `Result<UserId, Box<dyn std::error::Error>>`: The function returns a `Result` type.
    ///   On success, it returns a `UserId` representing the ID of the newly created user.
    ///   On failure, it returns a boxed dynamic error. The function fails if the username or password
    ///   are invalid or do not meet the bar, or if the user already exists.
    async fn create_account(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<UserId, Box<dyn std::error::Error>>;

    /// Asynchronously deletes a user account.
    ///
    /// # Parameters
    /// - `user_id: &UserId`: The ID of the user to be deleted.
    ///
    /// # Returns
    /// - `Result<(), Box<dyn std::error::Error>>`: The function returns a `Result` type.
    ///   On success, it returns an empty tuple `()`. On failure, it returns a boxed dynamic error.
    async fn delete_account(&mut self, user_id: &UserId) -> Result<(), Box<dyn std::error::Error>>;

    /// Asynchronously retrieves session information.
    ///
    /// # Parameters
    /// - `access_token: &Uuid`: The access token for the session.
    ///
    /// # Returns
    /// - `Result<UserId, Box<dyn std::error::Error>>`: The function returns a `Result` type.
    ///   On success, it returns a `UserId` representing the ID of the user associated with the session.
    ///   On failure, it returns a boxed dynamic error.
    async fn get_session(
        &mut self,
        access_token: &Uuid,
    ) -> Result<UserId, Box<dyn std::error::Error>>;

    /// Asynchronously lists all user accounts.
    ///
    /// # Returns
    /// - `Result<Vec<(UserId, String)>, Box<dyn std::error::Error>>`: The function returns a `Result` type.
    ///   On success, it returns a vector of tuples, each containing a `UserId` and a `String` representing the username.
    ///   On failure, it returns a boxed dynamic error.
    async fn list_accounts(&mut self) -> Result<Vec<(UserId, String)>, Box<dyn std::error::Error>>;

    /// Asynchronously logs in a user.
    ///
    /// # Parameters
    /// - `username: &str`: The username of the user.
    /// - `password: &str`: The password of the user.
    ///
    /// # Returns
    /// - `Result<String, Box<dyn std::error::Error>>`: The function returns a `Result` type.
    ///   On success, it returns a `String` representing the session ID.
    ///   On failure, it returns a boxed dynamic error.
    async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<String, Box<dyn std::error::Error>>;

    /// Asynchronously purges expired sessions.
    ///
    /// # Returns
    /// - `Result<u64, Box<dyn std::error::Error>>`: The function returns a `Result` type.
    ///   On success, it returns a `u64` representing the number of purged sessions.
    ///   On failure, it returns a boxed dynamic error.
    async fn purge_expired_sessions(&mut self) -> Result<u64, Box<dyn std::error::Error>>;
}
