use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::utils::trimmed_option;

#[derive(Debug, Validate, Deserialize)]
pub struct CreateMessage {
    #[serde(default, deserialize_with = "trimmed_option")]
    #[validate(length(max = 255, message = "body must be 0-30 characters"))]
    pub body: Option<String>,
    pub attachment_key: Option<String>,
    pub attachment_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub body: Option<String>,
    pub attachment_key: Option<String>,
    pub attachment_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetMessage {
    pub limit: Option<i64>,
    pub before: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize)]
pub struct GetMessageResponse {
    pub id: Uuid,
    pub username: String,
    pub body: Option<String>,
    pub attachment_key: Option<String>,
    pub attachment_type: Option<String>,
    pub created_at: OffsetDateTime,
}
