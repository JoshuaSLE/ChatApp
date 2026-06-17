use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use sqlx::query_as;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    errors::AppError,
    models::users::{LoginResponse, LoginUser, RegisterUser, UserResponse},
    tokens::generate_tokens,
    utils::{password_hash, password_verify},
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

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginUser>,
) -> Result<Response, AppError> {
    payload.validate()?;

    let user = sqlx::query!(
        r#"
        SELECT id, password FROM users 
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

    let token_response = generate_tokens(user.id, &state.jwt_encoding_key)?;

    let refresh_token_hash = password_hash(token_response.refresh_token.clone()).await?;
    let refresh_expiry = OffsetDateTime::now_utc() + Duration::days(7);
    sqlx::query!(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::now_v7(),
        user.id,
        refresh_token_hash,
        refresh_expiry
    )
    .execute(&state.pool)
    .await?;

    let cookie = Cookie::build(("refresh_token", token_response.refresh_token))
        .path("/auth/refresh")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::days(7))
        .build();

    let mut response = (
        StatusCode::OK,
        Json(LoginResponse {
            access_token: token_response.access_token,
        }),
    )
        .into_response();

    response.headers_mut().insert(
        axum::http::header::SET_COOKIE,
        cookie.to_string().parse().unwrap(),
    );

    Ok(response)
}
