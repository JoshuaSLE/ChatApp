use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use sqlx::{query, query_as};
use uuid::Uuid;

use crate::{
    AppState,
    errors::AppError,
    models::{
        messages::{CreateMessage, GetMessage, GetMessageResponse, MessageResponse},
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

pub async fn get(
    State(state): State<AppState>,
    claims: Claims,
    Path(room_id): Path<Uuid>,
    Query(params): Query<GetMessage>,
) -> Result<(StatusCode, Json<Vec<GetMessageResponse>>), AppError> {
    let user_id = claims.sub;
    let limit = params.limit.unwrap_or(50).min(100);

    let messages = query_as!(
        GetMessageResponse,
        r#"
        SELECT
            m.id,
            u.username AS "username!",
            m.body,
            m.attachment_key,
            m.attachment_type,
            m.created_at AS "created_at!"
        FROM messages m
        JOIN users u ON m.user_id = u.id
        WHERE m.room_id = $1
            AND EXISTS (
                SELECT 1 FROM room_members
                WHERE room_id = $1 AND user_id = $2
            )
            AND ($3::timestamptz IS NULL OR m.created_at < $3)
        ORDER BY m.created_at DESC, m.id DESC
        LIMIT $4
        "#,
        room_id,
        user_id,
        params.before,
        limit
    )
    .fetch_all(&state.pool)
    .await?;

    Ok((StatusCode::OK, Json(messages)))
}
