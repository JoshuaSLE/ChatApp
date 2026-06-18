use axum::{Json, extract::State, http::StatusCode};
use sqlx::query_as;
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    errors::AppError,
    models::{
        tokens::Claims,
        users::{RegisterUser, UpdateUser, UpdateUserResponse, UserResponse},
    },
    utils::password_hash,
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUser>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    payload.validate()?;

    let password = password_hash(payload.password).await?;

    let row = query_as!(
        UserResponse,
        r#"
        INSERT INTO users (id, username, password, bio, avatar_key)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING username
        "#,
        Uuid::now_v7(),
        payload.username,
        password,
        payload.bio,
        payload.avatar
    )
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn update(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<UpdateUser>,
) -> Result<(StatusCode, Json<UpdateUserResponse>), AppError> {
    payload.validate()?;

    let user_id = claims.sub;

    let hashed_password = match payload.password {
        Some(password) => Some(password_hash(password).await?),
        None => None,
    };

    let user = query_as!(
        UpdateUserResponse,
        r#"
        UPDATE users
        SET
            username    = COALESCE($1, username),
            password    = COALESCE($2, password),
            bio         = COALESCE($3, bio),
            avatar_key  = COALESCE($4, avatar_key)
        WHERE id = $5
        RETURNING username, bio, avatar_key
        "#,
        payload.username,
        hashed_password,
        payload.bio,
        payload.avatar,
        user_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::OK, Json(user)))
}
