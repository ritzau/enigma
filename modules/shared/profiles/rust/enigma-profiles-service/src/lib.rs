use crate::db::DatabaseError;
use async_trait::async_trait;
use enigma_auth::UserId;
use enigma_profiles::{
    EnigmaConnection, EnigmaConnectionStatus, EnigmaPost, EnigmaUserProfile, PostId,
};
use serde::{Deserialize, Serialize};

pub mod db;
pub mod default;

#[cfg(feature = "grpc")]
pub mod grpc;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionsCursor {
    version: u8,
    user_id: UserId,
    update_seq: i64,
}

impl TryFrom<ConnectionsCursor> for String {
    type Error = serde_json::Error;

    fn try_from(cursor: ConnectionsCursor) -> Result<Self, Self::Error> {
        serde_json::to_string(&cursor)
    }
}

impl TryFrom<String> for ConnectionsCursor {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&value)
    }
}

#[async_trait]
pub trait EnigmaProfilesService {
    // Profiles

    async fn create_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError>;

    async fn delete_profile(&self, user_id: &UserId) -> Result<(), DatabaseError>;

    async fn get_profile(&self, user_id: &UserId) -> Result<EnigmaUserProfile, DatabaseError>;

    async fn update_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError>;

    async fn search_profiles(&self, query: &str) -> Result<Vec<EnigmaUserProfile>, DatabaseError>;

    // Connections

    async fn request_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
        relationship: &str,
    ) -> Result<EnigmaConnection, DatabaseError>;

    async fn accept_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
        relationship: &str,
    ) -> Result<EnigmaConnection, DatabaseError>;

    async fn reject_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
    ) -> Result<(), DatabaseError>;

    async fn list_connections(
        &self,
        user_id: &UserId,
        status_filter: &[EnigmaConnectionStatus],
        cursor: Option<ConnectionsCursor>,
        limit: Option<u16>,
    ) -> Result<(Vec<EnigmaConnection>, ConnectionsCursor, bool), DatabaseError>;

    async fn add_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
        relationship: &str,
    ) -> Result<EnigmaConnection, DatabaseError>;

    async fn remove_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
    ) -> Result<(), DatabaseError>;

    async fn update_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
        relationship: &str,
    ) -> Result<EnigmaConnection, DatabaseError>;

    // Posts

    async fn create_post(&self, user_id: &UserId, content: &str) -> Result<PostId, DatabaseError>;

    async fn delete_post(
        &self,
        post_id: &PostId,
        user_id: Option<&UserId>,
    ) -> Result<(), DatabaseError>;

    async fn list_profile_posts(&self, user_id: &UserId) -> Result<Vec<EnigmaPost>, DatabaseError>;

    async fn list_feed_posts(&self, user_id: &UserId) -> Result<Vec<EnigmaPost>, DatabaseError>;
}
