use clap::{Args, Subcommand, ValueEnum};
use enigma_profiles_client::EnigmaProfilesClient;
use std::error::Error;

#[derive(Debug, Subcommand)]
pub enum ConnectionCommands {
    Request {
        user_id: i64,
        peer_id: i64,
        relationship: String,
    },
    Accept {
        user_id: i64,
        peer_id: i64,
        relationship: String,
    },
    Reject {
        user_id: i64,
        peer_id: i64,
    },
    Create {
        user_id: i64,
        peer_id: i64,
        relationship: String,
    },
    List {
        #[command(flatten)]
        relationship: Option<ConnectionKind>,
        user_id: i64,
        #[arg(short, long)]
        cursor: Option<String>,
        #[arg(short, long)]
        limit: Option<u16>,
    },
    Remove {
        user_id: i64,
        peer_id: i64,
    },
    Update {
        user_id: i64,
        peer_id: i64,
        relationship: String,
    },
}

#[derive(Args, Debug)]
#[group(required = false, multiple = false)]
pub struct ConnectionKind {
    #[arg(long)]
    active: bool,

    #[arg(long)]
    requests: bool,

    #[arg(long)]
    denied: bool,
}

#[derive(Debug, ValueEnum, Clone)]
pub enum ConnectionStatus {
    Active,
    Requests,
    Denied,
}

pub async fn run_command(
    profiles_command: ConnectionCommands,
    profiles_client: &mut dyn EnigmaProfilesClient,
) -> Result<(), Box<dyn Error>> {
    match profiles_command {
        ConnectionCommands::Request {
            user_id,
            peer_id,
            relationship,
        } => {
            let connection = profiles_client
                .request_connection(&user_id.into(), &peer_id.into(), &relationship)
                .await?;
            println!("Requested connection: {:?}", connection);
        }
        ConnectionCommands::Accept {
            user_id,
            peer_id,
            relationship,
        } => {
            let connection = profiles_client
                .accept_connection(&user_id.into(), &peer_id.into(), &relationship)
                .await?;
            println!("Accepted connection: {:?}", connection);
        }
        ConnectionCommands::Reject { user_id, peer_id } => {
            profiles_client
                .reject_connection(&user_id.into(), &peer_id.into())
                .await?;
        }
        ConnectionCommands::Create {
            user_id,
            peer_id,
            relationship,
        } => {
            profiles_client
                .add_connection(&user_id.into(), &peer_id.into(), &relationship)
                .await?;
        }
        ConnectionCommands::List {
            relationship,
            user_id,
            cursor,
            limit,
        } => {
            let (connections, next_cursor, has_more) = profiles_client
                .list_connections(&user_id.into(), cursor.as_deref(), limit)
                .await?;
            println!(
                "Connections for user {} with status {:?}",
                user_id, relationship
            );
            for connection in connections {
                println!(
                    "{}: {:?}",
                    connection.relationship, connection.profile.display_name
                );
            }
            println!("Next cursor: {} has_more: {}", next_cursor, has_more);
        }
        ConnectionCommands::Remove { user_id, peer_id } => {
            profiles_client
                .remove_connection(&user_id.into(), &peer_id.into())
                .await?;
        }
        ConnectionCommands::Update {
            user_id,
            peer_id,
            relationship,
        } => {
            let connection = profiles_client
                .update_connection(&user_id.into(), &peer_id.into(), &relationship)
                .await?;

            println!("Updated connection: {:?}", connection);
        }
    }
    Ok(())
}
