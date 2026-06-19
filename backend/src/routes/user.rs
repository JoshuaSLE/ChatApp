use axum::{
    Json,
    extract::{Query, State},
    http::{StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Response},
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use sqlx::{query, query_as};
use time::Duration;
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    errors::AppError,
    models::{
        tokens::Claims,
        users::{
            MeUserResponse, RegisterUser, SearchUser, SearchUserResponse, StatusUser,
            StatusUserResponse, UpdateUser, UpdateUserResponse, UserResponse,
        },
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

pub async fn delete(State(state): State<AppState>, claims: Claims) -> Result<Response, AppError> {
    let user_id = claims.sub;

    query!("DELETE FROM users WHERE id = $1", user_id)
        .execute(&state.pool)
        .await?;

    let removal_cookie = Cookie::build(("refresh_token", ""))
        .path("/auth")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::ZERO)
        .build();

    let mut response = StatusCode::NO_CONTENT.into_response();

    let cookie_header = removal_cookie
        .to_string()
        .parse()
        .map_err(|_| AppError::Internal)?;
    response.headers_mut().insert(SET_COOKIE, cookie_header);

    Ok(response)
}

pub async fn me(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<(StatusCode, Json<MeUserResponse>), AppError> {
    let user_id = claims.sub;

    let user = query_as!(
        MeUserResponse,
        r#"
        SELECT username, bio, created_at, last_seen, online, avatar_key FROM users
        WHERE id = $1
        "#,
        user_id
    )
    .fetch_one(&state.pool)
    .await?;

    Ok((StatusCode::OK, Json(user)))
}

pub async fn search(
    State(state): State<AppState>,
    claims: Claims,
    Query(params): Query<SearchUser>,
) -> Result<(StatusCode, Json<Vec<SearchUserResponse>>), AppError> {
    params.validate()?;

    let user_id = claims.sub;

    let rows = query_as!(
        SearchUserResponse,
        r#"
        SELECT username, bio FROM users
        WHERE id != $1 AND username % $2
        ORDER BY username <-> $2 ASC
        LIMIT 15
        "#,
        user_id,
        &params.username
    )
    .fetch_all(&state.pool)
    .await?;

    Ok((StatusCode::OK, Json(rows)))
}

pub async fn status(
    State(state): State<AppState>,
    claims: Claims,
    Query(params): Query<StatusUser>,
) -> Result<(StatusCode, Json<StatusUserResponse>), AppError> {
    params.validate()?;

    let user_id = claims.sub;

    let user_status = query_as!(
        StatusUserResponse,
        r#"
        SELECT online, last_seen
        FROM users
        WHERE id != $1 AND username = $2
        "#,
        user_id,
        params.username
    )
    .fetch_optional(&state.pool)
    .await?;

    let user = user_status.ok_or(AppError::UserNotFound)?;

    Ok((StatusCode::OK, Json(user)))
}
