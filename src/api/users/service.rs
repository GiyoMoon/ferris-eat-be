use axum::http::StatusCode;
use chrono::{Duration, Utc};
use entity::entities::user;
use jsonwebtoken::{encode, EncodingKey, Header};
use once_cell::sync::Lazy;
use sea_orm::{prelude::Uuid, DatabaseConnection, EntityTrait};

use super::auth::{Claims, Tokens};

static SECRET: Lazy<String> = Lazy::new(|| {
    let secret = std::env::var("SECRET").expect("SECRET env var not found");
    secret
});

static REFRESH_SECRET: Lazy<String> = Lazy::new(|| {
    let secret = std::env::var("REFRESH_SECRET").expect("REFRESH_SECRET env var not found");
    secret
});

pub fn get_tokens(uuid: Uuid, password: String) -> Result<Tokens, (StatusCode, String)> {
    let token = encode(
        &Header::default(),
        &Claims::new(uuid, (Utc::now() + Duration::minutes(5)).timestamp() as u64),
        &EncodingKey::from_secret(SECRET.as_ref()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating tokens".to_string(),
        )
    })?;

    let refresh_token = encode(
        &Header::default(),
        &Claims::new(uuid, (Utc::now() + Duration::days(7)).timestamp() as u64),
        &EncodingKey::from_secret((REFRESH_SECRET.clone() + password.as_str()).as_ref()),
    )
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating tokens".to_string(),
        )
    })?;

    Ok(Tokens {
        token,
        refresh_token,
    })
}

pub async fn get_user_by_uuid(
    uuid: Uuid,
    connection: &DatabaseConnection,
) -> Result<user::Model, (StatusCode, String)> {
    Ok(user::Entity::find_by_id(uuid)
        .one(connection)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed updating user".to_string(),
            )
        })?
        .ok_or((StatusCode::UNAUTHORIZED, "Failed updating user".to_string()))?)
}
