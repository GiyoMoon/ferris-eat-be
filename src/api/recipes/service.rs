use crate::structs::recipe::RecipeIngredientWithQuantity;
use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn save_recipe_ingredients(
    recipe_id: i32,
    user_id: Uuid,
    ingredients: &[RecipeIngredientWithQuantity],
    pool: &PgPool,
) -> Result<(), (StatusCode, String)> {
    for ingredient in ingredients.iter() {
        sqlx::query!(
            r#"SELECT id FROM ingredient WHERE id = $1 AND user_id = $2"#,
            ingredient.id,
            user_id
        )
        .fetch_one(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                format!("Ingredient with id {} not found", ingredient.id),
            )
        })?;
        sqlx::query!(
            r#"
                INSERT INTO recipe_quantity ( recipe_id, ingredient_id, quantity )
                VALUES ( $1, $2, $3 )
                RETURNING id
            "#,
            recipe_id,
            ingredient.id,
            ingredient.quantity
        )
        .fetch_one(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed saving ingredients for recipe".to_string(),
            )
        })?;
    }
    Ok(())
}
