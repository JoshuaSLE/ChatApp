use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::query;
use uuid::Uuid;

use crate::{
    AppState,
    errors::AppError,
    models::{
        messages::{CreateMessage, MessageResponse},
        tokens::Claims,
    },
};

pub async fn create(
    State(state): State<AppState>,
    claims: Claims,
    Path(room_id): Path<Uuid>,
    Json(payload): Json<CreateMessage>,
) -> Result<(StatusCode, Json<MessageResponse>), AppError> {
    let user_id = claims.sub;
    let message_id = Uuid::now_v7();

    let result = query!(
        r#"
        INSERT INTO messages (id, room_id, user_id, body, attachment_key, attachment_type)
        SELECT $1, $2, $3, $4, $5, $6
        WHERE EXISTS (
            SELECT 1 FROM room_members
            WHERE user_id = $3 AND room_id = $2
        )
        "#,
        message_id,
        room_id,
        user_id,
        payload.body,
        payload.attachment_key,
        payload.attachment_type
    )
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::RoomNotFound);
    }

    Ok((
        StatusCode::CREATED,
        Json(MessageResponse {
            body: payload.body,
            attachment_key: payload.attachment_key,
            attachment_type: payload.attachment_type,
        }),
    ))
}
