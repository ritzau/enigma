use chrono::NaiveDate;
use enigma_auth::UserId;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnigmaUserProfile {
    pub user_id: UserId,
    pub legal_name: String,
    pub display_name: String,
    pub profile_picture_url: String,
    pub primary_email: String,
    pub date_of_birth: NaiveDate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostId(String);

impl From<String> for PostId {
    fn from(value: String) -> Self {
        PostId(value)
    }
}

impl From<&str> for PostId {
    fn from(value: &str) -> Self {
        PostId(value.to_string())
    }
}

impl From<Uuid> for PostId {
    fn from(value: Uuid) -> Self {
        PostId(value.to_string())
    }
}

impl TryFrom<PostId> for Uuid {
    type Error = uuid::Error;

    fn try_from(value: PostId) -> Result<Self, Self::Error> {
        Uuid::parse_str(&value.0)
    }
}

impl TryFrom<&PostId> for Uuid {
    type Error = uuid::Error;

    fn try_from(value: &PostId) -> Result<Self, Self::Error> {
        Uuid::parse_str(&value.0)
    }
}

impl Display for PostId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug)]
pub struct EnigmaPost {
    pub post_id: PostId,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub user_id: UserId,
    pub user_profile: EnigmaUserProfile,
    pub content: String,
}

impl Display for EnigmaPost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:8} {}: {}",
            self.user_profile.display_name,
            self.created_at.naive_local().date(),
            self.content
        )
    }
}
