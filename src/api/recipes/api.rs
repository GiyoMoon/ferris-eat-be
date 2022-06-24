use crate::api::{auth::Claims, recipes::service::save_recipe_ingredients};
use axum::{
    extract::{self, Path},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::{OffsetDateTime, PrimitiveDateTime};

#[derive(Serialize)]
pub struct GetAllRes {
    pub id: i32,
    pub name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub ingredients: i64,
}

#[axum_macros::debug_handler]
pub async fn get_all(
    claims: Claims,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Vec<GetAllRes>>), (StatusCode, String)> {
    let recipes = sqlx::query!(
        r#"
            SELECT recipe.id, recipe.name, recipe.created_at, recipe.updated_at, count(iq.id) AS ingredients
            FROM recipe
            LEFT OUTER JOIN recipe_quantity AS iq ON recipe.id = iq.recipe_id
            WHERE recipe.user_id = $1 GROUP BY recipe.id
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
                .map(|record| GetAllRes {
                    id: record.id,
                    name: record.name,
                    created_at: record.created_at.assume_utc(),
                    updated_at: record.updated_at.assume_utc(),
                    ingredients: record.ingredients.unwrap_or(0),
                })
                .collect(),
        ),
    ))
}

#[derive(Deserialize)]
pub struct CreateReq {
    pub name: String,
    pub ingredients: Vec<IngredientWithQuantity>,
}

#[derive(Deserialize)]
pub struct IngredientWithQuantity {
    pub id: i32,
    pub quantity: i32,
}

#[axum_macros::debug_handler]
pub async fn create(
    claims: Claims,
    extract::Json(payload): extract::Json<CreateReq>,
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

    save_recipe_ingredients(
        insert_result.id,
        claims.get_sub(),
        &payload.ingredients,
        pool,
    )
    .await?;

    Ok(StatusCode::CREATED)
}

#[derive(Serialize)]
pub struct GetRes {
    pub id: i32,
    pub name: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub ingredients: Vec<IngredientDetail>,
}

#[derive(Serialize)]
pub struct IngredientDetail {
    pub id: i32,
    pub name: String,
    pub unit: String,
    pub quantity: i32,
}

impl GetRes {
    pub fn new(recipe: RecipeQuery, mut ingredients: Vec<IngredientForRecipeQuery>) -> Self {
        ingredients.sort_by(|a, b| a.sort.cmp(&b.sort));
        let ingredients = ingredients
            .into_iter()
            .map(|i| IngredientDetail {
                id: i.id,
                name: i.name,
                unit: i.unit,
                quantity: i.quantity,
            })
            .collect();

        GetRes {
            id: recipe.id,
            name: recipe.name,
            created_at: recipe.created_at.assume_utc(),
            updated_at: recipe.updated_at.assume_utc(),
            ingredients,
        }
    }
}

pub struct RecipeQuery {
    pub id: i32,
    pub name: String,
    pub created_at: PrimitiveDateTime,
    pub updated_at: PrimitiveDateTime,
}

pub struct IngredientForRecipeQuery {
    pub id: i32,
    pub name: String,
    pub unit: String,
    pub quantity: i32,
    pub sort: i32,
}

#[axum_macros::debug_handler]
pub async fn get(
    claims: Claims,
    Path(id): Path<i32>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<GetRes>), (StatusCode, String)> {
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

    let ingredients = sqlx::query_as!(
        IngredientForRecipeQuery,
        r#"
            SELECT i.id, i.name, u.name AS unit, inq.quantity, i.sort FROM recipe_quantity AS inq
            INNER JOIN ingredient AS i ON inq.ingredient_id = i.id
            INNER JOIN unit AS u ON i.unit_id = u.id
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

    Ok((StatusCode::OK, Json(GetRes::new(recipe, ingredients))))
}

#[derive(Deserialize)]
pub struct UpdateReq {
    pub name: Option<String>,
    pub ingredients: Option<Vec<IngredientWithQuantity>>,
}

#[axum_macros::debug_handler]
pub async fn update(
    claims: Claims,
    Path(id): Path<i32>,
    extract::Json(payload): extract::Json<UpdateReq>,
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

    if let Some(ref name) = payload.name {
        sqlx::query!(r#"UPDATE recipe SET name = $1 WHERE id = $2"#, name, id,)
            .execute(pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed updating recipe".to_string(),
                )
            })?;
    }

    if let Some(ref ingredients) = payload.ingredients {
        {
            sqlx::query!(r#"DELETE FROM recipe_quantity WHERE recipe_id = $1"#, id)
                .execute(pool)
                .await
                .map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed updating recipe".to_string(),
                    )
                })?;

            save_recipe_ingredients(id, claims.get_sub(), ingredients, pool).await?;
        }
    }

    if payload.name.is_none() && payload.ingredients.is_some() {
        let updated = OffsetDateTime::now_utc();
        sqlx::query!(
            r#"UPDATE recipe SET updated_at = $1 WHERE id = $2"#,
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
