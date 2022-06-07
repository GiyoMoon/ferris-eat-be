use bcrypt::BcryptError;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

use crate::entities::user::Model;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginUser {
    pub id: Uuid,
    pub username: String,
    pub alias: String,
    pub email: String,
    pub token: String,
    pub refresh_token: String,
}

impl LoginUser {
    pub fn new(a: Model, token: String, refresh_token: String) -> Self {
        LoginUser {
            id: a.id,
            username: a.username,
            alias: a.alias,
            email: a.email,
            token,
            refresh_token,
        }
    }
}

pub struct Password(String);

impl Password {
    pub fn from_plain(clear_text_password: String) -> Result<Password, BcryptError> {
        Ok(Password(bcrypt::hash(clear_text_password, 10)?))
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}
