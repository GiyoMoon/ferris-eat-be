use crate::{
    api::auth::Claims,
    structs::recipe::{
        IngredientForRecipeQuery, RecipeCreateReq, RecipeGetDetailRes, RecipeGetRes, RecipeQuery,
    },
};
use axum::{
    extract::{self, Path},
    http::StatusCode,
    Extension, Json,
};
use sqlx::PgPool;
use time_3::OffsetDateTime;

#[axum_macros::debug_handler]
pub async fn get_all(
    claims: Claims,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Vec<RecipeGetRes>>), (StatusCode, String)> {
    let recipes = sqlx::query!(
        r#"
        SELECT recipe.id,recipe. name, recipe.created_at,recipe. updated_at, count(iq.id) AS ingredients
        FROM recipe
        LEFT OUTER JOIN ingredient_quantity AS iq ON recipe.id = iq.recipe_id
        WHERE recipe.user_id = $1 group by recipe.id
        "#,
        claims.get_sub()
    )
    .fetch_all(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed getting recipes".to_string(),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(
            recipes
                .into_iter()
                .map(|record| RecipeGetRes {
                    id: record.id,
                    name: record.name,
                    created_at: OffsetDateTime::from_unix_timestamp(
                        record.created_at.assume_utc().unix_timestamp(),
                    )
                    .unwrap(),
                    updated_at: OffsetDateTime::from_unix_timestamp(
                        record.updated_at.assume_utc().unix_timestamp(),
                    )
                    .unwrap(),
                    ingredients: record.ingredients.unwrap_or(0),
                })
                .collect(),
        ),
    ))
}

#[axum_macros::debug_handler]
pub async fn create(
    claims: Claims,
    extract::Json(payload): extract::Json<RecipeCreateReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let insert_result = sqlx::query!(
        r#"
        INSERT INTO recipe ( name, user_id )
        VALUES ( $1, $2 )
        RETURNING id
        "#,
        payload.name,
        claims.get_sub()
    )
    .fetch_one(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error while creating recipe".to_string(),
        )
    })?;

    for ingredient in payload.ingredients.iter() {
        sqlx::query!(r#"SELECT id FROM ingredient where id = $1"#, ingredient.id)
            .fetch_one(pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::NOT_FOUND,
                    format!("Ingredient with id {} not found", ingredient.id).to_string(),
                )
            })?;
        sqlx::query!(
            r#"
            INSERT INTO ingredient_quantity ( recipe_id, ingredient_id, quantity )
            VALUES ( $1, $2, $3 )
            RETURNING id
            "#,
            insert_result.id,
            ingredient.id,
            ingredient.quantity
        )
        .fetch_one(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error while creating recipe".to_string(),
            )
        })?;
    }

    Ok(StatusCode::CREATED)
}

#[axum_macros::debug_handler]
pub async fn get(
    claims: Claims,
    Path(id): Path<i32>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<RecipeGetDetailRes>), (StatusCode, String)> {
    let recipe: RecipeQuery = sqlx::query_as!(
        RecipeQuery,
        r#"
        SELECT id, name, created_at, updated_at FROM recipe
        WHERE id = $1 AND user_id = $2
        "#,
        id,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error while getting recipe".to_string(),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Recipe not found".to_string()))?;

    let ingredients: Vec<IngredientForRecipeQuery> = sqlx::query_as!(
        IngredientForRecipeQuery,
        r#"
        SELECT i.id, i.name, u.name as unit, inq.quantity, i.sort FROM ingredient_quantity as inq
        INNER JOIN ingredient as i ON inq.ingredient_id = i.id
        INNER JOIN unit as u ON i.unit_id = u.id
        WHERE inq.recipe_id = $1
        "#,
        recipe.id
    )
    .fetch_all(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Error while getting recipe".to_string(),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(RecipeGetDetailRes::new(recipe, ingredients)),
    ))
}
