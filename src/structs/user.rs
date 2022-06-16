use bcrypt::{BcryptError, BcryptResult};
use fancy_regex::Regex;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use time::{PrimitiveDateTime, OffsetDateTime};
use uuid::Uuid;
use validator::{Validate, ValidationError};

pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub alias: String,
    pub email: String,
    pub password: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
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
pub struct UserRegisterReq {
    #[validate(length(min = 4, message = "Username has to be 4 or more characters long"))]
    pub username: String,
    pub alias: String,
    #[validate(email(message = "Not a valid email address"))]
    pub email: String,
    #[validate(custom(
        function = "validate_password",
        message = "Password must at least be 8 chars long, contain at least one letter, one number and one special character"
    ))]
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct UserUpdateReq {
    pub alias: String,
    #[validate(email(message = "Not a valid email address"))]
    pub email: String,
}

#[derive(Deserialize)]
pub struct UserLoginReq {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Validate)]
pub struct UserChangePasswordReq {
    pub old_password: String,
    #[validate(custom(
        function = "validate_password",
        message = "Password must at least be 8 chars long, contain at least one letter, one number and one special character"
    ))]
    pub new_password: String,
}

#[derive(Serialize)]
pub struct UserMeRes {
    pub id: Uuid,
    pub username: String,
    pub alias: String,
    pub email: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime
}

impl From<UserModel> for UserMeRes {
    fn from(a: UserModel) -> UserMeRes {
        UserMeRes {
            id: a.id,
            username: a.username,
            alias: a.alias,
            email: a.email,
            created_at: a.created_at.assume_utc(),
            updated_at: a.updated_at.assume_utc(),
        }
    }
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
