use axum::{Json, extract::State, http::StatusCode};
use sqlx::query_as;
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    errors::AppError,
    models::users::{RegisterUser, UserResponse},
    utils::password_hash,
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
