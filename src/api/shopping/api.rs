use axum::{http::StatusCode, Extension, Json};
use serde::Serialize;
use sqlx::PgPool;

use crate::api::auth::Claims;

#[derive(Serialize)]
pub struct ShoppingGetRes {
    pub id: i32,
    pub name: String,
    pub ingredients: i64,
    pub checked: i64,
}

#[axum_macros::debug_handler]
pub async fn get_all(
    claims: Claims,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Vec<ShoppingGetRes>>), (StatusCode, String)> {
    let recipes = sqlx::query!(
      r#"
      SELECT shopping.id, shopping.name, count(ins.id) filter (where ins.checked) as checked, count(ins.id) AS ingredients
      FROM shopping
      LEFT OUTER JOIN ingredient_shopping AS ins ON shopping.id = ins.shopping_id
      WHERE shopping.user_id = $1 group by shopping.id
      "#,
      claims.get_sub()
  )
  .fetch_all(pool)
  .await
  .map_err(|_| {
      (
          StatusCode::INTERNAL_SERVER_ERROR,
          "Failed getting shopping lists".to_string(),
      )
  })?;

    Ok((
        StatusCode::OK,
        Json(
            recipes
                .into_iter()
                .map(|record| ShoppingGetRes {
                    id: record.id,
                    name: record.name,
                    ingredients: record.ingredients.unwrap_or(0),
                    checked: record.checked.unwrap_or(0),
                })
                .collect(),
        ),
    ))
}
