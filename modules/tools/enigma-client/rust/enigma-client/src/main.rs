use clap::{Parser, Subcommand};
use enigma_auth::EnigmaAuthClient;
use enigma_auth_client::grpc::GrpcAuthClient;
use std::error::Error;
use tracing::instrument;
use tracing_subscriber::fmt::format::FmtSpan;
use uuid::Uuid;

#[derive(Parser)]
#[command()]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(subcommand)]
    Auth(AuthCommands),
}

#[derive(Debug, Subcommand)]
enum AuthCommands {
    Create {
        username: String,
        password: String,
    },
    Delete {
        user_id: i64,
    },
    ChangePassword {
        user_id: i64,
        old_password: String,
        new_password: String,
    },
    GetSession {
        access_token: String,
    },
    RefreshSession {
        refresh_token: String,
    },
    List,
    Login {
        username: String,
        password: String,
    },
    PurgeSessions,
    Samples,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .init();

    let cli = Cli::parse();

    run_command(cli).await?;

    Ok(())
}

#[instrument(err, skip(cli))]
async fn run_command(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Commands::Auth(command) => match command {
            AuthCommands::Create { username, password } => {
                let mut client = GrpcAuthClient::default().await?;
                let response = client.create_account(&username, &password).await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::Delete { user_id } => {
                let mut client = GrpcAuthClient::default().await?;
                client.delete_account(&user_id.into()).await?;
            }
            AuthCommands::ChangePassword {
                user_id,
                old_password,
                new_password,
            } => {
                let mut client = GrpcAuthClient::default().await?;
                client
                    .change_password(user_id, &old_password, &new_password)
                    .await?;
            }
            AuthCommands::GetSession { access_token } => {
                let mut client = GrpcAuthClient::default().await?;
                let response = client.get_session(&Uuid::parse_str(&access_token)?).await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::RefreshSession { refresh_token } => {
                let mut client = GrpcAuthClient::default().await?;
                let response = client
                    .refresh_session(&Uuid::parse_str(&refresh_token)?)
                    .await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::List => {
                let mut client = GrpcAuthClient::default().await?;
                let response = client.list_accounts().await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::Login { username, password } => {
                let mut client = GrpcAuthClient::default().await?;
                let response = client.login(&username, &password).await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::PurgeSessions => {
                let mut client = GrpcAuthClient::default().await?;
                let response = client.purge_expired_sessions().await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::Samples => {
                let mut client = GrpcAuthClient::default().await?;
                create_test_users(&mut client).await?;
            }
        },
    }
    Ok(())
}

async fn create_test_users(client: &mut GrpcAuthClient) -> Result<(), Box<dyn std::error::Error>> {
    let test_users = vec![
        ("user1", "password1"),
        ("user2", "password2"),
        ("user3", "password3"),
        ("user4", "password4"),
        ("user5", "password5"),
        ("user6", "password6"),
        ("user7", "password7"),
        ("user8", "password8"),
    ];

    for (username, password) in test_users {
        let response = client.create_account(username, password).await?;
        println!(
            "Created user: {} with response: {:?}",
            username,
            response.value()
        );
    }

    Ok(())
}
