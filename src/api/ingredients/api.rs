use crate::api::{
    auth::Claims,
    global::get_default_err,
    ingredients::service::{get_last_ingredient_by_sort, update_ingredient_sort},
};
use axum::{
    extract::{self, Path},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Serialize)]
pub struct GetRes {
    id: i32,
    name: String,
    unit: String,
    sort: i32,
}

#[axum_macros::debug_handler]
pub async fn get_all(
    claims: Claims,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Vec<GetRes>>), (StatusCode, String)> {
    let units = sqlx::query!(
        r#"
        SELECT i.id, i.name, u.name AS unit, i.sort FROM ingredient AS i
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

    let units: Vec<GetRes> = units
        .into_iter()
        .map(|record| GetRes {
            id: record.id,
            name: record.name,
            unit: record.unit,
            sort: record.sort,
        })
        .collect();

    Ok((StatusCode::OK, Json(units)))
}

#[derive(Deserialize)]
pub struct CreateReq {
    pub name: String,
    pub unit_id: i32,
    pub sort: Option<i32>,
}

#[axum_macros::debug_handler]
pub async fn create(
    claims: Claims,
    extract::Json(payload): extract::Json<CreateReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed creating ingredient");

    // Lat ingredient
    let max = get_last_ingredient_by_sort(claims.get_sub(), default_err.clone(), pool).await?;

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
        r#"SELECT id, sort FROM ingredient WHERE sort >= $1 AND user_id = $2 ORDER BY sort"#,
        sort,
        claims.get_sub()
    )
    .fetch_all(pool)
    .await
    .map_err(|_| default_err.clone())?;

    sqlx::query!(
        r#"INSERT INTO ingredient ( name, unit_id, sort, user_id ) VALUES ( $1, $2, $3, $4 )"#,
        payload.name,
        payload.unit_id,
        sort,
        claims.get_sub()
    )
    .execute(pool)
    .await
    .map_err(|_| default_err.clone())?;

    for ingredient in ingredients_after.into_iter() {
        update_ingredient_sort(
            ingredient.id,
            ingredient.sort + 1,
            default_err.clone(),
            pool,
        )
        .await?;
    }

    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
pub struct UpdateReq {
    name: Option<String>,
    unit_id: Option<i32>,
}

#[axum_macros::debug_handler]
pub async fn update(
    claims: Claims,
    Path(id): Path<i32>,
    extract::Json(payload): extract::Json<UpdateReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed updating ingredient");

    sqlx::query!(
        r#"SELECT id FROM ingredient WHERE id = $1 AND user_id = $2"#,
        id,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())?
    .ok_or((StatusCode::NOT_FOUND, "Ingredient not found".to_string()))?;

    if let Some(name) = payload.name {
        sqlx::query!(r#"UPDATE ingredient SET name = $1 WHERE id = $2"#, name, id,)
            .execute(pool)
            .await
            .map_err(|_| default_err.clone())?;
    }

    if let Some(unit_id) = payload.unit_id {
        sqlx::query!(
            r#"UPDATE ingredient SET unit_id = $1 WHERE id = $2"#,
            unit_id,
            id,
        )
        .execute(pool)
        .await
        .map_err(|_| default_err)?;
    }

    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct SortReq {
    new_sort: i32,
}

#[axum_macros::debug_handler]
pub async fn sort(
    claims: Claims,
    Path(id): Path<i32>,
    extract::Json(mut payload): extract::Json<SortReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed sorting ingredient");

    // Last ingredient
    let max = get_last_ingredient_by_sort(claims.get_sub(), default_err.clone(), pool).await?;

    // Old position of ingredient
    let old_sort = sqlx::query!(
        r#"SELECT sort FROM ingredient WHERE id = $1 AND user_id = $2 ORDER BY sort DESC LIMIT 1"#,
        id,
        claims.get_sub(),
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())?
    .ok_or((StatusCode::NOT_FOUND, "Ingredient not found".to_string()))?
    .sort;

    if payload.new_sort < 1 {
        payload.new_sort = 1;
    } else if let Some(max) = max {
        // Ingredient is already at last position, no need to sort
        if payload.new_sort > max.sort + 1 && old_sort == max.sort {
            return Err((StatusCode::BAD_REQUEST, "Nothing to sort".to_string()));
        } else if payload.new_sort > max.sort + 1 {
            payload.new_sort = max.sort + 1;
        }
    } else {
        payload.new_sort = 1;
    }

    if payload.new_sort == old_sort {
        return Err((StatusCode::BAD_REQUEST, "Nothing to sort".to_string()));
    }

    if payload.new_sort < old_sort {
        let add_sort = sqlx::query!(
            r#"SELECT id, sort FROM ingredient WHERE sort >= $1 AND sort < $2 AND user_id = $3 ORDER BY sort"#,
            payload.new_sort,
            old_sort,
            claims.get_sub()
        )
        .fetch_all(pool)
        .await
        .map_err(|_| default_err.clone())?;

        for ingredient in add_sort.into_iter() {
            update_ingredient_sort(
                ingredient.id,
                ingredient.sort + 1,
                default_err.clone(),
                pool,
            )
            .await?;
        }
    }

    if payload.new_sort > old_sort {
        let subtract_sort = sqlx::query!(
            r#"SELECT id, sort FROM ingredient WHERE sort > $1 AND sort < $2 AND user_id = $3 ORDER BY sort"#,
            old_sort,
            payload.new_sort,
            claims.get_sub()
        )
        .fetch_all(pool)
        .await
        .map_err(|_| default_err.clone())?;

        for ingredient in subtract_sort.into_iter() {
            update_ingredient_sort(
                ingredient.id,
                ingredient.sort - 1,
                default_err.clone(),
                pool,
            )
            .await?;
        }
    }

    update_ingredient_sort(
        id,
        if payload.new_sort < old_sort {
            payload.new_sort
        } else {
            payload.new_sort - 1
        },
        default_err.clone(),
        pool,
    )
    .await?;

    Ok(StatusCode::OK)
}

#[axum_macros::debug_handler]
pub async fn delete(
    claims: Claims,
    Path(id): Path<i32>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed deleting ingredient");

    let to_delete = sqlx::query!(
        r#"SELECT id, sort FROM ingredient WHERE id = $1 AND user_id = $2"#,
        id,
        claims.get_sub()
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())?
    .ok_or((StatusCode::NOT_FOUND, "Ingredient not found".to_string()))?;

    sqlx::query!(r#"DELETE FROM ingredient WHERE id = $1"#, id)
        .execute(pool)
        .await
        .map_err(|_| default_err.clone())?;

    let ingredients_after = sqlx::query!(
        r#"SELECT id, sort FROM ingredient WHERE sort > $1 AND user_id = $2 ORDER BY sort"#,
        to_delete.sort,
        claims.get_sub()
    )
    .fetch_all(pool)
    .await
    .map_err(|_| default_err.clone())?;

    for ingredient in ingredients_after.into_iter() {
        update_ingredient_sort(
            ingredient.id,
            ingredient.sort - 1,
            default_err.clone(),
            pool,
        )
        .await?;
    }

    Ok(StatusCode::CREATED)
}
