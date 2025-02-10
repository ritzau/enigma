#![allow(clippy::match_single_binding)]

use chrono::DateTime;
use enigma_profiles::EnigmaUserProfile;
use tonic::Status;

tonic::include_proto!("profiles");

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
