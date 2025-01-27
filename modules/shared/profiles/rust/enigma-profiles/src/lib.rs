use chrono::NaiveDate;
use enigma_auth::UserId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnigmaUserProfile {
    pub user_id: UserId,
    pub legal_name: String,
    pub display_name: String,
    pub profile_picture_url: String,
    pub primary_email: String,
    pub date_of_birth: NaiveDate,
}
