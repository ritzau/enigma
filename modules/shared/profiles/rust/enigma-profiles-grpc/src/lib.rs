#![allow(clippy::match_single_binding)]

use chrono::{DateTime, Utc};
use enigma_profiles::{EnigmaConnection, EnigmaConnectionStatus, EnigmaUserProfile};
use prost_types::Timestamp;
use tonic::Status;

tonic::include_proto!("profiles");

////////////////////////////////////////
// Connection Status

impl From<EnigmaConnectionStatus> for ConnectionStatus {
    fn from(value: EnigmaConnectionStatus) -> Self {
        match value {
            EnigmaConnectionStatus::Unspecified => Self::Unspecified,
            EnigmaConnectionStatus::RequestSent => Self::RequestSent,
            EnigmaConnectionStatus::Requested => Self::Requested,
            EnigmaConnectionStatus::Connected => Self::Connected,
            EnigmaConnectionStatus::Deleted => Self::Deleted,
            EnigmaConnectionStatus::Denied => Self::Denied,
            EnigmaConnectionStatus::Follower => Self::Follower,
            EnigmaConnectionStatus::Followed => Self::Followed,
            EnigmaConnectionStatus::Ghost => Self::Ghost,
        }
    }
}

impl From<ConnectionStatus> for EnigmaConnectionStatus {
    fn from(value: ConnectionStatus) -> Self {
        match value {
            ConnectionStatus::Unspecified => Self::Unspecified,
            ConnectionStatus::RequestSent => Self::RequestSent,
            ConnectionStatus::Requested => Self::Requested,
            ConnectionStatus::Connected => Self::Connected,
            ConnectionStatus::Deleted => Self::Deleted,
            ConnectionStatus::Denied => Self::Denied,
            ConnectionStatus::Follower => Self::Follower,
            ConnectionStatus::Followed => Self::Followed,
            ConnectionStatus::Ghost => Self::Ghost,
        }
    }
}

////////////////////////////////////////
// Connection

impl TryFrom<EnigmaConnection> for Connection {
    type Error = Status;

    fn try_from(connection: EnigmaConnection) -> Result<Self, Self::Error> {
        Ok(Self {
            connection_id: connection.connection_id.to_string(),
            user_id: connection.user_id.into(),
            peer_id: connection.peer_id.into(),
            relationship: connection.relationship,
            status: ConnectionStatus::from(connection.status).into(),
            created_at: Some(chrono_to_protobuf_timestamp(connection.created_at)),
            updated_at: Some(chrono_to_protobuf_timestamp(connection.updated_at)),
            profile: Some(connection.profile.try_into()?),
        })
    }
}

impl TryFrom<Connection> for EnigmaConnection {
    type Error = Status;

    fn try_from(connection: Connection) -> Result<Self, Self::Error> {
        let status = ConnectionStatus::try_from(connection.status)
            .map_err(|_| Status::invalid_argument("Invalid status"))?;

        Ok(Self {
            connection_id: connection
                .connection_id
                .as_str()
                .try_into()
                .map_err(|_| Status::invalid_argument("Invalid UUID"))?,
            user_id: connection.user_id.into(),
            peer_id: connection.peer_id.into(),
            relationship: connection.relationship,
            status: status.into(),
            created_at: protobuf_to_chrono_timestamp(
                &connection
                    .created_at
                    .ok_or(Status::internal("Missing creation date"))?,
            )?,
            updated_at: protobuf_to_chrono_timestamp(
                &connection
                    .updated_at
                    .ok_or(Status::internal("Missing update date"))?,
            )?,
            update_seq: None,
            profile: EnigmaUserProfile::try_from(
                connection
                    .profile
                    .ok_or(Status::invalid_argument("Missing profile"))?,
            )?,
        })
    }
}

////////////////////////////////////////
// User profile

impl TryFrom<EnigmaUserProfile> for UserProfile {
    type Error = Status;

    fn try_from(profile: EnigmaUserProfile) -> Result<Self, Self::Error> {
        let date_of_birth = profile
            .date_of_birth
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| Status::invalid_argument("Invalid time stamp"))?
            .and_utc()
            .timestamp();

        Ok(Self {
            user_id: profile.user_id.value(),
            legal_name: profile.legal_name,
            display_name: profile.display_name,
            profile_picture_url: profile.profile_picture_url,
            primary_email: profile.primary_email,
            date_of_birth,
        })
    }
}

