use crate::EnigmaProfilesClient;
use async_trait::async_trait;
use enigma_auth::UserId;
use enigma_auth_client::authenticator::Authenticator;
use enigma_auth_client::session::Session;
use enigma_auth_grpc::auth_client::AuthClient;
use enigma_profiles::EnigmaUserProfile;
use enigma_profiles_grpc::profiles_client::ProfilesClient;
use enigma_profiles_grpc::UserProfile;
use http_body::Body;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tonic::body::BoxBody;
use tonic::client::GrpcService;
use tonic::codegen::{Bytes, StdError};
use tonic::transport::Channel;
use tracing::instrument;

mod params;

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
        self.authenticator
            .authenticated_call(
                || params::CreateProfileParameters(profile.clone()),
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
                || params::DeleteProfileParameters(user_id.clone()),
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
                || params::GetProfileParameters(user_id.clone()),
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
        self.authenticator
            .authenticated_call(
                || params::UpdateProfileParameters(profile.clone()),
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.update_profile(request).await }
                },
            )
            .await?;

        Ok(())
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
