use crate::api::{
    auth::Claims,
    global::{get_default_err, ValidatedJson},
    shopping::service::{add_shopping_quantity, validate_shopping_id},
};
use axum::{extract::Path, http::StatusCode, Extension};
use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct AddIngredientReq {
    #[validate(range(min = 1, message = "Quantity has to be at least 1"))]
    quantity: i32,
}

#[axum_macros::debug_handler]
pub async fn add_ingredient(
    claims: Claims,
    Path((id, ingredient_id)): Path<(i32, i32)>,
    ValidatedJson(payload): ValidatedJson<AddIngredientReq>,
    Extension(pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed adding ingredient to shopping list");

    validate_shopping_id(id, claims.get_sub(), default_err.clone(), &pool).await?;

    add_shopping_quantity(
        ingredient_id,
        payload.quantity,
        claims.get_sub(),
        id,
        None,
        default_err.clone(),
        &pool,
    )
    .await?;

    Ok(StatusCode::CREATED)
}

#[axum_macros::debug_handler]
pub async fn check_ingredient(
    claims: Claims,
    Path((id, ingredient_id)): Path<(i32, i32)>,
    Extension(pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed checking shopping ingredient");

    validate_shopping_id(id, claims.get_sub(), default_err.clone(), &pool).await?;

    sqlx::query!(
        r#"
            UPDATE shopping_ingredient
            SET checked = NOT checked
            WHERE id = $1 AND shopping_id = $2
            RETURNING id
        "#,
        ingredient_id,
        id,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| default_err)?
    .ok_or((
        StatusCode::NOT_FOUND,
        "Shopping ingredient not found".to_string(),
    ))?;

    Ok(StatusCode::OK)
}

#[axum_macros::debug_handler]
pub async fn delete_ingredient(
    claims: Claims,
    Path((id, ingredient_id)): Path<(i32, i32)>,
    Extension(pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed deleting shopping ingredient");

    validate_shopping_id(id, claims.get_sub(), default_err.clone(), &pool).await?;

    let shopping_ingredient = sqlx::query!(
        r#"
            SELECT id
            FROM shopping_ingredient
            WHERE id = $1 AND shopping_id = $2
        "#,
        ingredient_id,
        id,
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| default_err.clone())?
    .ok_or((
        StatusCode::NOT_FOUND,
        "Shopping ingredient not found".to_string(),
    ))?;

    // On delete cascade for shopping_quantities
    sqlx::query!(
        r#"DELETE FROM shopping_ingredient WHERE id = $1"#,
        shopping_ingredient.id
    )
    .execute(&pool)
    .await
    .map_err(|_| default_err)?;

    Ok(StatusCode::OK)
}
