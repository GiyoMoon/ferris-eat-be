use crate::api::auth::Claims;
use axum::{
    extract::{self, Path},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize)]
pub struct ShoppingGetAllRes {
    pub id: i32,
    pub name: String,
    pub ingredients: i64,
    pub checked: i64,
}

#[axum_macros::debug_handler]
pub async fn get_all(
    claims: Claims,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Vec<ShoppingGetAllRes>>), (StatusCode, String)> {
    let recipes = sqlx::query!(
      r#"
      SELECT shopping.id, shopping.name, count(ins.id) filter (where ins.checked) AS checked, count(ins.id) AS ingredients
      FROM shopping
      LEFT OUTER JOIN ingredient_shopping AS ins ON shopping.id = ins.shopping_id
      WHERE shopping.user_id = $1 GROUP BY shopping.id
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
                .map(|record| ShoppingGetAllRes {
                    id: record.id,
                    name: record.name,
                    ingredients: record.ingredients.unwrap_or(0),
                    checked: record.checked.unwrap_or(0),
                })
                .collect(),
        ),
    ))
}

#[derive(Serialize)]
pub struct ShoppingGetRes {
    pub id: i32,
    pub name: String,
    pub ingredients: Vec<ShoppingIngredient>,
}

#[derive(Serialize)]
pub struct ShoppingIngredient {
    pub id: i32,
    pub name: String,
    pub unit: String,
    pub checked: bool,
    pub quantities: Vec<ShoppingQuantities>,
}

#[derive(Serialize, Clone)]
pub struct ShoppingQuantities {
    pub id: i32,
    pub quantity: i32,
    pub recipe_id: Option<i32>,
    pub recipe_name: Option<String>,
}

#[axum_macros::debug_handler]
pub async fn get(
    claims: Claims,
    Path(id): Path<i32>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<ShoppingGetRes>), (StatusCode, String)> {
    let shopping_list = sqlx::query!(
        r#"SELECT id, name FROM shopping WHERE id = $1 AND user_id = $2"#,
        id,
        claims.get_sub()
    )
    .fetch_one(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed getting shopping list".to_string(),
        )
    })?;

    let ingredients = sqlx::query!(
        r#"
            SELECT ins.id, ins.checked, i.name, u.name AS unit
            FROM ingredient_shopping AS ins
            JOIN ingredient AS i ON ins.ingredient_id = i.id
            JOIN unit AS u ON i.unit_id = u.id
            WHERE shopping_id = $1"#,
        id
    )
    .fetch_all(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed getting shopping list".to_string(),
        )
    })?;

    let ids: Vec<i32> = ingredients.iter().map(|i| i.id).collect();

    let quantities = sqlx::query_as!(
        ShoppingQuantities,
        r#"
            SELECT sq.id, sq.quantity, r.id AS recipe_id, r.name AS recipe_name
            FROM shopping_quantity AS sq
            LEFT JOIN recipe AS r ON sq.recipe_id = r.id
            WHERE ingredient_shopping_id = ANY($1)"#,
        &ids
    )
    .fetch_all(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed getting shopping list".to_string(),
        )
    })?;

    let ingredients = ingredients
        .into_iter()
        .map(|i| {
            let quantities = quantities
                .clone()
                .into_iter()
                .filter(|q| q.id == i.id)
                .collect();
            ShoppingIngredient {
                id: i.id,
                name: i.name,
                unit: i.unit,
                checked: i.checked,
                quantities,
            }
        })
        .collect();

    Ok((
        StatusCode::OK,
        Json(ShoppingGetRes {
            id: shopping_list.id,
            name: shopping_list.name,
            ingredients,
        }),
    ))
}

#[derive(Deserialize)]
pub struct ShoppingCreateReq {
    name: String,
}

#[axum_macros::debug_handler]
pub async fn create(
    claims: Claims,
    extract::Json(payload): extract::Json<ShoppingCreateReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query!(
        r#"INSERT INTO shopping ( name, user_id ) VALUES ( $1, $2 )"#,
        payload.name,
        claims.get_sub()
    )
    .execute(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating shopping list".to_string(),
        )
    })?;

    Ok(StatusCode::CREATED)
}
