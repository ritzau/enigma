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

    async fn request_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError> {
        self.db
            .request_connection(user_id, connection_id, kind)
            .await
    }

    async fn accept_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError> {
        self.db
            .accept_connection(user_id, connection_id, kind)
            .await
    }

    async fn reject_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
    ) -> Result<(), DatabaseError> {
        self.db.reject_connection(user_id, connection_id).await
    }

    async fn get_connections(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<(String, EnigmaUserProfile)>, DatabaseError> {
        self.db.get_connections(user_id).await
    }

    async fn add_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError> {
        if user_id == connection_id {
            return Err(DatabaseError::CannotConnectToSelf(
                "Cannot connect to self",
                None,
            ));
        }
        self.db.add_connection(user_id, connection_id, kind).await
    }

    async fn remove_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
    ) -> Result<(), DatabaseError> {
        self.db.remove_connection(user_id, connection_id).await
    }

    async fn update_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), DatabaseError> {
        self.db
            .update_connection(user_id, connection_id, kind)
            .await
    }
}
