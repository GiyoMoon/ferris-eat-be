use sea_orm::{prelude::Uuid};
use serde::{Serialize, Deserialize};

use crate::entities::user::Model;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginUser {
  pub id: Uuid,
  pub username: String,
}

impl From<Model> for LoginUser {
  fn from(a: Model) -> Self {
    LoginUser { id: a.id, username: a.username }
  }
}
