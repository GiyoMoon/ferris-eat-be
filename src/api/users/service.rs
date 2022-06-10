use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use once_cell::sync::Lazy;
use sea_orm::prelude::Uuid;

use super::auth::{Claims, Tokens};

static SECRET: Lazy<String> = Lazy::new(|| {
    let secret = std::env::var("SECRET").expect("SECRET env var not found");
    secret
});

static REFRESH_SECRET: Lazy<String> = Lazy::new(|| {
    let secret = std::env::var("REFRESH_SECRET").expect("REFRESH_SECRET env var not found");
    secret
});

pub fn get_tokens(uuid: Uuid, password: String) -> Result<Tokens, jsonwebtoken::errors::Error> {
    let token = encode(
        &Header::default(),
        &Claims::new(uuid, (Utc::now() + Duration::minutes(5)).timestamp() as u64),
        &EncodingKey::from_secret(SECRET.as_ref()),
    )?;

    let refresh_token = encode(
        &Header::default(),
        &Claims::new(uuid, (Utc::now() + Duration::days(7)).timestamp() as u64),
        &EncodingKey::from_secret((REFRESH_SECRET.clone() + password.as_str()).as_ref()),
    )?;

    Ok(Tokens {
        token,
        refresh_token,
    })
}
