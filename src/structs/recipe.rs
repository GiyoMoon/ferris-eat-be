use serde::{Deserialize, Serialize};
use time_3::OffsetDateTime;

#[derive(Serialize)]
pub struct RecipeGetRes {
    pub id: i32,
    pub name: String,
    #[serde(with = "time_3::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time_3::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub ingredients: i64,
}

#[derive(Deserialize)]
pub struct RecipeCreateReq {
    pub name: String,
    pub ingredients: Vec<RecipeCreateIngredients>,
}

#[derive(Deserialize)]
pub struct RecipeCreateIngredients {
    pub id: i32,
    pub quantity: i32,
}
