use crate::db::DatabaseError;
use async_trait::async_trait;
use db::EnigmaProfilesDatabase;
use enigma_auth::UserId;
use enigma_profiles::EnigmaUserProfile;

pub mod db;

#[cfg(feature = "grpc")]
pub mod grpc;

#[async_trait]
pub trait EnigmaProfilesService {
    async fn create_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError>;

    async fn delete_profile(&self, user_id: &UserId) -> Result<(), DatabaseError>;

    async fn get_profile(&self, user_id: &UserId) -> Result<EnigmaUserProfile, DatabaseError>;

    async fn update_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError>;

    async fn get_connections(&self, user_id: &UserId);
    async fn add_connection(&self, user_id: &UserId, connection_id: &UserId, kind: &str);
    async fn remove_connection(&self, user_id: &UserId, connection_id: &UserId);
    async fn update_connection(&self, user_id: &UserId, connection_id: &UserId, kind: &str);
}

pub struct DefaultProfilesService<DB>
where
    DB: EnigmaProfilesDatabase,
{
    db: DB,
}

impl<DB: EnigmaProfilesDatabase> DefaultProfilesService<DB> {
    pub fn new(db: DB) -> Self {
        Self { db }
    }
}

#[async_trait]
impl<DB> EnigmaProfilesService for DefaultProfilesService<DB>
where
    DB: EnigmaProfilesDatabase + Send + Sync,
{
    async fn create_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError> {
        self.db.create_profile(profile).await
    }

    async fn delete_profile(&self, user_id: &UserId) -> Result<(), DatabaseError> {
        self.db.delete_profile(user_id).await
    }

    async fn get_profile(&self, user_id: &UserId) -> Result<EnigmaUserProfile, DatabaseError> {
        self.db.get_profile(user_id).await
    }

    async fn update_profile(&self, profile: &EnigmaUserProfile) -> Result<(), DatabaseError> {
        self.db.update_profile(profile).await
    }

    async fn get_connections(&self, _user_id: &UserId) {
        todo!()
    }

    async fn add_connection(&self, _user_id: &UserId, _connection_id: &UserId, _kind: &str) {
        todo!()
    }

    async fn remove_connection(&self, _user_id: &UserId, _connection_id: &UserId) {
        todo!()
    }

    async fn update_connection(&self, _user_id: &UserId, _connection_id: &UserId, _kind: &str) {
        todo!()
    }
}
