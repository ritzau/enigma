use crate::commands::auth::AuthCommands;
use crate::commands::connections::ConnectionCommands;
use crate::commands::profiles::ProfilesCommands;
use crate::Cli;
use clap::Subcommand;
use enigma_auth_client::EnigmaAuthClient;
use enigma_profiles_client::EnigmaProfilesClient;
use std::error::Error;
use tracing::instrument;

pub(crate) mod auth;
pub(crate) mod connections;
pub(crate) mod profiles;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(subcommand)]
    Auth(AuthCommands),
    Login {
        username: String,
        password: String,
    },
    #[command(subcommand)]
    Connections(ConnectionCommands),
    #[command(subcommand)]
    Profiles(ProfilesCommands),
}

#[instrument(err, skip_all)]
pub async fn run_command(
    cli: Cli,
    auth_client: &mut dyn EnigmaAuthClient,
    profiles_client: &mut dyn EnigmaProfilesClient,
) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Commands::Auth(command) => auth::run_command(auth_client, command).await?,
        Commands::Connections(command) => {
            connections::run_command(command, profiles_client).await?
        }
        Commands::Login { username, password } => {
            let response = auth_client.login(&username, &password).await?;
            println!("RESPONSE={:?}", response);
        }
        Commands::Profiles(profiles_command) => {
            profiles::run_command(profiles_command, profiles_client).await?
        }
    }
    Ok(())
}
