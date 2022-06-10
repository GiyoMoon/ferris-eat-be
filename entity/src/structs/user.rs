use bcrypt::{BcryptError, BcryptResult};
use fancy_regex::Regex;
use once_cell::sync::Lazy;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use crate::entities::user::Model;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub alias: String,
    pub email: String,
}

impl From<Model> for UserInfo {
    fn from(a: Model) -> Self {
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

#[derive(Deserialize, Validate)]
pub struct UserUpdate {
    pub alias: String,
    #[validate(email(message = "Not a valid email address"))]
    pub email: String,
}

static PASSWORD_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?=.*[A-Za-z])(?=.*\d)(?=.*[^\dA-Za-z]).{8,}").unwrap());

fn validate_password(value: &str) -> Result<(), ValidationError> {
    let is_match = PASSWORD_REGEX
        .is_match(value)
        .map_err(|_| ValidationError::new("invalid_password"))?;
    if !is_match {
        return Err(ValidationError::new("invalid_password"));
    }
    Ok(())
}

#[derive(Deserialize, Validate)]
pub struct UserChangePassword {
    pub old_password: String,
    #[validate(custom(
        function = "validate_password",
        message = "Password must at least be 8 chars long, contain at least one letter, one number and one special character"
    ))]
    pub new_password: String,
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
