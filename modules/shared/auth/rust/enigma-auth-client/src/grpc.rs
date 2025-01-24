use crate::{session, EnigmaAuthClient};
use async_trait::async_trait;
use enigma_auth::{AccessToken, RefreshToken, UserId, UserName};
use enigma_auth_grpc::{
    auth_client::AuthClient, AddRoleRequest, ChangePasswordRequest, CreateAccountRequest,
    DeleteAccountRequest, GetSessionRequest, GetUserInfoRequest, ListAccountsRequest, LoginRequest,
    PurgeExpiredSessionsRequest, RefreshSessionRequest, RemoveRoleRequest,
};
use http_body::Body;
use session::Session;
use std::convert::Into;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tonic::transport::Channel;
use tonic::IntoRequest;
use tracing::instrument;
use uuid::Uuid;

use crate::authenticator::Authenticator;
use tonic::body::BoxBody;
use tonic::client::GrpcService;
use tonic::codegen::{Bytes, StdError};
use tonic::Request;

pub struct GrpcAuthClient<T>
where
    T: GrpcService<BoxBody>,
    T::Error: Into<StdError>,
    T::Future: Send + 'static,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    client: Arc<Mutex<AuthClient<T>>>,
    authenticator: Authenticator<T>,
}

impl<T> GrpcAuthClient<T>
where
    T: GrpcService<BoxBody> + Send + Sync + 'static,
    T::Error: Into<StdError>,
    T::Future: Send + 'static,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(client: AuthClient<T>) -> Self {
        // TODO(ENIGMA-23): Hard coded a few time too many...
        let session = Arc::new(RwLock::new(
            Session::load(Path::new("enigma-session.toml")).unwrap(),
        ));
        let client = Arc::new(Mutex::new(client));
        let authenticator = Authenticator::<T>::new(client.clone(), session.clone());
        Self {
            client,
            authenticator,
        }
    }
}

impl GrpcAuthClient<Channel> {
    #[instrument(err)]
    pub async fn connect(url: &str) -> Result<Self, Box<dyn Error>> {
        let channel = Channel::from_shared(url.to_string())?.connect().await?;
        Ok(Self::new(AuthClient::new(channel)))
    }

    pub async fn default() -> Result<Self, Box<dyn Error>> {
        Self::connect("http://[::1]:50051").await
    }
}

#[async_trait]
impl<T> EnigmaAuthClient for GrpcAuthClient<T>
where
    T: GrpcService<BoxBody> + Send + Sync + 'static,
    T::Error: Into<StdError>,
    T::Future: Send + 'static,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    async fn add_role(&mut self, user_id: UserId, role: &str) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || AddRoleParameters(user_id.clone(), role.into()),
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.add_role(request).await }
                },
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, password), err)]
    async fn create_account(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<UserId, Box<dyn Error>> {
        let response = self
            .authenticator
            .authenticated_call(
                || CreateAccountParameters(UserName::from(username), password.into()),
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.create_account(request).await }
                },
            )
            .await?;

        Ok(UserId::from(response.get_ref().user_id))
    }

    #[instrument(skip(self), err)]
    async fn delete_account(&mut self, user_id: &UserId) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || DeleteAccountParameters(user_id.clone()),
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.delete_account(request).await }
                },
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn change_password(
        &mut self,
        user_id: i64,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || {
                    ChangePasswordParameters(
                        UserId::from(user_id),
                        old_password.into(),
                        new_password.into(),
                    )
                },
                |r| {
                    let client = self.client.clone();
                    async move { client.lock().await.change_password(r).await }
                },
            )
            .await?;

        Ok(())
    }

    #[instrument(skip(self, refresh_token), err)]
    async fn refresh_session(
        &self,
        refresh_token: &Uuid,
    ) -> Result<(AccessToken, RefreshToken), Box<dyn Error>> {
        let request = Request::new(RefreshSessionRequest {
            refresh_token: refresh_token.to_string(),
        });
        let response = self.client.lock().await.refresh_session(request).await?;
        let response = response.get_ref();
        Ok((
            AccessToken::from(Uuid::parse_str(&response.access_token)?),
            RefreshToken::from(Uuid::parse_str(&response.refresh_token)?),
        ))
    }

    #[instrument(skip(self), err)]
    async fn get_session(&mut self, access_token: &Uuid) -> Result<UserId, Box<dyn Error>> {
        let request = Request::new(GetSessionRequest {
            access_token: access_token.to_string(),
        });

        let response = self.client.lock().await.get_session(request).await?;

        Ok(UserId::from(response.get_ref().user_id))
    }

    #[instrument(skip(self), err)]
    async fn get_user_info(
        &mut self,
        user_id: UserId,
    ) -> Result<(UserId, UserName, Vec<String>), Box<dyn Error>> {
        let response = self
            .authenticator
            .authenticated_call(
                || GetUserInfoParameter(user_id.clone()),
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.get_user_info(request).await }
                },
            )
            .await?
            .into_inner();

        Ok((
            UserId::from(response.user_id),
            UserName::from(response.username),
            response.roles,
        ))
    }

    #[instrument(skip(self), err)]
    async fn list_accounts(&mut self) -> Result<Vec<(UserId, String)>, Box<dyn Error>> {
        let response = self
            .authenticator
            .authenticated_call(
                || ListAccountParameters {},
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.list_accounts(request).await }
                },
            )
            .await?;

        let users = &response.get_ref().users;
        let result = users
            .iter()
            .map(|user| (UserId::from(user.id), user.name.clone()))
            .collect();

        Ok(result)
    }

    #[instrument(skip(self, password), err)]
    async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<(AccessToken, RefreshToken), Box<dyn Error>> {
        let request = Request::new(LoginRequest {
            name: username.into(),
            password: password.into(),
        });

        let response = self.client.lock().await.login(request).await?.into_inner();

        let session = Session::new(
            UserId::from(response.user_id),
            UserName::from(username),
            AccessToken::from(Uuid::parse_str(&response.access_token)?),
            RefreshToken::from(Uuid::parse_str(&response.refresh_token)?),
        );
        session.store(Path::new("enigma-session.toml"))?;

        Ok((
            session.access_token().clone(),
            session.refresh_token().clone(),
        ))
    }

    #[instrument(skip(self), err)]
    async fn purge_expired_sessions(&mut self) -> Result<u64, Box<dyn Error>> {
        let request = Request::new(PurgeExpiredSessionsRequest {});
        let response = self
            .client
            .lock()
            .await
            .purge_expired_sessions(request)
            .await?;
        Ok(response.get_ref().purged_session_count)
    }

    async fn remove_role(&mut self, user_id: UserId, role: &str) -> Result<(), Box<dyn Error>> {
        self.authenticator
            .authenticated_call(
                || RemoveRoleParameters(user_id.clone(), role.into()),
                |request| {
                    let client = self.client.clone();
                    async move { client.lock().await.remove_role(request).await }
                },
            )
            .await?;

        Ok(())
    }
}

