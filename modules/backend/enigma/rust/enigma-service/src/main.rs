use enigma_auth::{AccessToken, AuthExtension};
use enigma_auth_grpc::auth_server::AuthServer;
use enigma_auth_service::db::postgres::PostgresAuthDatabase;
use enigma_auth_service::grpc::GrpcAuthService;
use enigma_auth_service::{DefaultAuthService, EnigmaAuthService};
use http::Request;
use http_body_util::combinators::UnsyncBoxBody;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::Mutex;
use tonic::body::BoxBody;
use tonic::codegen::{http, Bytes};
use tonic::server::NamedService;
use tonic::transport::Server;
use tonic::Status;
use tower::layer::layer_fn;
use tower::{Service, ServiceBuilder};
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .init();

    let addr = "[::]:50051".parse().unwrap();
    info!(addr = %addr, "Enigma Server listening on {}", addr);

    let db = PostgresAuthDatabase::new().await?;
    let auth = Arc::new(Mutex::new(DefaultAuthService::new(db)));
    let grpc_auth = GrpcAuthService::new(auth.clone());
    let server = AuthServer::new(grpc_auth);

    let service = ServiceBuilder::new()
        .layer(layer_fn(move |inner| {
            AuthMiddleware::new(inner, auth.clone())
        }))
        .service(server);

    Server::builder().add_service(service).serve(addr).await?;

    info!("Enigma Server shutting down");

    Ok(())
}

struct AuthMiddleware<S, T: EnigmaAuthService> {
    inner: S,
    auth_service: Arc<Mutex<T>>,
}

impl<S, T: EnigmaAuthService> AuthMiddleware<S, T> {
    fn new(inner: S, auth_service: Arc<Mutex<T>>) -> Self {
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
            Response = http::response::Response<UnsyncBoxBody<Bytes, Status>>,
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
            let mut request = request;

            if let Some(access_token) = get_access_token(&request) {
                let session = auth_service
                    .lock()
                    .await
                    .get_session(&access_token)
                    .await
                    .ok();
                if let Some((is_valid, user_id)) = session {
                    match (is_valid, user_id) {
                        (true, Some(user_id)) => {
                            if let Ok((_, username, roles)) =
                                auth_service.lock().await.get_user_info(&user_id).await
                            {
                                request
                                    .extensions_mut()
                                    .insert(AuthExtension::Authenticated(user_id, username, roles));
                            }
                        }
                        _ => {
                            request.extensions_mut().insert(AuthExtension::Failed);
                        }
                    }
                }
            } else {
                request.extensions_mut().insert(AuthExtension::Anonymous);
            }

            inner.call(request).await
        })
    }
}

fn get_access_token(request: &Request<BoxBody>) -> Option<AccessToken> {
    if let Some(access_token_header) = request.headers().get("authorization") {
        let access_token = access_token_header
            .to_str()
            .unwrap()
            .strip_prefix("Bearer ")
            .unwrap();

        match Uuid::parse_str(access_token) {
            Ok(uuid) => Some(AccessToken::from(uuid)),
            Err(_) => None,
        }
    } else {
        None
    }
}
