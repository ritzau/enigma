use clap::Parser;
use commands::Commands;
use enigma_auth_client::grpc::GrpcAuthClient;
use enigma_auth_grpc::auth_client::AuthClient;
use enigma_profiles_client::grpc::ProfilesGrpcClient;
use std::error::Error;
use tonic::transport::Channel;
use tracing_subscriber::fmt::format::FmtSpan;

mod commands;

#[derive(Parser)]
#[command()]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logging();

    let cli = Cli::parse();

    let url = "http://[::1]:50051";
    let channel = Channel::from_shared(url.to_string())?.connect().await?;
    let grpc_auth_client = AuthClient::new(channel);

    let mut auth_client = GrpcAuthClient::new(grpc_auth_client.clone());
    let mut profiles_client = ProfilesGrpcClient::localhost(grpc_auth_client).await?;
    commands::run_command(cli, &mut auth_client, &mut profiles_client).await?;

    Ok(())
}

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_writer(std::io::stderr)
        .init();
}
