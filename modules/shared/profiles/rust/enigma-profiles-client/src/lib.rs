use async_trait::async_trait;
use enigma_auth::UserId;
use enigma_profiles::EnigmaUserProfile;
use mockall::automock;
use std::error::Error;

#[cfg(feature = "grpc")]
pub mod grpc;

#[async_trait]
#[automock]
pub trait EnigmaProfilesClient {
    async fn create_profile(&self, profile: &EnigmaUserProfile) -> Result<(), Box<dyn Error>>;

    async fn delete_profile(&self, user_id: &UserId) -> Result<(), Box<dyn Error>>;

    async fn get_profile(&self, user_id: &UserId) -> Result<EnigmaUserProfile, Box<dyn Error>>;

    async fn update_profile(&self, profile: &EnigmaUserProfile) -> Result<(), Box<dyn Error>>;

    async fn request_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), Box<dyn Error>>;

    async fn accept_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), Box<dyn Error>>;

    async fn reject_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
    ) -> Result<(), Box<dyn Error>>;

    async fn get_connections(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<(String, EnigmaUserProfile)>, Box<dyn Error>>;

    async fn add_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), Box<dyn Error>>;

    async fn remove_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
    ) -> Result<(), Box<dyn Error>>;

    async fn update_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), Box<dyn Error>>;
}
