use axum::{Json, extract::State, http::StatusCode};
use sqlx::{query, query_as};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    errors::AppError,
    models::users::{LoginUser, RegisterUser, UserResponse},
    utils::{password_hash, password_verify},
};

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUser>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    payload.validate()?;

    let id = Uuid::now_v7();

    let password = password_hash(payload.password).await?;

    let row = query_as!(
        UserResponse,
        r#"
        INSERT INTO users (id, username, password, bio, avatar_key)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING username
        "#,
        id,
        payload.username,
        password,
        payload.bio,
        payload.avatar
    )
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::CREATED, Json(row)))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginUser>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    payload.validate()?;

    let user = query!(
        r#"
        SELECT password FROM users
        WHERE username = $1
        "#,
        payload.username
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let is_valid = password_verify(payload.password, user.password).await?;

    if !is_valid {
        return Err(AppError::Unauthorized);
    }

    Ok((
        StatusCode::OK,
        Json(UserResponse {
            username: payload.username,
        }),
    ))
}
