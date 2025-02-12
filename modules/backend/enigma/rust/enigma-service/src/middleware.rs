use enigma_auth::{AccessToken, AuthExtension, UserId, UserName};
use enigma_auth_service::EnigmaAuthService;
use http::response::Response;
use http_body_util::combinators::UnsyncBoxBody;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tonic::body::BoxBody;
use tonic::codegen::http::Request;
use tonic::codegen::{http, Bytes, Service};
use tonic::server::NamedService;
use tonic::Status;
use uuid::Uuid;

pub struct AuthMiddleware<S, T: EnigmaAuthService> {
    inner: S,
    auth_service: Arc<T>,
}

impl<S, T: EnigmaAuthService> AuthMiddleware<S, T> {
    pub fn new(inner: S, auth_service: Arc<T>) -> Self {
        Self {
            inner,
            auth_service,
        }
    }
}

impl<S: Clone, T: EnigmaAuthService> Clone for AuthMiddleware<S, T> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone(), self.auth_service.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.inner = source.inner.clone();
        self.auth_service = source.auth_service.clone();
    }
}

impl<S, T: EnigmaAuthService> NamedService for AuthMiddleware<S, T>
where
    S: NamedService,
{
    const NAME: &'static str = S::NAME;
}

impl<S, T> Service<Request<BoxBody>> for AuthMiddleware<S, T>
where
    S: Service<
            Request<BoxBody>,
            Response = Response<UnsyncBoxBody<Bytes, Status>>,
            Error = Infallible,
        > + Clone
        + Send
        + 'static,
    S::Future: Send,
    T: EnigmaAuthService + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request<BoxBody>) -> Self::Future {
        let mut inner = self.inner.clone();
        let auth_service = self.auth_service.clone();
        Box::pin(async move {
            let request = process_request(auth_service, request).await;
            inner.call(request).await
        })
    }
}

async fn process_request(
    auth_service: Arc<impl EnigmaAuthService>,
    mut request: Request<BoxBody>,
) -> Request<BoxBody> {
    let Some(access_token) = get_access_token(&request) else {
        request.extensions_mut().insert(AuthExtension::Anonymous);
        return request;
    };

    let Some((user_id, username, roles)) = get_session(auth_service, &access_token).await else {
        request.extensions_mut().insert(AuthExtension::Failed);
        return request;
    };

    request
        .extensions_mut()
        .insert(AuthExtension::Authenticated(user_id, username, roles));

    request
}

async fn get_session(
    auth_service: Arc<impl EnigmaAuthService + Sized>,
    access_token: &AccessToken,
) -> Option<(UserId, UserName, Vec<String>)> {
    let user_id = match auth_service.get_session(access_token).await {
        Ok((true, Some(user_id))) => user_id,
        _ => return None,
    };

    auth_service.get_user_info(&user_id).await.ok()
}

fn get_access_token(request: &Request<BoxBody>) -> Option<AccessToken> {
    let access_token_header = request.headers().get("authorization")?;
    let access_token = access_token_header.to_str().ok()?.strip_prefix("Bearer ")?;
    let uuid = Uuid::parse_str(access_token).ok()?;
    Some(AccessToken::from(uuid))
}
