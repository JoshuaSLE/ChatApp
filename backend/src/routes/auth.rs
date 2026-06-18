use axum::{
    Json,
    extract::State,
    http::{StatusCode, header::SET_COOKIE},
    response::{IntoResponse, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use sqlx::query;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    errors::AppError,
    models::users::{LoginResponse, LoginUser},
    tokens::generate_tokens,
    utils::{password_hash, password_verify},
};

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginUser>,
) -> Result<Response, AppError> {
    payload.validate()?;

    let user = query!(
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

    let token_selector = Uuid::now_v7();

    query!(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        token_selector,
        user.id,
        refresh_token_hash,
        refresh_expiry
    )
    .execute(&state.pool)
    .await?;

    let cookie_value = format!("{}:{}", token_selector, token_response.refresh_token);
    let cookie = Cookie::build(("refresh_token", cookie_value))
        .path("/auth")
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

    let cookie_header = cookie.to_string().parse().map_err(|_| AppError::Internal)?;
    response.headers_mut().insert(SET_COOKIE, cookie_header);

    Ok(response)
}

pub async fn refresh(State(state): State<AppState>, jar: CookieJar) -> Result<Response, AppError> {
    let cookie_val = jar
        .get("refresh_token")
        .map(|c| c.value())
        .ok_or(AppError::Unauthorized)?;

    let (selector_str, raw_verifier) = cookie_val.split_once(':').ok_or(AppError::Unauthorized)?;
    let selector = uuid::Uuid::parse_str(selector_str).map_err(|_| AppError::Unauthorized)?;

    let token_record = query!(
        r#"
        SELECT user_id, token_hash, expires_at FROM refresh_tokens
        WHERE id = $1
        "#,
        selector
    )
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::Unauthorized)?;

    let is_valid = password_verify(raw_verifier.to_string(), token_record.token_hash).await?;
    if !is_valid {
        query!(
            "DELETE FROM refresh_tokens WHERE user_id = $1",
            token_record.user_id
        )
        .execute(&state.pool)
        .await?;
        return Err(AppError::Unauthorized);
    }

    query!("DELETE FROM refresh_tokens WHERE id = $1", selector)
        .execute(&state.pool)
        .await?;

    if token_record.expires_at < OffsetDateTime::now_utc() {
        return Err(AppError::Unauthorized);
    }

    let token_response = generate_tokens(token_record.user_id, &state.jwt_encoding_key)?;
    let new_selector = Uuid::now_v7();
    let new_token_hash = password_hash(token_response.refresh_token.clone()).await?;
    let refresh_expiry = OffsetDateTime::now_utc() + Duration::days(7);

    query!(
        r#"
        INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
        VALUES ($1, $2, $3, $4)
        "#,
        new_selector,
        token_record.user_id,
        new_token_hash,
        refresh_expiry
    )
    .execute(&state.pool)
    .await?;

    let cookie_value = format!("{}:{}", new_selector, token_response.refresh_token);
    let cookie = Cookie::build(("refresh_token", cookie_value))
        .path("/auth")
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

    let cookie_header = cookie.to_string().parse().map_err(|_| AppError::Internal)?;
    response.headers_mut().insert(SET_COOKIE, cookie_header);

    Ok(response)
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> Result<Response, AppError> {
    if let Some(selector) = jar
        .get("refresh_token")
        .and_then(|c| c.value().split_once(':'))
        .and_then(|(sel, _)| uuid::Uuid::parse_str(sel).ok())
    {
        let _ = query!("DELETE FROM refresh_tokens WHERE id = $1", selector)
            .execute(&state.pool)
            .await;
    }

    let removal_cookie = Cookie::build(("refresh_token", ""))
        .path("/auth")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(Duration::ZERO)
        .build();

    let mut response = StatusCode::OK.into_response();

    let cookie_header = removal_cookie
        .to_string()
        .parse()
        .map_err(|_| AppError::Internal)?;
    response.headers_mut().insert(SET_COOKIE, cookie_header);

    Ok(response)
}
