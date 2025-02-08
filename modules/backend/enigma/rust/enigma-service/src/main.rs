use enigma_auth_grpc::auth_server::AuthServer;
use enigma_auth_service::db::postgres::PostgresAuthDatabase;
use enigma_auth_service::default::DefaultAuthService;
use enigma_auth_service::grpc::GrpcAuthService;
use enigma_auth_service::EnigmaAuthService;
use enigma_profiles_grpc::profiles_server::ProfilesServer;
use enigma_profiles_service::db::postgres::PostgresProfilesDatabase;
use enigma_profiles_service::default::DefaultProfilesService;
use enigma_profiles_service::grpc::GrpcProfilesService;
use enigma_profiles_service::EnigmaProfilesService;
use enigma_service::middleware::AuthMiddleware;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;
use tower::layer::layer_fn;
use tower::ServiceBuilder;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .init();

    let addr = "[::]:50051".parse().unwrap();
    info!(addr = %addr, "Enigma Server listening on {}", addr);

    let (auth, authenticated_auth_service) = create_authenticating_auth_server().await?;
    let authenticated_profiles_service = create_authenticating_profiles_server(auth).await?;

    Server::builder()
        .add_service(authenticated_auth_service)
        .add_service(authenticated_profiles_service)
        .serve(addr)
        .await?;

    info!("Enigma Server shutting down");

    Ok(())
}

type DefaultPostgresAuthService = DefaultAuthService<PostgresAuthDatabase>;
type DefaultGrpcAuthServer = AuthServer<GrpcAuthService<DefaultPostgresAuthService>>;

async fn create_authenticating_auth_server() -> Result<
    (
        Arc<Mutex<DefaultPostgresAuthService>>,
        AuthMiddleware<DefaultGrpcAuthServer, impl EnigmaAuthService + Sized>,
    ),
    Box<dyn Error>,
> {
    let db = PostgresAuthDatabase::new().await?;
    let auth = Arc::new(Mutex::new(DefaultAuthService::new(db)));
    let grpc_auth = GrpcAuthService::new(auth.clone());
    let auth_server = AuthServer::new(grpc_auth);
    let authenticated_auth_service = with_authentication(auth.clone(), auth_server);
    Ok((auth, authenticated_auth_service))
}

async fn create_authenticating_profiles_server(
    auth: Arc<Mutex<DefaultPostgresAuthService>>,
) -> Result<
    AuthMiddleware<
        ProfilesServer<GrpcProfilesService<impl EnigmaProfilesService + Sized>>,
        impl EnigmaAuthService + Sized,
    >,
    Box<dyn Error>,
> {
    let db = PostgresProfilesDatabase::new().await?;
    let profiles = Arc::new(Mutex::new(DefaultProfilesService::new(db)));
    let grpc_profiles = GrpcProfilesService::new(profiles.clone());
    let profiles_server = ProfilesServer::new(grpc_profiles);
    let authenticated_profiles_service = with_authentication(auth, profiles_server);
    Ok(authenticated_profiles_service)
}

fn with_authentication<S>(
    auth: Arc<Mutex<impl EnigmaAuthService>>,
    auth_server: S,
) -> AuthMiddleware<S, impl EnigmaAuthService> {
    ServiceBuilder::new()
        .layer(layer_fn(move |inner| {
            AuthMiddleware::new(inner, auth.clone())
        }))
        .service(auth_server)
}