struct AddRoleParameters(UserId, String);

impl IntoRequest<AddRoleRequest> for AddRoleParameters {
    fn into_request(self) -> Request<AddRoleRequest> {
        Request::new(AddRoleRequest {
            user_id: self.0.value(),
            role: self.1,
        })
    }
}

struct CreateAccountParameters(UserName, String);

impl IntoRequest<CreateAccountRequest> for CreateAccountParameters {
    fn into_request(self) -> Request<CreateAccountRequest> {
        Request::new(CreateAccountRequest {
            username: self.0.to_string(),
            password: self.1,
        })
    }
}

struct DeleteAccountParameters(UserId);

impl IntoRequest<DeleteAccountRequest> for DeleteAccountParameters {
    fn into_request(self) -> Request<DeleteAccountRequest> {
        Request::new(DeleteAccountRequest {
            user_id: self.0.value(),
        })
    }
}

struct ListAccountParameters;

impl IntoRequest<ListAccountsRequest> for ListAccountParameters {
    fn into_request(self) -> Request<ListAccountsRequest> {
        Request::new(ListAccountsRequest {})
    }
}

struct ChangePasswordParameters(UserId, String, String);

impl IntoRequest<ChangePasswordRequest> for ChangePasswordParameters {
    fn into_request(self) -> Request<ChangePasswordRequest> {
        Request::new(ChangePasswordRequest {
            user_id: self.0.value(),
            old_password: self.1,
            new_password: self.2,
        })
    }
}

pub struct GetUserInfoParameter(UserId);

impl IntoRequest<GetUserInfoRequest> for GetUserInfoParameter {
    fn into_request(self) -> Request<GetUserInfoRequest> {
        Request::new(GetUserInfoRequest {
            user_id: self.0.value(),
        })
    }
}

pub struct RefreshSessionParameters(pub RefreshToken);

impl IntoRequest<RefreshSessionRequest> for RefreshSessionParameters {
    fn into_request(self) -> Request<RefreshSessionRequest> {
        Request::new(RefreshSessionRequest {
            refresh_token: self.0.value().to_string(),
        })
    }
}

struct RemoveRoleParameters(UserId, String);

impl IntoRequest<RemoveRoleRequest> for RemoveRoleParameters {
    fn into_request(self) -> Request<RemoveRoleRequest> {
        Request::new(RemoveRoleRequest {
            user_id: self.0.value(),
            role: self.1,
        })
    }
}
