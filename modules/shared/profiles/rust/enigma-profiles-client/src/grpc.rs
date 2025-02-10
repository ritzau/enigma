use crate::EnigmaProfilesClient;
use async_trait::async_trait;
use enigma_auth::UserId;
use enigma_auth_client::authenticator::Authenticator;
use enigma_auth_client::session::Session;
use enigma_auth_grpc::auth_client::AuthClient;
use enigma_profiles::{EnigmaPost, EnigmaUserProfile, PostId};
use enigma_profiles_grpc::profiles_client::ProfilesClient;
use enigma_profiles_grpc::{
    AcceptConnectionRequest, AddConnectionRequest, CreatePostRequest, CreateProfileRequest,
    DeletePostRequest, DeleteProfileRequest, GetConnectionsRequest, GetProfileRequest,
    ListFeedPostsRequest, ListProfilePostsRequest, RejectConnectionRequest,
    RemoveConnectionRequest, RequestConnectionRequest, UpdateConnectionRequest,
    UpdateProfileRequest, UserProfile,
};
use http_body::Body;
use itertools::Itertools;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tonic::body::BoxBody;
use tonic::client::GrpcService;
use tonic::codegen::{Bytes, StdError};
use tonic::transport::Channel;
use tracing::instrument;

pub struct ProfilesGrpcClient<T>
where
    T: GrpcService<BoxBody>,
    T::Error: Into<StdError>,
    T::Future: Send + 'static,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    client: Arc<Mutex<ProfilesClient<T>>>,
    authenticator: Authenticator<T>,
}

impl<T> ProfilesGrpcClient<T>
where
    T: GrpcService<BoxBody> + Send + Sync + 'static,
    T::Error: Into<StdError>,
    T::Future: Send + 'static,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(auth_client: AuthClient<T>, profiles_client: ProfilesClient<T>) -> Self {
        // TODO(ENIGMA-23): Hard coded a few time too many...
        let session = Arc::new(RwLock::new(
            Session::load(Path::new("enigma-session.toml")).unwrap(),
        ));
        let client = Arc::new(Mutex::new(auth_client));
        // TODO(ENIGMA-31): Pass an arc to the authenticator
        let authenticator = Authenticator::<T>::new(client.clone(), session.clone());
        Self {
            client: Arc::new(Mutex::new(profiles_client)),
            authenticator,
        }
    }
}

impl ProfilesGrpcClient<Channel> {
    #[instrument(err)]
    pub async fn connect(
        auth_client: AuthClient<Channel>,
        url: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let channel = Channel::from_shared(url.to_string())?.connect().await?;
        Ok(Self::new(auth_client, ProfilesClient::new(channel)))
    }

    pub async fn localhost(auth_client: AuthClient<Channel>) -> Result<Self, Box<dyn Error>> {
        Self::connect(auth_client, "http://[::1]:50051").await
    }
}

