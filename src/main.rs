use foorum::auth::service::db::postgres::PostgresAuthDatabase;
use foorum::auth::service::grpc::auth_server::AuthServer;
use foorum::auth::service::grpc::GrpcAuthService;
use foorum::auth::service::DefaultAuthService;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    println!("Foorum Server listening on {}", addr);

    let db = PostgresAuthDatabase::new().await?;
    let auth = DefaultAuthService::new(db);
    let grpc_auth = GrpcAuthService::new(auth);

    Server::builder()
        .add_service(AuthServer::new(grpc_auth))
        .serve(addr)
        .await?;

    Ok(())
}
