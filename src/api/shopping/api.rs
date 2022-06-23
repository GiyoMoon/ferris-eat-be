use crate::api::{auth::Claims, global::get_default_err};
use axum::{
    extract::{self, Path},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize)]
pub struct GetAllRes {
    pub id: i32,
    pub name: String,
    pub ingredients: i64,
    pub checked: i64,
}

#[axum_macros::debug_handler]
pub async fn get_all(
    claims: Claims,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Vec<GetAllRes>>), (StatusCode, String)> {
    let recipes = sqlx::query!(
      r#"
      SELECT shopping.id, shopping.name, count(si.id) filter (where si.checked) AS checked, count(si.id) AS ingredients
      FROM shopping
      LEFT OUTER JOIN shopping_ingredient AS si ON shopping.id = si.shopping_id
      WHERE shopping.user_id = $1 GROUP BY shopping.id
      "#,
      claims.get_sub()
  )
  .fetch_all(pool)
  .await
  .map_err(|_|  (
    StatusCode::INTERNAL_SERVER_ERROR,
    "Failed getting shopping lists".to_string(),
    ))?;

    Ok((
        StatusCode::OK,
        Json(
            recipes
                .into_iter()
                .map(|record| GetAllRes {
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
pub struct GetRes {
    pub id: i32,
    pub name: String,
    pub ingredients: Vec<Ingredient>,
}

#[derive(Serialize)]
pub struct Ingredient {
    pub id: i32,
    pub name: String,
    pub unit: String,
    pub checked: bool,
    pub quantities: Vec<Quantities>,
}

#[derive(Serialize, Clone)]
pub struct Quantities {
    pub id: i32,
    pub shopping_ingredient_id: i32,
    pub quantity: i32,
    pub recipe_id: Option<i32>,
    pub recipe_name: Option<String>,
}

#[axum_macros::debug_handler]
pub async fn get(
    claims: Claims,
    Path(id): Path<i32>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<GetRes>), (StatusCode, String)> {
    let default_err = get_default_err("Failed getting shopping list");

    let shopping_list = sqlx::query!(
        r#"SELECT id, name FROM shopping WHERE id = $1 AND user_id = $2"#,
        id,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())?
    .ok_or((StatusCode::NOT_FOUND, "Shopping list not found".to_string()))?;

    let ingredients = sqlx::query!(
        r#"
            SELECT si.id, si.checked, i.name, u.name AS unit
            FROM shopping_ingredient AS si
            JOIN ingredient AS i ON si.ingredient_id = i.id
            JOIN unit AS u ON i.unit_id = u.id
            WHERE shopping_id = $1"#,
        id
    )
    .fetch_all(pool)
    .await
    .map_err(|_| default_err.clone())?;

    let ids: Vec<i32> = ingredients.iter().map(|i| i.id).collect();

    let quantities = sqlx::query_as!(
        Quantities,
        r#"
            SELECT sq.id, sq.shopping_ingredient_id, sq.quantity, r.id AS recipe_id, r.name AS recipe_name
            FROM shopping_quantity AS sq
            LEFT JOIN recipe AS r ON sq.recipe_id = r.id
            WHERE shopping_ingredient_id = ANY($1)"#,
        &ids
    )
    .fetch_all(pool)
    .await
    .map_err(|_| default_err )?;

    let ingredients = ingredients
        .into_iter()
        .map(|i| {
            let quantities = quantities
                .clone()
                .into_iter()
                .filter(|q| q.shopping_ingredient_id == i.id)
                .collect();
            Ingredient {
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
        Json(GetRes {
            id: shopping_list.id,
            name: shopping_list.name,
            ingredients,
        }),
    ))
}

#[derive(Deserialize)]
pub struct CreateReq {
    name: String,
}

#[axum_macros::debug_handler]
pub async fn create(
    claims: Claims,
    extract::Json(payload): extract::Json<CreateReq>,
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

#[axum_macros::debug_handler]
pub async fn delete(
    claims: Claims,
    Path(id): Path<i32>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query!(
        r#"
            DELETE FROM shopping
            WHERE id = $1 AND user_id = $2
            RETURNING id
        "#,
        id,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed deleting shopping list".to_string(),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Shopping list not found".to_string()))?;

    Ok(StatusCode::OK)
}
