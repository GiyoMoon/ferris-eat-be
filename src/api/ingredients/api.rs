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
    pub sort: Option<i32>,
}

#[axum_macros::debug_handler]
pub async fn create(
    claims: Claims,
    extract::Json(payload): extract::Json<IngredientsCreateReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let max = sqlx::query!(
        r#"SELECT sort FROM ingredient WHERE user_id = $1 ORDER BY sort DESC LIMIT 1"#,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed sorting ingredient".to_string(),
        )
    })?;

    let sort = match payload.sort {
        Some(sort) => {
            if sort < 1 {
                1
            } else {
                match max {
                    Some(max) => {
                        if sort > max.sort + 1 {
                            max.sort + 1
                        } else {
                            sort
                        }
                    }
                    None => 1,
                }
            }
        }
        None => match max {
            Some(max) => max.sort + 1,
            None => 1,
        },
    };

    let ingredients_after = sqlx::query!(
        r#"SELECT id, sort FROM ingredient where sort >= $1 AND user_id = $2 ORDER BY sort"#,
        sort,
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

    sqlx::query!(
        r#"INSERT INTO ingredient ( name, unit_id, sort, user_id ) VALUES ( $1, $2, $3, $4 )"#,
        payload.name,
        payload.unit_id,
        sort,
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

#[derive(Deserialize)]
pub struct IngredientUpdateReq {
    name: Option<String>,
    unit_id: Option<i32>,
}

#[axum_macros::debug_handler]
pub async fn update(
    claims: Claims,
    Path(id): Path<i32>,
    extract::Json(payload): extract::Json<IngredientUpdateReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    sqlx::query!(
        r#"SELECT id FROM ingredient WHERE id = $1 AND user_id = $2"#,
        id,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed updating ingredient".to_string(),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Ingredient not found".to_string()))?;

    if let Some(name) = payload.name {
        sqlx::query!(r#"UPDATE ingredient SET name = $1 WHERE id = $2"#, name, id,)
            .execute(pool)
            .await
            .map_err(|_| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed updating ingredient".to_string(),
                )
            })?;
    }

    if let Some(unit_id) = payload.unit_id {
        sqlx::query!(
            r#"UPDATE ingredient SET unit_id = $1 WHERE id = $2"#,
            unit_id,
            id,
        )
        .execute(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed updating ingredient".to_string(),
            )
        })?;
    }

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct IngredientSortReq {
    id: i32,
    new_sort: i32,
}

#[axum_macros::debug_handler]
pub async fn sort(
    claims: Claims,
    extract::Json(mut payload): extract::Json<IngredientSortReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Last ingredient
    let max = sqlx::query!(
        r#"SELECT sort FROM ingredient WHERE user_id = $1 ORDER BY sort DESC LIMIT 1"#,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed sorting ingredient".to_string(),
        )
    })?;

    // Old position of ingredient
    let old_sort = sqlx::query!(
        r#"SELECT sort FROM ingredient WHERE id = $1 AND user_id = $2 ORDER BY sort DESC LIMIT 1"#,
        payload.id,
        claims.get_sub(),
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed sorting ingredient".to_string(),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Ingredient not found".to_string()))?
    .sort;

    if payload.new_sort < 1 {
        payload.new_sort = 1;
    } else {
        if let Some(max) = max {
            // Ingredient is already at last position, no need to sort
            if payload.new_sort > max.sort + 1 && old_sort == max.sort {
                return Err((StatusCode::BAD_REQUEST, "Nothing to sort".to_string()));
            } else if payload.new_sort > max.sort + 1 {
                payload.new_sort = max.sort + 1;
            }
        } else {
            payload.new_sort = 1;
        }
    }

    if payload.new_sort == old_sort {
        return Err((StatusCode::BAD_REQUEST, "Nothing to sort".to_string()));
    }

    if payload.new_sort < old_sort {
        let add_sort = sqlx::query!(
            r#"SELECT id, sort FROM ingredient where sort >= $1 AND sort < $2 AND user_id = $3 ORDER BY sort"#,
            payload.new_sort,
            old_sort,
            claims.get_sub()
        )
        .fetch_all(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed sorting ingredient".to_string(),
            )
        })?;

        for ingredient in add_sort.into_iter() {
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
                    "Failed sorting ingredient".to_string(),
                )
            })?;
        }
    }

    if payload.new_sort > old_sort {
        let subtract_sort = sqlx::query!(
            r#"SELECT id, sort FROM ingredient where sort > $1 AND sort < $2 AND user_id = $3 ORDER BY sort"#,
            old_sort,
            payload.new_sort,
            claims.get_sub()
        )
        .fetch_all(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed sorting ingredient".to_string(),
            )
        })?;

        for ingredient in subtract_sort.into_iter() {
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
                    "Failed sorting ingredient".to_string(),
                )
            })?;
        }
    }

    sqlx::query!(
        r#"UPDATE ingredient SET sort = $1 WHERE id = $2"#,
        if payload.new_sort < old_sort {
            payload.new_sort
        } else {
            payload.new_sort - 1
        },
        payload.id
    )
    .execute(pool)
    .await
    .map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed sorting ingredient".to_string(),
        )
    })?;

    Ok(StatusCode::OK)
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
