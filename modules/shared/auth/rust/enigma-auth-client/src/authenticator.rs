use crate::grpc::RefreshSessionParameters;
use crate::session::Session;
use enigma_auth::{AccessToken, RefreshToken};
use enigma_auth_grpc::auth_client::AuthClient;
use http_body::Body;
use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tonic::body::BoxBody;
use tonic::client::GrpcService;
use tonic::codegen::{Bytes, StdError};
use tonic::{Code, IntoRequest, Request, Status};
use tracing::instrument;
use uuid::Uuid;

pub struct Authenticator<T>
where
    T: GrpcService<BoxBody>,
    T::Error: Into<StdError>,
    T::Future: Send + 'static,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    auth_client: Arc<Mutex<AuthClient<T>>>,
    session: Arc<RwLock<Session>>,
}

impl<T> Authenticator<T>
where
    T: GrpcService<BoxBody>,
    T::Error: Into<StdError>,
    T::Future: Send + 'static,
    T::ResponseBody: Body<Data = Bytes> + Send + 'static,
    <T::ResponseBody as Body>::Error: Into<StdError> + Send,
{
    pub fn new(auth_client: Arc<Mutex<AuthClient<T>>>, session: Arc<RwLock<Session>>) -> Self {
        Self {
            auth_client,
            session,
        }
    }

    pub async fn authenticated_call<Ret, FReq, Req, F, Fut, IR>(
        &self,
        request_factory: FReq,
        request_fn: F,
    ) -> Result<Ret, Status>
    where
        F: Fn(Request<Req>) -> Fut,
        Fut: Future<Output = Result<Ret, Status>>,
        FReq: Fn() -> IR,
        IR: IntoRequest<Req>,
    {
        match self
            .call_with_access_token(&request_factory, &request_fn)
            .await
        {
            Ok(res) => Ok(res),
            Err(status) if status.code() == Code::Unauthenticated => {
                self.refresh_and_update_session().await?;
                self.call_with_access_token(&request_factory, &request_fn)
                    .await
            }
            Err(err) => Err(err),
        }
    }

    async fn call_with_access_token<Req, Ret, FReq, F, Fut, IR>(
        &self,
        request_factory: &FReq,
        request_fn: &F,
    ) -> Result<Ret, Status>
    where
        F: Fn(Request<Req>) -> Fut,
        Fut: Future<Output = Result<Ret, Status>>,
        FReq: Fn() -> IR,
        IR: IntoRequest<Req>,
    {
        let mut request = request_factory().into_request();
        request.metadata_mut().insert(
            "authorization",
            format!("Bearer {}", self.session.read().await.access_token())
                .parse()
                .unwrap(),
        );
        request_fn(request).await
    }

    #[instrument(err, skip_all)]
    async fn refresh_and_update_session(&self) -> Result<(), Status> {
        let refresh_token = self.session.read().await.refresh_token().clone();
        let (access_token, refresh_token) = self.refresh_session(refresh_token).await?;

        let mut session = self.session.write().await;
        session.set(access_token, refresh_token);
        session.store(Path::new("enigma-session.toml")).unwrap();

        Ok(())
    }

    async fn refresh_session(
        &self,
        refresh_token: RefreshToken,
    ) -> Result<(AccessToken, RefreshToken), Status> {
        let mut client = self.auth_client.lock().await;
        let response = client
            .refresh_session(RefreshSessionParameters(refresh_token))
            .await?
            .into_inner();
        let access_token = AccessToken::from(
            Uuid::parse_str(&response.access_token)
                .map_err(|_| Status::invalid_argument("Invalid access token"))?,
        );

        let refresh_token = RefreshToken::from(
            Uuid::parse_str(&response.refresh_token)
                .map_err(|_| Status::invalid_argument("Invalid refresh token"))?,
        );

        Ok((access_token, refresh_token))
    }
}
