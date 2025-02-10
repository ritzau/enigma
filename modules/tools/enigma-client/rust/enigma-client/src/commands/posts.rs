use clap::Subcommand;
use enigma_auth::UserId;
use enigma_profiles::PostId;
use enigma_profiles_client::EnigmaProfilesClient;
use std::error::Error;

#[derive(Debug, Subcommand)]
pub enum PostCommands {
    Create { user_id: i64, content: String },
    Delete { post_id: String },
    Feed { user_id: i64 },
    List { user_id: i64 },
}

pub async fn run_command(
    command: PostCommands,
    profiles_client: &mut dyn EnigmaProfilesClient,
) -> Result<(), Box<dyn Error>> {
    match command {
        PostCommands::Create { user_id, content } => {
            let post_id = profiles_client
                .create_post(&UserId::from(user_id), &content)
                .await?;
            println!("Post created with id: {}", post_id);
        }
        PostCommands::Delete { post_id } => {
            profiles_client.delete_post(&PostId::from(post_id)).await?;
        }
        PostCommands::Feed { user_id } => {
            let posts = profiles_client
                .list_feed_posts(&UserId::from(user_id))
                .await?;
            for post in posts {
                println!("{}", post);
            }
        }
        PostCommands::List { user_id } => {
            let posts = profiles_client
                .list_profile_posts(&UserId::from(user_id))
                .await?;
            for post in posts {
                println!("{}", post);
            }
        }
    }
    Ok(())
}
