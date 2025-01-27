use async_trait::async_trait;
use enigma_auth::{AccessToken, RefreshToken, UserId, UserName};
use mockall::predicate::*;
use mockall::*;
use uuid::Uuid;

#[cfg(feature = "grpc")]
pub mod grpc;

pub mod authenticator;
pub mod session;

/// The `EnigmaAuthClient` trait provides asynchronous methods for creating accounts, deleting accounts,
/// retrieving session information, listing accounts, logging in, and purging expired sessions.
#[async_trait]
#[automock]
pub trait EnigmaAuthClient {
    ///
    /// # Parameters
    /// - `user_id: UserId`: The ID of the user.
    /// - `role: &str`: The role to be added.
    ///
    /// # Returns
    /// - `Result<(), Box<dyn std::error::Error>>`: The function returns a `Result` type.
    ///   On success, it returns an empty tuple `()`. On failure, it returns a boxed dynamic error.
    async fn add_role(
        &mut self,
        user_id: UserId,
        role: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

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

    /// Asynchronously changes a user's password.
    ///
    /// # Parameters
    /// - `user_id: i64`: The ID of the user.
    /// - `old_password: &str`: The user's current password.
    /// - `new_password: &str`: The user's new password.
    ///
    /// # Returns
    /// - `Result<(), Box<dyn std::error::Error>>`: The function returns a `Result` type.
    async fn change_password(
        &mut self,
        user_id: i64,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;

    /// Asynchronously refreshes a session.
    ///
    /// # Parameters
    /// - `refresh_token: &Uuid`: The refresh token for the session.
    ///
    /// # Returns
    /// - `Result<(AccessToken, RefreshToken), Box<dyn std::error::Error>>`: The function returns a `Result` type.
    ///   On success, it returns a tuple containing the new `AccessToken` and `RefreshToken`.
    ///   On failure, it returns a boxed dynamic error.
    async fn refresh_session(
        &self,
        refresh_token: &Uuid,
    ) -> Result<(AccessToken, RefreshToken), Box<dyn std::error::Error>>;

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

    /// Asynchronously retrieves user information.
    ///
    /// # Parameters
    /// - `user_id: UserId`: The ID of the user.
    ///
    /// # Returns
    /// - `Result<(UserId, UserName, Vec<String>), Box<dyn std::error::Error>>`: The function returns a `Result` type.
    ///   On success, it returns a tuple containing the `UserId`, `UserName`, and a vector of roles.
    ///   On failure, it returns a boxed dynamic error.
    async fn get_user_info(
        &mut self,
        user_id: UserId,
    ) -> Result<(UserId, UserName, Vec<String>), Box<dyn std::error::Error>>;

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
    ) -> Result<(AccessToken, RefreshToken), Box<dyn std::error::Error>>;

    /// Asynchronously purges expired sessions.
    ///
    /// # Returns
    /// - `Result<u64, Box<dyn std::error::Error>>`: The function returns a `Result` type.
    ///   On success, it returns a `u64` representing the number of purged sessions.
    ///   On failure, it returns a boxed dynamic error.
    async fn purge_expired_sessions(&mut self) -> Result<u64, Box<dyn std::error::Error>>;

    /// Asynchronously removes a role from a user.
    ///
    /// # Parameters
    /// - `user_id: UserId`: The ID of the user.
    /// - `role: &str`: The role to be removed.
    ///
    /// # Returns
    /// - `Result<(), Box<dyn std::error::Error>>`: The function returns a `Result` type.
    ///   On success, it returns an empty tuple `()`. On failure, it returns a boxed dynamic error.
    async fn remove_role(
        &mut self,
        user_id: UserId,
        role: &str,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
