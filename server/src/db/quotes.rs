use crate::db::models::Quote;
use crate::db::DbPool;
use sqlx::Row;
use uuid::Uuid;

pub async fn create_quote(
    pool: &DbPool,
    user_id: Uuid,
    request_id: &str,
    request_data: serde_json::Value,
    provider: &str,
    premium: f64,
    response_data: serde_json::Value,
) -> Result<Quote, sqlx::Error> {
    sqlx::query_as::<_, Quote>(
        r#"
        INSERT INTO quotes (user_id, request_id, request_data, provider, premium, response_data, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'completed')
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(request_id)
    .bind(request_data)
    .bind(provider)
    .bind(premium)
    .bind(response_data)
    .fetch_one(pool)
    .await
}

pub async fn get_quote_by_id(pool: &DbPool, id: Uuid) -> Result<Option<Quote>, sqlx::Error> {
    sqlx::query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_quotes_by_user(
    pool: &DbPool,
    user_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<Quote>, sqlx::Error> {
    sqlx::query_as::<_, Quote>(
        r#"
        SELECT * FROM quotes
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn count_quotes(pool: &DbPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM quotes")
        .fetch_one(pool)
        .await?;
    Ok(row.get("count"))
}

pub async fn count_quotes_by_user(pool: &DbPool, user_id: Uuid) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM quotes WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    Ok(row.get("count"))
}

// Alias for consistency with API naming
pub async fn list_user_quotes(pool: &DbPool, user_id: Uuid) -> Result<Vec<Quote>, sqlx::Error> {
    list_quotes_by_user(pool, user_id, 100, 0).await
}

