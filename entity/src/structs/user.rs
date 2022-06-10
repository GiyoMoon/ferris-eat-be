use bcrypt::{BcryptError, BcryptResult};
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

#[derive(Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}

pub struct Password(String);

impl Password {
    pub fn from_plain(clear_text_password: String) -> Result<Password, BcryptError> {
        Ok(Password(bcrypt::hash(clear_text_password, 10)?))
    }

    pub fn from_hash(hash: String) -> Password {
        Password(hash)
    }

    pub fn get(&self) -> &str {
        &self.0
    }

    pub fn verify(&self, clear_text_password: String) -> BcryptResult<bool> {
        bcrypt::verify(clear_text_password, &self.0)
    }
}
