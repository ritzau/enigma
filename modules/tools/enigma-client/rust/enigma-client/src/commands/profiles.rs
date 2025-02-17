use chrono::NaiveDate;
use clap::Subcommand;
use enigma_profiles::EnigmaUserProfile;
use enigma_profiles_client::EnigmaProfilesClient;
use std::error::Error;

#[derive(Debug, Subcommand)]
pub enum ProfilesCommands {
    Create {
        user_id: i64,
        legal_name: String,
        display_name: String,
        profile_picture_url: String,
        primary_email: String,
        date_of_birth: String,
    },
    Delete {
        user_id: i64,
    },
    Get {
        user_id: i64,
    },
    Search {
        query: String,
    },
    Update {
        user_id: i64,
        legal_name: String,
        display_name: String,
        profile_picture_url: String,
        primary_email: String,
        date_of_birth: String,
    },
}

pub async fn run_command(
    profiles_command: ProfilesCommands,
    profiles_client: &mut dyn EnigmaProfilesClient,
) -> Result<(), Box<dyn Error>> {
    match profiles_command {
        ProfilesCommands::Create {
            user_id,
            legal_name,
            display_name,
            profile_picture_url,
            primary_email,
            date_of_birth,
        } => {
            let date_of_birth = NaiveDate::parse_from_str(&date_of_birth, "%Y-%m-%d")?;
            let profile = EnigmaUserProfile {
                user_id: user_id.into(),
                legal_name,
                display_name,
                profile_picture_url,
                primary_email,
                date_of_birth,
            };

            profiles_client.create_profile(&profile).await?;
        }
        ProfilesCommands::Delete { user_id } => {
            profiles_client.delete_profile(&user_id.into()).await?;
            println!("Profile {} deleted", user_id);
        }
        ProfilesCommands::Get { user_id } => {
            let profile = profiles_client.get_profile(&user_id.into()).await?;
            println!("{:?}", profile);
        }
        ProfilesCommands::Search { query } => {
            let profiles = profiles_client.search_profiles(&query).await?;
            for profile in profiles {
                println!(
                    "[{}] {} ({})",
                    profile.user_id.value(),
                    profile.display_name,
                    profile.legal_name
                );
            }
        }
        ProfilesCommands::Update {
            user_id,
            legal_name,
            display_name,
            profile_picture_url,
            primary_email,
            date_of_birth,
        } => {
            let date_of_birth = NaiveDate::parse_from_str(&date_of_birth, "%Y-%m-%d")?;
            let profile = EnigmaUserProfile {
                user_id: user_id.into(),
                legal_name,
                display_name,
                profile_picture_url,
                primary_email,
                date_of_birth,
            };

            profiles_client.update_profile(&profile).await?;
        }
    }
    Ok(())
}