#[async_trait]
impl<T> EnigmaProfilesClient for ProfilesGrpcClient<T>
where
    T: GrpcService<BoxBody> + Send + Sync + 'static,
    T::Error: Into<StdError>,
    T::Future: Send + 'static,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn create_profile(&self, profile: &EnigmaUserProfile) -> Result<(), Box<dyn Error>> {
        let profile_message = UserProfile::try_from(profile)?;

        self.authenticator
            .authenticated_call(
                || CreateProfileRequest {
                    profile: Some(profile_message.clone()),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.create_profile(request).await }
                },
            )
            .await?;

        Ok(())
    }

    #[instrument(err, skip(self))]
    async fn delete_profile(&self, user_id: &UserId) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || DeleteProfileRequest {
                    user_id: user_id.value(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.delete_profile(request).await }
                },
            )
            .await?;

        Ok(())
    }

    #[instrument(err, skip_all, fields(user_id = %user_id))]
    async fn get_profile(&self, user_id: &UserId) -> Result<EnigmaUserProfile, Box<dyn Error>> {
        let reply = self
            .authenticator
            .authenticated_call(
                || GetProfileRequest {
                    user_id: user_id.value(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.get_profile(request).await }
                },
            )
            .await?
            .into_inner();

        let profile_message: UserProfile = reply.profile.ok_or("Missing profile")?;
        Ok(EnigmaUserProfile::try_from(profile_message)?)
    }

    async fn update_profile(&self, profile: &EnigmaUserProfile) -> Result<(), Box<dyn Error>> {
        let profile_message = UserProfile::try_from(profile)?;

        self.authenticator
            .authenticated_call(
                || UpdateProfileRequest {
                    profile: Some(profile_message.clone()),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.update_profile(request).await }
                },
            )
            .await?;

        Ok(())
    }

    async fn request_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || RequestConnectionRequest {
                    user_id: user_id.value(),
                    connection_id: connection_id.value(),
                    kind: kind.to_string(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.request_connection(request).await }
                },
            )
            .await?;

        Ok(())
    }

    async fn accept_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || AcceptConnectionRequest {
                    user_id: user_id.value(),
                    connection_id: connection_id.value(),
                    kind: kind.to_string(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.accept_connection(request).await }
                },
            )
            .await?;

        Ok(())
    }

    async fn reject_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
    ) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || RejectConnectionRequest {
                    user_id: user_id.value(),
                    connection_id: connection_id.value(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.reject_connection(request).await }
                },
            )
            .await?;

        Ok(())
    }

    async fn get_connections(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<(String, EnigmaUserProfile)>, Box<dyn Error>> {
        let reply = self
            .authenticator
            .authenticated_call(
                || GetConnectionsRequest {
                    user_id: user_id.value(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.get_connections(request).await }
                },
            )
            .await?;

        Ok(reply
            .into_inner()
            .connections
            .iter()
            .map(|c| {
                let Some(ref profile) = c.profile else {
                    return Err("Missing profile");
                };

                let Ok(profile) = EnigmaUserProfile::try_from(profile) else {
                    return Err("Invalid profile");
                };

                Ok((c.kind.clone(), profile))
            })
            .try_collect()?)
    }

    async fn add_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || AddConnectionRequest {
                    user_id: user_id.value(),
                    connection_user_id: connection_id.value(),
                    kind: kind.to_string(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.add_connection(request).await }
                },
            )
            .await?;

        Ok(())
    }

    async fn remove_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
    ) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || RemoveConnectionRequest {
                    user_id: user_id.value(),
                    connection_user_id: connection_id.value(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.remove_connection(request).await }
                },
            )
            .await?;

        Ok(())
    }

    async fn update_connection(
        &self,
        user_id: &UserId,
        connection_id: &UserId,
        kind: &str,
    ) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || UpdateConnectionRequest {
                    user_id: user_id.value(),
                    connection_user_id: connection_id.value(),
                    kind: kind.to_string(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.update_connection(request).await }
                },
            )
            .await?;

        Ok(())
    }

    async fn create_post(&self, user_id: &UserId, content: &str) -> Result<PostId, Box<dyn Error>> {
        let response = self
            .authenticator
            .authenticated_call(
                || CreatePostRequest {
                    user_id: user_id.value(),
                    content: content.to_string(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.create_post(request).await }
                },
            )
            .await?
            .into_inner();

        Ok(PostId::from(response.post_id))
    }

    async fn delete_post(&self, post_id: &PostId) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || DeletePostRequest {
                    post_id: post_id.to_string(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.delete_post(request).await }
                },
            )
            .await?;

        Ok(())
    }

    async fn list_feed_posts(&self, user_id: &UserId) -> Result<Vec<EnigmaPost>, Box<dyn Error>> {
        let reply = self
            .authenticator
            .authenticated_call(
                || ListFeedPostsRequest {
                    user_id: user_id.value(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.list_feed_posts(request).await }
                },
            )
            .await?;

        Ok(reply
            .into_inner()
            .posts
            .iter()
            .map(EnigmaPost::try_from)
            .try_collect()?)
    }

    async fn list_profile_posts(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<EnigmaPost>, Box<dyn Error>> {
        let reply = self
            .authenticator
            .authenticated_call(
                || ListProfilePostsRequest {
                    user_id: user_id.value(),
                },
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.list_profile_posts(request).await }
                },
            )
            .await?;

        Ok(reply
            .into_inner()
            .posts
            .iter()
            .map(EnigmaPost::try_from)
            .try_collect()?)
    }
}
