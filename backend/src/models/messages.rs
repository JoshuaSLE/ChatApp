use serde::{Deserialize, Serialize};
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
