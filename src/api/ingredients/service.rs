use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn update_ingredient_sort(
    id: i32,
    new_sort: i32,
    default_err: (StatusCode, String),
    pool: &PgPool,
) -> Result<(), (StatusCode, String)> {
    sqlx::query!(
        r#"UPDATE ingredient SET sort = $1 WHERE id = $2"#,
        new_sort,
        id
    )
    .execute(pool)
    .await
    .map_err(|_| default_err)
    .map(|_| ())
}

pub struct IngredientSort {
    pub sort: i32,
}

pub async fn get_last_ingredient_by_sort(
    user_id: Uuid,
    default_err: (StatusCode, String),
    pool: &PgPool,
) -> Result<Option<IngredientSort>, (StatusCode, String)> {
    sqlx::query_as!(
        IngredientSort,
        r#"SELECT sort FROM ingredient WHERE user_id = $1 ORDER BY sort DESC LIMIT 1"#,
        user_id
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| default_err.clone())
}
