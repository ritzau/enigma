use crate::db::{DatabaseError, EnigmaProfilesDatabase};
use crate::EnigmaProfilesService;
use async_trait::async_trait;
use enigma_auth::UserId;
use enigma_profiles::{EnigmaConnection, EnigmaPost, EnigmaUserProfile, PostId};

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
    // Profiles

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

    async fn search_profiles(&self, query: &str) -> Result<Vec<EnigmaUserProfile>, DatabaseError> {
        self.db.search_profiles(query).await
    }

    // Connections

    async fn request_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
        relationship: &str,
    ) -> Result<EnigmaConnection, DatabaseError> {
        self.db
            .request_connection(user_id, peer_id, relationship)
            .await
    }

    async fn accept_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
        relationship: &str,
    ) -> Result<EnigmaConnection, DatabaseError> {
        self.db
            .accept_connection(user_id, peer_id, relationship)
            .await
    }

    async fn reject_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
    ) -> Result<(), DatabaseError> {
        self.db.reject_connection(user_id, peer_id).await
    }

    async fn list_connections(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<EnigmaConnection>, DatabaseError> {
        self.db.list_connections(user_id).await
    }

    async fn add_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
        relationship: &str,
    ) -> Result<EnigmaConnection, DatabaseError> {
        if user_id == peer_id {
            return Err(DatabaseError::CannotConnectToSelf(
                "Cannot connect to self",
                None,
            ));
        }
        self.db.add_connection(user_id, peer_id, relationship).await
    }

    async fn remove_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
    ) -> Result<(), DatabaseError> {
        self.db.remove_connection(user_id, peer_id).await
    }

    async fn update_connection(
        &self,
        user_id: &UserId,
        peer_id: &UserId,
        relationship: &str,
    ) -> Result<EnigmaConnection, DatabaseError> {
        self.db
            .update_connection(user_id, peer_id, relationship)
            .await
    }

    // Posts

    async fn create_post(&self, user_id: &UserId, content: &str) -> Result<PostId, DatabaseError> {
        self.db.create_post(user_id, content).await
    }

    async fn delete_post(
        &self,
        post_id: &PostId,
        user_id: Option<&UserId>,
    ) -> Result<(), DatabaseError> {
        self.db.delete_post(post_id, user_id).await
    }

    async fn list_profile_posts(&self, user_id: &UserId) -> Result<Vec<EnigmaPost>, DatabaseError> {
        self.db.list_profile_posts(user_id).await
    }

    async fn list_feed_posts(&self, user_id: &UserId) -> Result<Vec<EnigmaPost>, DatabaseError> {
        self.db.list_feed_posts(user_id).await
    }
}
