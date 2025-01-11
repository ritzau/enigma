use clap::{Parser, Subcommand};
use foorum_auth::UserId;
use foorum_auth_client_grpc::GrpcAuthClient;
use foorum_auth::FoorumAuthClient;
use uuid::Uuid;

#[derive(Parser)]
#[command()]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    Auth(AuthCommands),
}

#[derive(Subcommand)]
enum AuthCommands {
    Create { username: String, password: String },
    Delete { user_id: i64 },
    GetSession { access_token: String },
    List,
    Login { username: String, password: String },
    PurgeSessions,
    Samples,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth(command) => match command {
            AuthCommands::Create { username, password } => {
                let mut client = GrpcAuthClient::default().await?;
                let response = client.create_account(&username, &password).await?;
                println!("RESPONSE={:?}", response.0);
            }
            AuthCommands::Delete { user_id } => {
                let mut client = GrpcAuthClient::default().await?;
                let response = client.delete_account(&UserId(user_id)).await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::GetSession { access_token } => {
                let mut client = GrpcAuthClient::default().await?;
                let response = client.get_session(&Uuid::parse_str(&access_token)?).await?;
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
        println!("Created user: {} with response: {:?}", username, response.0);
    }

    Ok(())
}
