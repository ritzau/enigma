use clap::{Parser, Subcommand};
use enigma_auth_client::grpc::GrpcAuthClient;
use enigma_auth_client::EnigmaAuthClient;
use std::error::Error;
use tonic::transport::Channel;
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
    Login {
        username: String,
        password: String,
    },
}

#[derive(Debug, Subcommand)]
enum AuthCommands {
    AddRole {
        user_id: i64,
        role: String,
    },
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
    GetUserInfo {
        user_id: i64,
    },
    RefreshSession {
        refresh_token: String,
    },
    RemoveRole {
        user_id: i64,
        role: String,
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

    run_command(cli, &mut GrpcAuthClient::<Channel>::default().await?).await?;

    Ok(())
}

#[instrument(err, skip(cli, client))]
async fn run_command(cli: Cli, client: &mut dyn EnigmaAuthClient) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Commands::Auth(command) => match command {
            AuthCommands::AddRole { user_id, role } => {
                client.add_role(user_id.into(), &role).await?;
            }
            AuthCommands::Create { username, password } => {
                let response = client.create_account(&username, &password).await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::Delete { user_id } => {
                client.delete_account(&user_id.into()).await?;
            }
            AuthCommands::ChangePassword {
                user_id,
                old_password,
                new_password,
            } => {
                client
                    .change_password(user_id, &old_password, &new_password)
                    .await?;
            }
            AuthCommands::GetSession { access_token } => {
                let response = client.get_session(&Uuid::parse_str(&access_token)?).await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::GetUserInfo { user_id } => {
                let response = client.get_user_info(user_id.into()).await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::RefreshSession { refresh_token } => {
                let response = client
                    .refresh_session(&Uuid::parse_str(&refresh_token)?)
                    .await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::RemoveRole { user_id, role } => {
                client.remove_role(user_id.into(), &role).await?;
            }
            AuthCommands::List => {
                let response = client.list_accounts().await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::Login { username, password } => {
                let response = client.login(&username, &password).await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::PurgeSessions => {
                let response = client.purge_expired_sessions().await?;
                println!("RESPONSE={:?}", response);
            }
            AuthCommands::Samples => {
                create_test_users(client).await?;
            }
        },
        Commands::Login { username, password } => {
            let response = client.login(&username, &password).await?;
            println!("RESPONSE={:?}", response);
        }
    }
    Ok(())
}

async fn create_test_users(client: &mut dyn EnigmaAuthClient) -> Result<(), Box<dyn Error>> {
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

#[cfg(test)]
mod test {
    use super::*;
    use enigma_auth::{AccessToken, RefreshToken, UserId};
    use enigma_auth_client::MockEnigmaAuthClient;
    use mockall::predicate::*;
    use std::future::Future;
    use std::pin::Pin;

    type BoxedFuture<T> = Pin<Box<dyn Future<Output = Result<T, Box<dyn Error>>> + Send>>;

    fn mock_response<T: Send + 'static>(result: T) -> BoxedFuture<T> {
        Box::pin(async move { Ok(result) })
    }

    #[tokio::test]
    async fn test_run_command_create() {
        let mut mock = MockEnigmaAuthClient::new();
        mock.expect_create_account()
            .with(eq("test_user"), eq("test_password"))
            .times(1)
            .returning(|_, _| mock_response(UserId::from(1)));

        let cli = Cli {
            command: Commands::Auth(AuthCommands::Create {
                username: "test_user".to_string(),
                password: "test_password".to_string(),
            }),
        };

        let result = run_command(cli, &mut mock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_delete() {
        let mut mock = MockEnigmaAuthClient::new();
        mock.expect_delete_account()
            .with(eq(UserId::from(1)))
            .times(1)
            .returning(|_| Box::pin(async { Ok(()) }) as BoxedFuture<()>);

        let cli = Cli {
            command: Commands::Auth(AuthCommands::Delete { user_id: 1 }),
        };

        let result = run_command(cli, &mut mock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_change_password() {
        let mut mock = MockEnigmaAuthClient::new();
        mock.expect_change_password()
            .with(eq(1), eq("old_password"), eq("new_password"))
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok(()) }) as BoxedFuture<()>);

        let cli = Cli {
            command: Commands::Auth(AuthCommands::ChangePassword {
                user_id: 1,
                old_password: "old_password".to_string(),
                new_password: "new_password".to_string(),
            }),
        };

        let result = run_command(cli, &mut mock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_get_session() {
        let mut mock = MockEnigmaAuthClient::new();
        mock.expect_get_session()
            .with(eq(
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
            ))
            .times(1)
            .returning(|_| mock_response(UserId::from(0)));

        let cli = Cli {
            command: Commands::Auth(AuthCommands::GetSession {
                access_token: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            }),
        };

        let result = run_command(cli, &mut mock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_refresh_session() {
        let mut mock = MockEnigmaAuthClient::new();
        mock.expect_refresh_session()
            .with(eq(
                Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
            ))
            .times(1)
            .returning(|_| {
                mock_response((
                    AccessToken::from(Uuid::default()),
                    RefreshToken::from(Uuid::default()),
                ))
            });

        let cli = Cli {
            command: Commands::Auth(AuthCommands::RefreshSession {
                refresh_token: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            }),
        };

        let result = run_command(cli, &mut mock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_list() {
        let mut mock = MockEnigmaAuthClient::new();
        mock.expect_list_accounts()
            .times(1)
            .returning(|| mock_response(vec![(UserId::from(0), "user2".to_string())]));

        let cli = Cli {
            command: Commands::Auth(AuthCommands::List),
        };

        let result = run_command(cli, &mut mock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_login() {
        let mut mock = MockEnigmaAuthClient::new();
        mock.expect_login()
            .with(eq("test_user"), eq("test_password"))
            .times(1)
            .returning(|_, _| {
                mock_response((
                    AccessToken::from(Uuid::default()),
                    RefreshToken::from(Uuid::default()),
                ))
            });

        let cli = Cli {
            command: Commands::Auth(AuthCommands::Login {
                username: "test_user".to_string(),
                password: "test_password".to_string(),
            }),
        };

        let result = run_command(cli, &mut mock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_purge_sessions() {
        let mut mock = MockEnigmaAuthClient::new();
        mock.expect_purge_expired_sessions()
            .times(1)
            .returning(|| mock_response(0));

        let cli = Cli {
            command: Commands::Auth(AuthCommands::PurgeSessions),
        };

        let result = run_command(cli, &mut mock).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_command_samples() {
        let mut mock = MockEnigmaAuthClient::new();
        mock.expect_create_account()
            .times(8)
            .returning(|_, _| mock_response(UserId::from(1)));

        let cli = Cli {
            command: Commands::Auth(AuthCommands::Samples),
        };

        let result = run_command(cli, &mut mock).await;
        assert!(result.is_ok());
    }
}
