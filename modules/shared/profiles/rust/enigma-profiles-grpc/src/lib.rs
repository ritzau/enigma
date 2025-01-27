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
            user_id: profile.user_id.into(),
            legal_name: profile.legal_name,
            display_name: profile.display_name,
            profile_picture_url: profile.profile_picture_url,
            primary_email: profile.primary_email,
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
