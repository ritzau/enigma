use crate::db::DatabaseError;
use async_trait::async_trait;
use enigma_auth::UserId;
use enigma_profiles::EnigmaUserProfile;

pub mod db;
pub mod default;

#[cfg(feature = "grpc")]
pub mod grpc;

#[async_trait]
pub trait EnigmaProfilesService {
    async fn create_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError>;

    async fn delete_profile(&self, user_id: &UserId) -> Result<(), DatabaseError>;

    async fn get_profile(&self, user_id: &UserId) -> Result<EnigmaUserProfile, DatabaseError>;

    async fn update_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError>;

    async fn request_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError>;

    async fn accept_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError>;

    async fn reject_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
    ) -> Result<(), DatabaseError>;

    async fn get_connections(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<(String, EnigmaUserProfile)>, DatabaseError>;
    async fn add_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError>;

    async fn remove_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
    ) -> Result<(), DatabaseError>;
    async fn update_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError>;
}
