use crate::api::auth::Claims;
use axum::{
    extract::{self, Path},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize)]
pub struct IngredientsGetRes {
    id: i32,
    name: String,
    unit: String,
    sort: i32,
}

#[axum_macros::debug_handler]
pub async fn get_all(
    claims: Claims,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Vec<IngredientsGetRes>>), (StatusCode, String)> {
    let units = sqlx::query!(
        r#"
        SELECT i.id, i.name, u.name as unit, i.sort FROM ingredient AS i
        INNER JOIN unit AS u ON i.unit_id = u.id
        WHERE i.user_id = $1
        ORDER BY i.sort
        "#,
        claims.get_sub()
    )
    .fetch_all(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed getting ingredients".to_string(),
        )
    })?;

    let units: Vec<IngredientsGetRes> = units
        .into_iter()
        .map(|record| IngredientsGetRes {
            id: record.id,
            name: record.name,
            unit: record.unit,
            sort: record.sort,
        })
        .collect();

    Ok((StatusCode::OK, Json(units)))
}

#[derive(Deserialize)]
pub struct IngredientsCreateReq {
    pub name: String,
    pub unit_id: i32,
    pub sort: i32,
}

#[axum_macros::debug_handler]
pub async fn create(
    claims: Claims,
    extract::Json(mut payload): extract::Json<IngredientsCreateReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    if payload.sort < 1 {
        payload.sort = 1;
    }

    let ingredients_after = sqlx::query!(
        r#"SELECT id, sort FROM ingredient where sort >= $1 AND user_id = $2 ORDER BY sort"#,
        payload.sort,
        claims.get_sub()
    )
    .fetch_all(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating ingredient".to_string(),
        )
    })?;

    if ingredients_after.len() == 0 {
        let last_id = sqlx::query!(
            r#"SELECT sort FROM ingredient WHERE user_id = $1 ORDER BY sort DESC LIMIT 1"#,
            claims.get_sub()
        )
        .fetch_optional(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed creating ingredient".to_string(),
            )
        })?;

        if let Some(last_id) = last_id {
            if payload.sort > last_id.sort + 1 {
                payload.sort = last_id.sort + 1;
            }
        }
    }

    sqlx::query!(
        r#"INSERT INTO ingredient ( name, unit_id, sort, user_id ) VALUES ( $1, $2, $3, $4 )"#,
        payload.name,
        payload.unit_id,
        payload.sort,
        claims.get_sub()
    )
    .execute(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed creating ingredient".to_string(),
        )
    })?;

    for ingredient in ingredients_after.into_iter() {
        sqlx::query!(
            r#"UPDATE ingredient SET sort = $1 WHERE id = $2"#,
            ingredient.sort + 1,
            ingredient.id
        )
        .execute(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed creating ingredient".to_string(),
            )
        })?;
    }

    Ok(StatusCode::CREATED)
}

#[axum_macros::debug_handler]
pub async fn delete(
    claims: Claims,
    Path(id): Path<i32>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let to_delete = sqlx::query!(
        r#"SELECT id, sort FROM ingredient WHERE id = $1 AND user_id = $2"#,
        id,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed deleting ingredient".to_string(),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Ingredient not found".to_string()))?;

    sqlx::query!(r#"DELETE FROM ingredient WHERE id = $1"#, id)
        .execute(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed deleting ingredient".to_string(),
            )
        })?;

    let ingredients_after = sqlx::query!(
        r#"SELECT id, sort FROM ingredient where sort > $1 AND user_id = $2 ORDER BY sort"#,
        to_delete.sort,
        claims.get_sub()
    )
    .fetch_all(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed deleting ingredient".to_string(),
        )
    })?;

    for ingredient in ingredients_after.into_iter() {
        sqlx::query!(
            r#"UPDATE ingredient SET sort = $1 WHERE id = $2"#,
            ingredient.sort - 1,
            ingredient.id
        )
        .execute(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed deleting ingredient".to_string(),
            )
        })?;
    }

    Ok(StatusCode::CREATED)
}
