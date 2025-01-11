use sqlx::types::Uuid;

pub mod client;
pub mod service;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UserId(pub i64);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UserName(pub String);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UserHash(pub String);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AccessToken(pub Uuid);
