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
}

impl From<Model> for LoginUser {
    fn from(a: Model) -> Self {
        LoginUser {
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
        let hash = bcrypt::hash(clear_text_password, 10);
        match hash {
            Ok(hash) => Ok(Password(hash)),
            Err(e) => Err(e),
        }
    }

    pub fn get(&self) -> &str {
        &self.0
    }
}
