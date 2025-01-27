use clap::{Args, Subcommand, ValueEnum};
use enigma_profiles_client::EnigmaProfilesClient;
use std::error::Error;

#[derive(Debug, Subcommand)]
pub enum ConnectionCommands {
    Request {
        user_id: i64,
        connection_id: i64,
        kind: String,
    },
    Accept {
        user_id: i64,
        connection_id: i64,
        kind: String,
    },
    Reject {
        user_id: i64,
        connection_id: i64,
    },
    Create {
        user_id: i64,
        connection_id: i64,
        kind: String,
    },
    List {
        #[command(flatten)]
        kind: Option<ConnectionKind>,
        user_id: i64,
    },
    Remove {
        user_id: i64,
        connection_id: i64,
    },
    Update {
        user_id: i64,
        connection_id: i64,
        kind: String,
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
            connection_id,
            kind,
        } => {
            profiles_client
                .request_connection(&user_id.into(), &connection_id.into(), &kind)
                .await?;
        }
        ConnectionCommands::Accept {
            user_id,
            connection_id,
            kind,
        } => {
            profiles_client
                .accept_connection(&user_id.into(), &connection_id.into(), &kind)
                .await?;
        }
        ConnectionCommands::Reject {
            user_id,
            connection_id,
        } => {
            profiles_client
                .reject_connection(&user_id.into(), &connection_id.into())
                .await?;
        }
        ConnectionCommands::Create {
            user_id,
            connection_id,
            kind,
        } => {
            profiles_client
                .add_connection(&user_id.into(), &connection_id.into(), &kind)
                .await?;
        }
        ConnectionCommands::List { kind, user_id } => {
            let connections = profiles_client.get_connections(&user_id.into()).await?;
            println!("Connections for user {} with status {:?}", user_id, kind);
            for (kind, profile) in connections {
                println!("{}: {:?}", kind, profile);
            }
        }
        ConnectionCommands::Remove {
            user_id,
            connection_id,
        } => {
            profiles_client
                .remove_connection(&user_id.into(), &connection_id.into())
                .await?;
        }
        ConnectionCommands::Update {
            user_id,
            connection_id,
            kind,
        } => {
            profiles_client
                .update_connection(&user_id.into(), &connection_id.into(), &kind)
                .await?;
        }
    }
    Ok(())
}
