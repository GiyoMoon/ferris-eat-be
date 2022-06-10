use chrono::{Duration, Utc};
use entity::entities::user;
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::app::EnvVars;

use super::auth::{Claims, Tokens};

pub fn login(
    user: &user::Model,
    env_vars: &EnvVars,
) -> Result<Tokens, jsonwebtoken::errors::Error> {
    let token = encode(
        &Header::default(),
        &Claims::new(
            user.id,
            (Utc::now() + Duration::minutes(5)).timestamp() as u64,
        ),
        &EncodingKey::from_secret(env_vars.secret.as_ref()),
    )?;

    let refresh_token = encode(
        &Header::default(),
        &Claims::new(user.id, (Utc::now() + Duration::days(7)).timestamp() as u64),
        &EncodingKey::from_secret(
            (env_vars.refresh_secret.clone() + user.password.as_str()).as_ref(),
        ),
    )?;

    Ok(Tokens {
        token,
        refresh_token,
    })
}

pub fn refresh_token(
    claims: Claims,
    password: String,
    env_vars: &EnvVars,
) -> Result<Tokens, jsonwebtoken::errors::Error> {
    let token = encode(
        &Header::default(),
        &Claims::new(
            claims.get_sub(),
            (Utc::now() + Duration::minutes(5)).timestamp() as u64,
        ),
        &EncodingKey::from_secret(env_vars.secret.as_ref()),
    )?;

    let refresh_token = encode(
        &Header::default(),
        &Claims::new(
            claims.get_sub(),
            (Utc::now() + Duration::days(7)).timestamp() as u64,
        ),
        &EncodingKey::from_secret((env_vars.refresh_secret.clone() + password.as_str()).as_ref()),
    )?;

    Ok(Tokens {
        token,
        refresh_token,
    })
}
