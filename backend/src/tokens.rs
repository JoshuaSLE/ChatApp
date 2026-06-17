use axum::{
    RequestPartsExt,
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{EncodingKey, Header, Validation, decode, encode};
use rand::{Rng, rng};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    AppState,
    errors::AppError,
    models::tokens::{Claims, TokenResponse},
};

impl<S> FromRequestParts<S> for Claims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AppError::MissingCredentials)?;

        let token_data = decode::<Claims>(
            bearer.token(),
            &app_state.jwt_decoding_key,
            &Validation::default(),
        )
        .map_err(|_| AppError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

pub fn generate_tokens(
    user_id: Uuid,
    encoding_key: &EncodingKey,
) -> Result<TokenResponse, AppError> {
    let now = OffsetDateTime::now_utc();
    let access_expiry = now + Duration::minutes(15);

    let claims = Claims {
        sub: user_id,
        iat: now.unix_timestamp(),
        exp: access_expiry.unix_timestamp(),
    };

    let access_token = encode(&Header::default(), &claims, encoding_key)?;

    let mut rand_bytes = [0u8; 32];
    rng().fill_bytes(&mut rand_bytes);
    let refresh_token = hex::encode(rand_bytes);

    Ok(TokenResponse {
        access_token,
        refresh_token,
    })
}
