use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    headers::{authorization::Bearer, Authorization},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json, TypedHeader,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use migration::tests_cfg::json;
use once_cell::sync::Lazy;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

static SECRET: Lazy<String> = Lazy::new(|| {
    let secret = std::env::var("SECRET").expect("SECRET env var not found");
    secret
});

static REFRESH_SECRET: Lazy<String> = Lazy::new(|| {
    let secret = std::env::var("REFRESH_SECRET").expect("REFRESH_SECRET env var not found");
    secret
});

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Claims {
    sub: Uuid,
    exp: u64,
}

impl Claims {
    pub fn new(user_id: Uuid, exp: u64) -> Self {
        Self { sub: user_id, exp }
    }
    // pub fn get_sub(&self) -> Uuid {
    //     self.sub
    // }
}

#[async_trait]
impl<B> FromRequest<B> for Claims
where
    B: Send,
{
    type Rejection = AuthError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| AuthError::MissingCredentials)?;
        // Decode the user data
        let token_data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(&SECRET.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RefreshClaims {
    sub: Uuid,
    exp: u64,
}

impl RefreshClaims {
    pub fn new(user_id: Uuid, exp: u64) -> Self {
        Self { sub: user_id, exp }
    }
    pub fn get_sub(&self) -> Uuid {
        self.sub
    }
}

#[async_trait]
impl<B> FromRequest<B> for RefreshClaims
where
    B: Send,
{
    type Rejection = AuthError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| AuthError::MissingCredentials)?;
        // Decode the user data
        let token_data = decode::<RefreshClaims>(
            bearer.token(),
            &DecodingKey::from_secret(&REFRESH_SECRET.as_ref()),
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

pub enum AuthError {
    MissingCredentials,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}
