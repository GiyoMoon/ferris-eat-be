use bcrypt::BcryptError;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

use crate::entities::user::Model;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub alias: String,
    pub email: String,
}

impl UserInfo {
    pub fn new(a: Model) -> Self {
        UserInfo {
            id: a.id,
            username: a.username,
            alias: a.alias,
            email: a.email,
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
