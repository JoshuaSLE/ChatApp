use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::utils::trimmed_option;

#[derive(Debug, Validate, Deserialize)]
pub struct CreateRoom {
    #[serde(default, deserialize_with = "trimmed_option")]
    #[validate(length(min = 3, max = 30, message = "Name must be 3-30 characters"))]
    pub name: Option<String>,
    pub direct: bool,
    pub members: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateRoomResponse {
    pub room_id: Uuid,
}
