use crate::api::auth::Claims;
use axum::{http::StatusCode, Extension, Json};
use serde::Serialize;
use sqlx::PgPool;

#[derive(Serialize)]
pub struct UnitGetRes {
    id: i32,
    name: String,
}

#[axum_macros::debug_handler]
pub async fn get_all(
    _: Claims,
    Extension(ref pool): Extension<PgPool>,
) -> Result<(StatusCode, Json<Vec<UnitGetRes>>), (StatusCode, String)> {
    let units = sqlx::query!(r#"SELECT id, name FROM unit"#)
        .fetch_all(pool)
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed getting units".to_string(),
            )
        })?;

    Ok((
        StatusCode::OK,
        Json(
            units
                .into_iter()
                .map(|record| UnitGetRes {
                    id: record.id,
                    name: record.name,
                })
                .collect(),
        ),
    ))
}
