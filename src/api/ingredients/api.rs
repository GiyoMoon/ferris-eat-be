use crate::api::auth::Claims;
use axum::{http::StatusCode, Extension, Json};
use serde::Serialize;
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
