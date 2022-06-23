use crate::api::{auth::Claims, global::get_default_err, shopping::service::validate_shopping_id};
use axum::{
    extract::{self, Path},
    http::StatusCode,
    Extension,
};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct UpdateQuantityReq {
    pub quantity: i32,
}

#[axum_macros::debug_handler]
pub async fn update_quantity(
    claims: Claims,
    Path((id, ingredient_id)): Path<(i32, i32)>,
    extract::Json(payload): extract::Json<UpdateQuantityReq>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed updating shopping ingredient");

    validate_shopping_id(id, claims.get_sub(), default_err.clone(), pool).await?;

    sqlx::query!(
        r#"
            UPDATE shopping_quantity AS sq
            SET quantity = $1
            FROM shopping_ingredient AS si
            WHERE sq.shopping_ingredient_id = si.id AND sq.id = $2 AND si.shopping_id = $3
            RETURNING sq.id
        "#,
        payload.quantity,
        ingredient_id,
        id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err)?
    .ok_or((
        StatusCode::NOT_FOUND,
        "Shopping quantity not found".to_string(),
    ))?;

    Ok(StatusCode::OK)
}

#[axum_macros::debug_handler]
pub async fn delete_quantity(
    claims: Claims,
    Path((id, quantity_id)): Path<(i32, i32)>,
    Extension(ref pool): Extension<PgPool>,
) -> Result<StatusCode, (StatusCode, String)> {
    let default_err = get_default_err("Failed deleting shopping quantity");

    validate_shopping_id(id, claims.get_sub(), default_err.clone(), pool).await?;

    let shopping_ingredient = sqlx::query!(
        r#"
          SELECT sq.id, COUNT(*) AS quantities
          FROM shopping_quantity AS sq
          JOIN shopping_ingredient AS si ON sq.shopping_ingredient_id = si.id
          JOIN shopping_quantity AS sq2 ON si.id = sq2.shopping_ingredient_id
          WHERE sq.id = $1 AND si.shopping_id = $2
          GROUP BY sq.id
        "#,
        quantity_id,
        id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())?
    .ok_or((
        StatusCode::NOT_FOUND,
        "Shopping ingredient not found".to_string(),
    ))?;

    sqlx::query!(
        r#"
            DELETE FROM shopping_quantity
            WHERE id = $1
            RETURNING id
        "#,
        shopping_ingredient.id
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())?
    .ok_or((
        StatusCode::NOT_FOUND,
        "Shopping quantity not found".to_string(),
    ))?;

    if shopping_ingredient.quantities.unwrap_or(0) <= 1 {
        sqlx::query!(
            r#"DELETE FROM shopping_ingredient WHERE id = $1"#,
            shopping_ingredient.id
        )
        .execute(pool)
        .await
        .map_err(|_| default_err)?;
    }

    Ok(StatusCode::OK)
}
