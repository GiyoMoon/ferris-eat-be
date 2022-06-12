use crate::{
    api::{auth::Claims, recipes::service::save_recipe_ingredients},
    structs::recipe::{
        IngredientForRecipeQuery, RecipeCreateReq, RecipeGetDetailRes, RecipeGetRes, RecipeQuery,
        RecipeUpdateReq,
    },
};
use axum::{
    extract::{self, Path},
    http::StatusCode,
    Extension, Json,
};
use sqlx::PgPool;
use time::PrimitiveDateTime;
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
            "Failed creating recipe".to_string(),
        )
    })?;

    save_recipe_ingredients(insert_result.id, payload.ingredients, pool).await?;

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
            "Failed getting recipe".to_string(),
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
            "Failed getting recipe".to_string(),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(RecipeGetDetailRes::new(recipe, ingredients)),
    ))
}

#[axum_macros::debug_handler]
pub async fn update(
    claims: Claims,
    Path(id): Path<i32>,
    extract::Json(payload): extract::Json<RecipeUpdateReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query!(
        r#"SELECT id FROM recipe WHERE id = $1 AND user_id = $2"#,
        id,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed updating recipe".to_string(),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Recipe not found".to_string()))?;

    if let Some(name) = payload.name {
        let updated = time::OffsetDateTime::now_utc();
        sqlx::query!(
            r#"UPDATE recipe SET name = $1, updated_at = $2 WHERE id = $3"#,
            name,
            PrimitiveDateTime::new(updated.date(), updated.time()),
            id,
        )
        .execute(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed updating recipe".to_string(),
            )
        })?;
    }

    if let Some(ingredients) = payload.ingredients {
        {
            sqlx::query!(
                r#"DELETE FROM ingredient_quantity WHERE recipe_id = $1"#,
                id
            )
            .execute(pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed updating recipe".to_string(),
                )
            })?;

            save_recipe_ingredients(id, ingredients, pool).await?;
        }
    }

    Ok(StatusCode::OK)
}

#[axum_macros::debug_handler]
pub async fn delete(
    claims: Claims,
    Path(id): Path<i32>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query!(
        r#"
        DELETE FROM recipe
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
            "Failed deleting recipe".to_string(),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Recipe not found".to_string()))?;

    Ok(StatusCode::OK)
}
