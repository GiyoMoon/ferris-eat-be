use bcrypt::{BcryptError, BcryptResult};
use once_cell::sync::Lazy;
use regex::Regex;
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

static LETTER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^.*[a-zA-Z]+.*$").unwrap());
static NUMBER_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^.*\d+.*$").unwrap());
static SPECIAL_CHAR_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^.*[^\da-zA-Z]+.*$").unwrap());

fn validate_username(value: &str) -> Result<(), ValidationError> {
    let letter = LETTER_REGEX.is_match(value);
    if !letter {
        return Err(ValidationError::new("invalid_password"));
    };
    let number = NUMBER_REGEX.is_match(value);
    if !number {
        return Err(ValidationError::new("invalid_password"));
    };
    let special_char = SPECIAL_CHAR_REGEX.is_match(value);
    if !special_char {
        return Err(ValidationError::new("invalid_password"));
    };
    Ok(())
}

#[derive(Deserialize, Validate)]
pub struct UserChangePassword {
    pub old_password: String,
    #[validate(
        length(min = 8, message = "Password has to be 8 or more characters long"),
        custom(
            function = "validate_username",
            message = "Password must contain at least one letter, one number and one special character"
        )
    )]
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
