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
      LEFT OUTER JOIN shopping_ingredient AS ins ON shopping.id = ins.shopping_id
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
            FROM shopping_ingredient AS ins
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
            WHERE shopping_ingredient_id = ANY($1)"#,
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

#[derive(Deserialize)]
pub struct ShoppingAddRecipeReq {
    recipe_id: i32,
    ingredients: Vec<IngredientQuantity>,
}

#[derive(Deserialize)]
pub struct IngredientQuantity {
    id: i32,
    quantity: i32,
}

pub struct IngredientShopping {
    id: i32,
}

#[axum_macros::debug_handler]
pub async fn add_recipe(
    claims: Claims,
    Path(id): Path<i32>,
    extract::Json(payload): extract::Json<ShoppingAddRecipeReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query!(
        r#"SELECT * FROM shopping WHERE id = $1 AND user_id = $2"#,
        id,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed adding recipe to shopping list".to_string(),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Shopping list not found".to_string()))?;

    sqlx::query!(
        r#"SELECT * FROM recipe WHERE id = $1 AND user_id = $2"#,
        payload.recipe_id,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed adding recipe to shopping list".to_string(),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Recipe not found".to_string()))?;

    for ingredient in payload.ingredients.into_iter() {
        sqlx::query!(
            r#"SELECT * FROM ingredient WHERE id = $1 AND user_id = $2"#,
            ingredient.id,
            claims.get_sub()
        )
        .fetch_one(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                format!("Ingredient with id {} not found", ingredient.id),
            )
        })?;

        let shopping_ingredient = sqlx::query_as!(
            IngredientShopping,
            r#"SELECT id FROM shopping_ingredient WHERE shopping_id = $1 AND ingredient_id = $2"#,
            id,
            ingredient.id,
        )
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                "Failed adding recipe to shopping list".to_string(),
            )
        })?;

        let shopping_ingredient = match shopping_ingredient {
            Some(shopping_ingredient) => shopping_ingredient,
            None => sqlx::query_as!(
                IngredientShopping,
                r#"
                    INSERT INTO shopping_ingredient ( shopping_id, ingredient_id, checked )
                    VALUES ( $1, $2, false) RETURNING id
                "#,
                id,
                ingredient.id,
            )
            .fetch_one(pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::NOT_FOUND,
                    "Failed adding recipe to shopping list".to_string(),
                )
            })?,
        };

        let shopping_quantity = sqlx::query!(
            r#"
                SELECT id from shopping_quantity
                WHERE shopping_ingredient_id = $1 AND recipe_id = $2
            "#,
            shopping_ingredient.id,
            payload.recipe_id
        )
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                "Failed adding recipe to shopping list".to_string(),
            )
        })?;

        if shopping_quantity.is_some() {
            sqlx::query!(
                r#"
                    UPDATE shopping_quantity
                    SET quantity = $1
                    WHERE shopping_ingredient_id = $2 AND recipe_id = $3
                "#,
                ingredient.quantity,
                shopping_ingredient.id,
                payload.recipe_id
            )
            .execute(pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::NOT_FOUND,
                    "Failed adding recipe to shopping list".to_string(),
                )
            })?;
        } else {
            sqlx::query!(
                r#"
                    INSERT INTO shopping_quantity ( shopping_ingredient_id, recipe_id, quantity )
                    VALUES ( $1, $2, $3)
                "#,
                shopping_ingredient.id,
                payload.recipe_id,
                ingredient.quantity,
            )
            .execute(pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::NOT_FOUND,
                    "Failed adding recipe to shopping list".to_string(),
                )
            })?;
        }
    }

    Ok(StatusCode::CREATED)
}
