use crate::api::{
    auth::Claims,
    global::{get_default_err, ValidatedJson},
    shopping::service::{add_shopping_quantity, validate_recipe_id, validate_shopping_id},
};
use axum::{extract::Path, http::StatusCode, Extension};
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct AddRecipeReq {
    #[validate]
    ingredients: Vec<Ingredient>,
}

#[derive(Deserialize, Validate)]
pub struct Ingredient {
    id: i32,
    #[validate(range(min = 1, message = "Quantity has to be at least 1"))]
    quantity: i32,
}

#[axum_macros::debug_handler]
pub async fn add_recipe(
    claims: Claims,
    Path((id, recipe_id)): Path<(i32, i32)>,
    ValidatedJson(payload): ValidatedJson<AddRecipeReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed adding recipe to shopping list");

    validate_shopping_id(id, claims.get_sub(), default_err.clone(), pool).await?;

    validate_recipe_id(recipe_id, claims.get_sub(), default_err.clone(), pool).await?;

    for ingredient in payload.ingredients.into_iter() {
        add_shopping_quantity(
            ingredient.id,
            ingredient.quantity,
            claims.get_sub(),
            id,
            Some(recipe_id),
            default_err.clone(),
            pool,
        )
        .await?;
    }

    Ok(StatusCode::CREATED)
}

#[axum_macros::debug_handler]
pub async fn delete_recipe(
    claims: Claims,
    Path((id, recipe_id)): Path<(i32, i32)>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed deleting shopping recipe");

    validate_shopping_id(id, claims.get_sub(), default_err.clone(), pool).await?;

    validate_recipe_id(recipe_id, claims.get_sub(), default_err.clone(), pool).await?;

    let shopping_quantities = sqlx::query!(
        r#"
            SELECT sq.id, sq.shopping_ingredient_id, COUNT(*) AS quantities
            FROM shopping_quantity AS sq
            JOIN shopping_ingredient AS si ON sq.shopping_ingredient_id = si.id
            JOIN shopping_quantity AS sq2  ON si.id = sq2.shopping_ingredient_id
            WHERE sq.recipe_id = $1 AND si.shopping_id = $2
            GROUP BY sq.id
        "#,
        recipe_id,
        id,
    )
    .fetch_all(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            "Shopping ingredient not found".to_string(),
        )
    })?;

    let sq_ids: &Vec<i32> = &shopping_quantities.iter().map(|sq| sq.id).collect();

    sqlx::query!(
        r#"DELETE FROM shopping_quantity WHERE id = ANY($1)"#,
        sq_ids
    )
    .execute(pool)
    .await
    .map_err(|_| default_err.clone())?;

    let si_ids: &Vec<i32> = &shopping_quantities
        .iter()
        .filter_map(|sq| {
            if sq.quantities.unwrap_or(0) > 1 {
                None
            } else {
                Some(sq.shopping_ingredient_id)
            }
        })
        .collect();

    sqlx::query!(
        r#"DELETE FROM shopping_ingredient WHERE id = ANY($1)"#,
        si_ids
    )
    .execute(pool)
    .await
    .map_err(|_| default_err)?;

    Ok(StatusCode::OK)
}
