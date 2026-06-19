use axum::{Json, extract::State, http::StatusCode};
use sqlx::{query, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    errors::AppError,
    models::{
        rooms::{CreateRoom, CreateRoomResponse},
        tokens::Claims,
    },
};

pub async fn create(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateRoom>,
) -> Result<(StatusCode, Json<CreateRoomResponse>), AppError> {
    payload.validate()?;
    let creator_id = claims.sub;

    let mut member_ids: Vec<Uuid> = query!(
        r#"
        SELECT id FROM users 
        WHERE username = ANY($1)
        "#,
        &payload.members[..]
    )
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|row| row.id)
    .collect();

    if member_ids.len() != payload.members.len() {
        return Err(AppError::UserNotFound);
    }

    if payload.direct {
        if member_ids.len() != 1 {
            return Err(AppError::BadRequest(
                "Direct messages must have exactly one recipient".into(),
            ));
        }
        if member_ids[0] == creator_id {
            return Err(AppError::BadRequest(
                "You cannot start a direct message with yourself".into(),
            ));
        }

        let dm_participants = vec![creator_id, member_ids[0]];
        let existing_room = query_as!(
            CreateRoomResponse,
            r#"
            SELECT room_id FROM room_members
            WHERE room_id IN (SELECT id FROM rooms WHERE is_direct = true)
            GROUP BY room_id
            HAVING COUNT(user_id) = 2 AND bool_and(user_id = ANY($1::uuid[]))
            "#,
            &dm_participants as &[Uuid]
        )
        .fetch_optional(&state.pool)
        .await?;

        if let Some(room) = existing_room {
            return Ok((StatusCode::OK, Json(room)));
        }
    }

    member_ids.push(creator_id);
    member_ids.sort();
    member_ids.dedup();

    let room = CreateRoomResponse {
        room_id: Uuid::now_v7(),
    };
    let mut tx = state.pool.begin().await?;

    query!(
        r#"
        INSERT INTO rooms (id, name, is_direct, created_by) 
        VALUES ($1, $2, $3, $4)
        "#,
        room.room_id,
        payload.name,
        payload.direct,
        creator_id
    )
    .execute(&mut *tx)
    .await?;

    query!(
        r#"
        INSERT INTO room_members (room_id, user_id)
        SELECT $1, * FROM UNNEST($2::uuid[])
        ON CONFLICT DO NOTHING
        "#,
        room.room_id,
        &member_ids as &[Uuid]
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(room)))
}
