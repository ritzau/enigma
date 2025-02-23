use crate::ConnectionsCursor;
use async_trait::async_trait;
use enigma_auth::UserId;
use enigma_profiles::{
    EnigmaConnection, EnigmaConnectionStatus, EnigmaPost, EnigmaUserProfile, PostId,
};
use std::error::Error;
use std::fmt::Display;

pub mod postgres;

#[derive(Debug)]
pub enum DatabaseError {
    InvalidArgument(&'static str, Option<Box<dyn Error>>),
    IllegalState(&'static str, Option<Box<dyn Error>>),
    InvalidUser(&'static str, Option<Box<dyn Error>>),
    NotFound(&'static str, Option<Box<dyn Error>>),
    CannotConnectToSelf(&'static str, Option<Box<dyn Error>>),
}

impl Error for DatabaseError {}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::InvalidArgument(msg, None)
            | DatabaseError::IllegalState(msg, None)
            | DatabaseError::InvalidUser(msg, None)
            | DatabaseError::NotFound(msg, None)
            | DatabaseError::CannotConnectToSelf(msg, None) => {
                write!(f, "{}", msg)
            }
            DatabaseError::InvalidArgument(msg, Some(err))
            | DatabaseError::IllegalState(msg, Some(err))
            | DatabaseError::InvalidUser(msg, Some(err))
            | DatabaseError::NotFound(msg, Some(err))
            | DatabaseError::CannotConnectToSelf(msg, Some(err)) => {
                write!(f, "{} ({})", msg, err)
            }
        }
    }
}

#[async_trait]
pub trait EnigmaProfilesDatabase {
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

    async fn add_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
        relationship: &str,
    ) -> Result<EnigmaConnection, DatabaseError>;

    async fn get_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
    ) -> Result<EnigmaConnection, DatabaseError>;

    async fn list_connections(
        &self,
        user_id: &UserId,
        status_filter: &[EnigmaConnectionStatus],
        cursor: Option<ConnectionsCursor>,
        limit: Option<u16>,
    ) -> Result<(Vec<EnigmaConnection>, ConnectionsCursor, bool), DatabaseError>;

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
