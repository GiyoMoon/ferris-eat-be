use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn validate_shopping_id(
    shopping_id: i32,
    user_id: Uuid,
    default_err: (StatusCode, String),
    pool: &PgPool,
) -> Result<(), (StatusCode, String)> {
    sqlx::query!(
        r#"SELECT * FROM shopping WHERE id = $1 AND user_id = $2"#,
        shopping_id,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err)?
    .ok_or((StatusCode::NOT_FOUND, "Shopping list not found".to_string()))
    .map(|_| ())
}

pub async fn validate_recipe_id(
    recipe_id: i32,
    user_id: Uuid,
    default_err: (StatusCode, String),
    pool: &PgPool,
) -> Result<(), (StatusCode, String)> {
    sqlx::query!(
        r#"SELECT * FROM recipe WHERE id = $1 AND user_id = $2"#,
        recipe_id,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())?
    .ok_or((StatusCode::NOT_FOUND, "Recipe not found".to_string()))
    .map(|_| ())
}

pub struct ShoppingIngredientId {
    id: i32,
}

pub async fn add_shopping_quantity(
    ingredient_id: i32,
    ingredient_quantity: i32,
    user_id: Uuid,
    shopping_id: i32,
    recipe_id: Option<i32>,
    default_err: (StatusCode, String),
    pool: &PgPool,
) -> Result<(), (StatusCode, String)> {
    sqlx::query!(
        r#"SELECT * FROM ingredient WHERE id = $1 AND user_id = $2"#,
        ingredient_id,
        user_id
    )
    .fetch_one(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::NOT_FOUND,
            format!("Ingredient with id {} not found", ingredient_id),
        )
    })?;

    let shopping_ingredient = sqlx::query_as!(
        ShoppingIngredientId,
        r#"SELECT id FROM shopping_ingredient WHERE shopping_id = $1 AND ingredient_id = $2"#,
        shopping_id,
        ingredient_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())?;

    let shopping_ingredient = match shopping_ingredient {
        Some(shopping_ingredient) => shopping_ingredient,
        None => sqlx::query_as!(
            ShoppingIngredientId,
            r#"
                INSERT INTO shopping_ingredient ( shopping_id, ingredient_id, checked )
                VALUES ( $1, $2, false) RETURNING id
            "#,
            shopping_id,
            ingredient_id,
        )
        .fetch_one(pool)
        .await
        .map_err(|_| default_err.clone())?,
    };

    let shopping_quantity = sqlx::query!(
        r#"
            SELECT quantity from shopping_quantity
            WHERE shopping_ingredient_id = $1 AND recipe_id IS NOT DISTINCT FROM $2
        "#,
        shopping_ingredient.id,
        recipe_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())?;

    if let Some(shopping_quantity) = shopping_quantity {
        sqlx::query!(
            r#"
                UPDATE shopping_quantity
                SET quantity = $1
                WHERE shopping_ingredient_id = $2 AND recipe_id IS NOT DISTINCT FROM $3
            "#,
            shopping_quantity.quantity + ingredient_quantity,
            shopping_ingredient.id,
            recipe_id
        )
        .execute(pool)
        .await
        .map_err(|_| default_err.clone())?;
    } else {
        sqlx::query!(
            r#"
                INSERT INTO shopping_quantity ( shopping_ingredient_id, recipe_id, quantity )
                VALUES ( $1, $2, $3)
            "#,
            shopping_ingredient.id,
            recipe_id,
            ingredient_quantity,
        )
        .execute(pool)
        .await
        .map_err(|_| default_err.clone())?;
    }

    Ok(())
}