impl TryFrom<&EnigmaUserProfile> for UserProfile {
    type Error = Status;

    fn try_from(profile: &EnigmaUserProfile) -> Result<Self, Self::Error> {
        let date_of_birth = profile
            .date_of_birth
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| Status::invalid_argument("Invalid time stamp"))?
            .and_utc()
            .timestamp();

        Ok(Self {
            user_id: profile.user_id.value(),
            legal_name: profile.legal_name.clone(),
            display_name: profile.display_name.clone(),
            profile_picture_url: profile.profile_picture_url.clone(),
            primary_email: profile.primary_email.clone(),
            date_of_birth,
        })
    }
}

impl TryFrom<UserProfile> for EnigmaUserProfile {
    type Error = Status;

    fn try_from(profile: UserProfile) -> Result<Self, Self::Error> {
        Ok(EnigmaUserProfile {
            user_id: profile.user_id.into(),
            legal_name: profile.legal_name,
            display_name: profile.display_name,
            profile_picture_url: profile.profile_picture_url,
            primary_email: profile.primary_email,
            date_of_birth: DateTime::from_timestamp(profile.date_of_birth, 0)
                .ok_or(Status::invalid_argument("Invalid date of birth"))?
                .date_naive(),
        })
    }
}

impl TryFrom<&UserProfile> for EnigmaUserProfile {
    type Error = Status;

    fn try_from(profile: &UserProfile) -> Result<Self, Self::Error> {
        Ok(EnigmaUserProfile {
            user_id: profile.user_id.into(),
            legal_name: profile.legal_name.clone(),
            display_name: profile.display_name.clone(),
            profile_picture_url: profile.profile_picture_url.clone(),
            primary_email: profile.primary_email.clone(),
            date_of_birth: DateTime::from_timestamp(profile.date_of_birth, 0)
                .ok_or(Status::invalid_argument("Invalid date of birth"))?
                .date_naive(),
        })
    }
}

////////////////////////////////////////
// Post

impl TryFrom<enigma_profiles::EnigmaPost> for Post {
    type Error = Status;

    fn try_from(value: enigma_profiles::EnigmaPost) -> Result<Self, Self::Error> {
        Ok(Self {
            post_id: value.post_id.to_string(),
            created_at: value.created_at.timestamp(),
            updated_at: value.updated_at.timestamp(),
            user_id: value.user_id.into(),
            user_profile: Some(value.user_profile.try_into()?),
            content: value.content,
        })
    }
}

impl TryFrom<Post> for enigma_profiles::EnigmaPost {
    type Error = Status;

    fn try_from(value: Post) -> Result<Self, Self::Error> {
        Ok(Self {
            post_id: value.post_id.into(),
            created_at: DateTime::from_timestamp(value.created_at, 0)
                .ok_or(Status::invalid_argument("Invalid creation date"))?,
            updated_at: DateTime::from_timestamp(value.updated_at, 0)
                .ok_or(Status::invalid_argument("Invalid update date"))?,
            user_id: value.user_id.into(),
            user_profile: EnigmaUserProfile::try_from(
                value
                    .user_profile
                    .ok_or(Status::invalid_argument("Missing profile"))?,
            )?,
            content: value.content,
        })
    }
}

impl TryFrom<&Post> for enigma_profiles::EnigmaPost {
    type Error = Status;

    fn try_from(value: &Post) -> Result<Self, Self::Error> {
        Ok(Self {
            post_id: value.post_id.to_string().into(),
            created_at: DateTime::from_timestamp(value.created_at, 0)
                .ok_or(Status::invalid_argument("Invalid creation date"))?,
            updated_at: DateTime::from_timestamp(value.updated_at, 0)
                .ok_or(Status::invalid_argument("Invalid update date"))?,
            user_id: value.user_id.into(),
            user_profile: EnigmaUserProfile::try_from(
                value
                    .user_profile
                    .as_ref()
                    .ok_or(Status::invalid_argument("Missing profile"))?,
            )?,
            content: value.content.clone(),
        })
    }
}

fn chrono_to_protobuf_timestamp(dt: DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

fn protobuf_to_chrono_timestamp(ts: &Timestamp) -> Result<DateTime<Utc>, Status> {
    DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
        .ok_or(Status::invalid_argument("Invalid timestamp"))
}
