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
            profiles_client
                .request_connection(&user_id.into(), &peer_id.into(), &relationship)
                .await?;
        }
        ConnectionCommands::Accept {
            user_id,
            peer_id,
            relationship,
        } => {
            profiles_client
                .accept_connection(&user_id.into(), &peer_id.into(), &relationship)
                .await?;
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
        } => {
            let connections = profiles_client.list_connections(&user_id.into()).await?;
            println!(
                "Connections for user {} with status {:?}",
                user_id, relationship
            );
            for (relationship, profile) in connections {
                println!("{}: {:?}", relationship, profile);
            }
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
            profiles_client
                .update_connection(&user_id.into(), &peer_id.into(), &relationship)
                .await?;
        }
    }
    Ok(())
}
