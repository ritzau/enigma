use foorum_auth_grpc::auth_server::AuthServer;
use foorum_auth_service::db::postgres::PostgresAuthDatabase;
use foorum_auth_service::grpc::GrpcAuthService;
use foorum_auth_service::DefaultAuthService;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::]:50051".parse().unwrap();
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
