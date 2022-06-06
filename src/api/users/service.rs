use chrono::{Duration, Utc};
use entity::entities::user;
use jsonwebtoken::{encode, EncodingKey, Header};

use crate::app::EnvVars;

use super::auth::{Claims, RefreshClaims};

pub fn login(
    user: &user::Model,
    env_vars: &EnvVars,
) -> Result<(String, String), jsonwebtoken::errors::Error> {
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
        &EncodingKey::from_secret(env_vars.refresh_secret.as_ref()),
    )?;

    Ok((token, refresh_token))
}

pub fn refresh_token(
    refresh_claims: RefreshClaims,
    env_vars: &EnvVars,
) -> Result<(String, String), jsonwebtoken::errors::Error> {
    let token = encode(
        &Header::default(),
        &Claims::new(
            refresh_claims.get_sub(),
            (Utc::now() + Duration::minutes(5)).timestamp() as u64,
        ),
        &EncodingKey::from_secret(env_vars.secret.as_ref()),
    )?;

    let refresh_token = encode(
        &Header::default(),
        &RefreshClaims::new(
            refresh_claims.get_sub(),
            (Utc::now() + Duration::days(7)).timestamp() as u64,
        ),
        &EncodingKey::from_secret(env_vars.refresh_secret.as_ref()),
    )?;

    Ok((token, refresh_token))
}
