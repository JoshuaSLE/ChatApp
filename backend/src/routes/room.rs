use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::{query, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState, errors::AppError, models::{
        rooms::{CreateRoom, CreateRoomResponse, ListRoomResponse, MeRoomResponse, UpdateRoom, UpdateRoomResponse}, tokens::Claims,
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

    if member_ids.len() < 2 {
        return Err(AppError::BadRequest(
            "You cannot start a group message with yourself, add at least one more member".into(),
        ));
    }

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

pub async fn list(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<ListRoomResponse>>, AppError> {
    let user_id = claims.sub;

    let rooms = query_as!(
        ListRoomResponse,
        r#"
        SELECT
            r.id AS room_id,
            COALESCE(r.name, u.username) AS "room_name!",
            r.is_direct AS "is_direct!"
        FROM rooms r
        JOIN room_members rm ON r.id = rm.room_id
        LEFT JOIN room_members other_rm
            ON r.id = other_rm.room_id AND other_rm.user_id != $1 AND r.is_direct = true
        LEFT JOIN users u
            ON other_rm.user_id = u.id
        WHERE rm.user_id = $1
        ORDER BY r.created_at DESC
        "#,
        user_id
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(rooms))
}

pub async fn delete(
    State(state): State<AppState>,
    claims: Claims,
    Path(room_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let user_id = claims.sub;

    let row = query!(
        r#"
        DELETE FROM rooms
        WHERE id = $1 AND created_by = $2
        RETURNING id
        "#,
        room_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?;

    if row.is_none() {
        return Err(AppError::BadRequest(
            "Thread not found or you do not have permission to delete it".into(),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn me(
    State(state): State<AppState>,
    claims: Claims,
    Path(room_id): Path<Uuid>,
) -> Result<(StatusCode, Json<MeRoomResponse>), AppError> {
    let user_id = claims.sub;

    let room_data = query!(
        r#"
        SELECT 
            r.name,
            r.is_direct,
            EXISTS (
                SELECT 1 FROM room_members 
                WHERE room_id = $1 AND user_id = $2
            ) AS "is_member!",
            ARRAY_AGG(u.username) AS "members!"
        FROM rooms r
        JOIN room_members rm ON r.id = rm.room_id
        JOIN users u ON rm.user_id = u.id
        WHERE r.id = $1
        GROUP BY r.id, r.name, r.is_direct
        "#,
        room_id,
        user_id
    )
    .fetch_optional(&state.pool)
    .await?;

    let Some(room) = room_data else {
        return Err(AppError::RoomNotFound);
    };

    if !room.is_member {
        return Err(AppError::BadRequest(
            "You do not have permission to view this room".into(),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(MeRoomResponse {
            name: room.name,
            is_direct: room.is_direct.unwrap_or(false),
            members: room.members,
        }),
    ))
}

pub async fn update(
    State(state): State<AppState>,
    claims: Claims,
    Path(room_id): Path<Uuid>,
    Json(payload): Json<UpdateRoom>,
) -> Result<(StatusCode, Json<UpdateRoomResponse>), AppError> {
    payload.validate()?;
    let creator_id = claims.sub;

    let room_meta = query!(
        r#"
        SELECT created_by, is_direct FROM rooms
        WHERE id = $1
        "#,
        room_id
    )
    .fetch_optional(&state.pool)
    .await?;

    let Some(room) = room_meta else {
        return Err(AppError::RoomNotFound);
    };

    if room.created_by != Some(creator_id) {
        return Err(AppError::BadRequest(
            "You do not have permission to manage this room".into(),
        ));
    }

    if room.is_direct.unwrap_or(false) && !payload.members.is_empty() {
        return Err(AppError::BadRequest(
            "You cannot alter the members of a Direct Message thread".into(),
        ));
    }

    let mut member_ids: Vec<Uuid> = query!(
        "SELECT id FROM users WHERE username = ANY($1)",
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

    member_ids.push(creator_id);
    member_ids.sort();
    member_ids.dedup();

    if member_ids.len() < 2 {
        return Err(AppError::BadRequest(
            "Groups must have at least 2 distinct members".into(),
        ));
    }

    let mut tx = state.pool.begin().await?;

    if let Some(ref new_name) = payload.name {
        query!(
            r#"
            UPDATE rooms SET name = $1
            WHERE id = $2
            "#,
            new_name,
            room_id
        )
        .execute(&mut *tx)
        .await?;
    }

    query!(
        r#"
        DELETE FROM room_members
        WHERE room_id = $1 AND NOT (user_id = ANY($2::uuid[]))
        "#,
        room_id,
        &member_ids as &[Uuid]
    )
    .execute(&mut *tx)
    .await?;

    query!(
        r#"
        INSERT INTO room_members (room_id, user_id)
        SELECT $1, * FROM UNNEST($2::uuid[])
        ON CONFLICT DO NOTHING
        "#,
        room_id,
        &member_ids as &[Uuid]
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok((
        StatusCode::OK,
        Json(UpdateRoomResponse {
            name: payload.name,
            members: payload.members,
        }),
    ))
}
