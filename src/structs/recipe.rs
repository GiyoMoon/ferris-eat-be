use serde::{Deserialize, Serialize};
use time::PrimitiveDateTime;
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
    pub ingredients: Vec<RecipeIngredientWithQuantity>,
}

#[derive(Deserialize)]
pub struct RecipeIngredientWithQuantity {
    pub id: i32,
    pub quantity: i32,
}

pub struct RecipeQuery {
    pub id: i32,
    pub name: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}

pub struct IngredientForRecipeQuery {
    pub id: i32,
    pub name: String,
    pub unit: String,
    pub quantity: i32,
    pub sort: i32,
}

#[derive(Serialize)]
pub struct RecipeGetDetailRes {
    pub id: i32,
    pub name: String,
    #[serde(with = "time_3::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time_3::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub ingredients: Vec<IngredientWithQuantity>,
}

impl RecipeGetDetailRes {
    pub fn new(recipe: RecipeQuery, mut ingredients: Vec<IngredientForRecipeQuery>) -> Self {
        ingredients.sort_by(|a, b| a.sort.cmp(&b.sort));
        let ingredients = ingredients
            .into_iter()
            .map(|i| IngredientWithQuantity {
                id: i.id,
                name: i.name,
                unit: i.unit,
                quantity: i.quantity,
            })
            .collect();

        RecipeGetDetailRes {
            id: recipe.id,
            name: recipe.name,
            created_at: OffsetDateTime::from_unix_timestamp(
                recipe.created_at.assume_utc().unix_timestamp(),
            )
            .unwrap(),
            updated_at: OffsetDateTime::from_unix_timestamp(
                recipe.updated_at.assume_utc().unix_timestamp(),
            )
            .unwrap(),
            ingredients,
        }
    }
}

#[derive(Serialize)]
pub struct IngredientWithQuantity {
    pub id: i32,
    pub name: String,
    pub unit: String,
    pub quantity: i32,
}

#[derive(Deserialize)]
pub struct RecipeUpdateReq {
    pub name: Option<String>,
    pub ingredients: Option<Vec<RecipeIngredientWithQuantity>>,
}
