use axum::{Json, extract::State, http::StatusCode};
use sqlx::query;
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    errors::AppError,
    models::{rooms::CreateRoom, tokens::Claims},
};

// TODO: Fix recreation of the same room with the same members 
pub async fn create(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateRoom>,
) -> Result<StatusCode, AppError> {
    payload.validate()?;

    let creator_id = claims.sub;

    let mut member_ids = Vec::new();
    for username in &payload.members {
        let user = query!("SELECT id FROM users WHERE username = $1", username)
            .fetch_optional(&state.pool)
            .await?;

        match user {
            Some(row) => member_ids.push(row.id),
            None => return Err(AppError::UserNotFound),
        }
    }

    if payload.direct {
        if member_ids.len() != 1 {
            return Err(AppError::BadRequest(String::from(
                "Direct messages must have exactly one recipient",
            )));
        }
        if member_ids[0] == creator_id {
            return Err(AppError::BadRequest(String::from(
                "You cannot start a direct message with yourself",
            )));
        }
    }

    member_ids.push(creator_id);
    member_ids.sort();
    member_ids.dedup();

    let room_id = Uuid::now_v7();

    let mut tx = state.pool.begin().await?;

    query!(
        r#"
        INSERT INTO rooms (id, name, is_direct, created_by)
        VALUES ($1, $2, $3, $4)
        "#,
        room_id,
        payload.name,
        payload.direct,
        creator_id
    )
    .execute(&mut *tx)
    .await?;

    for user_id in member_ids {
        query!(
            r#"
            INSERT INTO room_members (user_id, room_id)
            VALUES ($1, $2)
            ON CONFLICT DO NOTHING
            "#,
            user_id,
            room_id
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(StatusCode::CREATED)
}
