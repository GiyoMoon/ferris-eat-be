use chrono::{Duration, Utc};
use entity::entities::user;
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

use crate::app::EnvVars;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Claims {
    sub: Uuid,
    exp: u64,
}

impl Claims {
    fn new(user_id: Uuid, exp: u64) -> Self {
        Self { sub: user_id, exp }
    }
}

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
