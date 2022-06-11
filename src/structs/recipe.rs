use serde::Serialize;
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
