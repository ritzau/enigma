use enigma_auth_grpc::auth_server::AuthServer;
use enigma_auth_service::db::postgres::PostgresAuthDatabase;
use enigma_auth_service::grpc::GrpcAuthService;
use enigma_auth_service::DefaultAuthService;
use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .init();

    let addr = "[::]:50051".parse().unwrap();
    info!(addr = %addr, "Enigma Server listening on {}", addr);

    let db = PostgresAuthDatabase::new().await?;
    let auth = DefaultAuthService::new(db);
    let grpc_auth = GrpcAuthService::new(auth);

    Server::builder()
        .add_service(AuthServer::new(grpc_auth))
        .serve(addr)
        .await?;

    info!("Enigma Server shutting down");

    Ok(())
}
