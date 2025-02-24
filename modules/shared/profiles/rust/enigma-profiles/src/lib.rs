use chrono::NaiveDate;
use enigma_auth::UserId;
use std::fmt::Display;
use uuid::Uuid;

////////////////////////////////////////////////////////////////////////////////
// EnigmaConnectionId

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnigmaConnectionId(Uuid);

impl From<Uuid> for EnigmaConnectionId {
    fn from(value: Uuid) -> Self {
        EnigmaConnectionId(value)
    }
}

impl From<&Uuid> for EnigmaConnectionId {
    fn from(value: &Uuid) -> Self {
        EnigmaConnectionId(*value)
    }
}

impl From<EnigmaConnectionId> for Uuid {
    fn from(value: EnigmaConnectionId) -> Self {
        value.0
    }
}

impl From<&EnigmaConnectionId> for Uuid {
    fn from(value: &EnigmaConnectionId) -> Self {
        value.0
    }
}

impl TryFrom<&str> for EnigmaConnectionId {
    type Error = uuid::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Uuid::parse_str(value).map(EnigmaConnectionId)
    }
}

impl TryFrom<String> for EnigmaConnectionId {
    type Error = uuid::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Uuid::parse_str(&value).map(EnigmaConnectionId)
    }
}

impl Display for EnigmaConnectionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

////////////////////////////////////////////////////////////////////////////////
// EnigmaUserProfile

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnigmaUserProfile {
    pub user_id: UserId,
    pub legal_name: String,
    pub display_name: String,
    pub profile_picture_url: String,
    pub primary_email: String,
    pub date_of_birth: NaiveDate,
}

////////////////////////////////////////////////////////////////////////////////
// PostId

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

////////////////////////////////////////////////////////////////////////////////
// EnigmaConnectionStatus

#[derive(Clone, Debug)]
pub enum EnigmaConnectionStatus {
    Unspecified,
    RequestSent,
    Requested,
    Connected,
    Deleted,
    Denied,
    Follower,
    Followed,
    Ghost,
}

impl Display for EnigmaConnectionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnigmaConnectionStatus::Unspecified => write!(f, "Unspecified"),
            EnigmaConnectionStatus::RequestSent => write!(f, "Request Sent"),
            EnigmaConnectionStatus::Requested => write!(f, "Requested"),
            EnigmaConnectionStatus::Connected => write!(f, "Connected"),
            EnigmaConnectionStatus::Deleted => write!(f, "Deleted"),
            EnigmaConnectionStatus::Denied => write!(f, "Denied"),
            EnigmaConnectionStatus::Follower => write!(f, "Follower"),
            EnigmaConnectionStatus::Followed => write!(f, "Followed"),
            EnigmaConnectionStatus::Ghost => write!(f, "Ghost"),
        }
    }
}

impl TryFrom<&str> for EnigmaConnectionStatus {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Unspecified" => Ok(EnigmaConnectionStatus::Unspecified),
            "Request Sent" | "request_sent" => Ok(EnigmaConnectionStatus::RequestSent),
            "Requested" => Ok(EnigmaConnectionStatus::Requested),
            "Connected" | "connected" => Ok(EnigmaConnectionStatus::Connected),
            "Deleted" | "deleted" => Ok(EnigmaConnectionStatus::Deleted),
            "Denied" => Ok(EnigmaConnectionStatus::Denied),
            "Follower" => Ok(EnigmaConnectionStatus::Follower),
            "Followed" => Ok(EnigmaConnectionStatus::Followed),
            "Ghost" => Ok(EnigmaConnectionStatus::Ghost),
            _ => Err(()),
        }
    }
}

impl TryFrom<String> for EnigmaConnectionStatus {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        EnigmaConnectionStatus::try_from(value.as_str())
    }
}

////////////////////////////////////////////////////////////////////////////////
// EnigmaConnection

#[derive(Clone, Debug)]
pub struct EnigmaConnection {
    pub connection_id: EnigmaConnectionId,
    pub user_id: UserId,
    pub peer_id: UserId,
    pub relationship: String,
    pub status: EnigmaConnectionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub update_seq: Option<i64>,
    pub profile: EnigmaUserProfile,
}

impl Display for EnigmaConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:8} {} {}",
            self.profile.display_name, self.relationship, self.status
        )
    }
}

////////////////////////////////////////////////////////////////////////////////
// EnigmaPost

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
