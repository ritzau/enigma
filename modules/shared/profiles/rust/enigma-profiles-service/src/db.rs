use async_trait::async_trait;
use enigma_auth::UserId;
use enigma_profiles::EnigmaUserProfile;
use std::error::Error;
use std::fmt::Display;

pub mod postgres;

#[derive(Debug)]
pub enum DatabaseError {
    IllegalState(&'static str, Option<Box<dyn Error>>),
    InvalidUser(&'static str, Option<Box<dyn Error>>),
}

impl Error for DatabaseError {}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::IllegalState(msg, _) | DatabaseError::InvalidUser(msg, _) => {
                write!(f, "{}", msg)
            }
        }
    }
}

#[async_trait]
pub trait EnigmaProfilesDatabase {
    async fn create_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError>;
    async fn delete_profile(&self, user_id: &UserId) -> Result<(), DatabaseError>;
    async fn get_profile(&self, user_id: &UserId) -> Result<EnigmaUserProfile, DatabaseError>;
    async fn update_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError>;
}
