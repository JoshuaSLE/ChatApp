use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use validator::Validate;

use crate::utils::{trimmed_option, trimmed_string};

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub username: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct RegisterUser {
    #[serde(deserialize_with = "trimmed_string")]
    #[validate(length(min = 3, max = 30, message = "Username must be 3-30 characters"))]
    pub username: String,

    #[validate(length(min = 8, max = 30, message = "Password must be at least 8 characters"))]
    pub password: String,

    #[serde(default, deserialize_with = "trimmed_option")]
    #[validate(length(max = 255, message = "Bio cannot exceed 255 characters"))]
    pub bio: Option<String>,

    pub avatar: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct LoginUser {
    #[serde(deserialize_with = "trimmed_string")]
    #[validate(length(min = 3, max = 30, message = "Username must be 3-30 characters"))]
    pub username: String,

    #[validate(length(min = 8, max = 30, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginUserResponse {
    pub access_token: String,
}

#[derive(Debug, Validate, Deserialize)]
pub struct UpdateUser {
    #[serde(default, deserialize_with = "trimmed_option")]
    #[validate(length(min = 3, max = 30, message = "Username must be 3-30 characters"))]
    pub username: Option<String>,

    #[validate(length(min = 8, max = 30, message = "Password must be at least 8 characters"))]
    pub password: Option<String>,

    #[serde(default, deserialize_with = "trimmed_option")]
    #[validate(length(max = 255, message = "Bio cannot exceed 255 characters"))]
    pub bio: Option<String>,

    pub avatar: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpdateUserResponse {
    pub username: String,
    pub bio: Option<String>,
    pub avatar_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MeUserResponse {
    pub username: String,
    pub bio: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_seen: Option<OffsetDateTime>,
    pub online: bool,
    pub avatar_key: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct SearchUser {
    #[serde(deserialize_with = "trimmed_string")]
    #[validate(length(
        min = 3,
        max = 30,
        message = "Search query must be between 3 and 30 characters"
    ))]
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct SearchUserResponse {
    pub username: String,
    pub bio: Option<String>,
}

#[derive(Debug, Validate, Deserialize)]
pub struct StatusUser {
    #[serde(deserialize_with = "trimmed_string")]
    #[validate(length(min = 3, max = 30, message = "Username must be 3-30 characters"))]
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct StatusUserResponse {
    #[serde(default, with = "time::serde::rfc3339::option")]
    pub last_seen: Option<OffsetDateTime>,
    pub online: bool,
}
