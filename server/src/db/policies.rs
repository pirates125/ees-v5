use crate::db::models::Policy;
use crate::db::DbPool;
use sqlx::Row;
use uuid::Uuid;

pub async fn create_policy(
    pool: &DbPool,
    user_id: &str,
    quote_id: Option<&str>,
    policy_number: &str,
    provider: &str,
    product_type: &str,
    premium: f64,
    commission: Option<f64>,
    policy_data: serde_json::Value,
    expires_at: Option<String>,
) -> Result<Policy, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    
    sqlx::query_as::<_, Policy>(
        r#"
        INSERT INTO policies 
        (id, user_id, quote_id, policy_number, provider, product_type, premium, commission, policy_data, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
    )
    .bind(id)
    .bind(user_id)
    .bind(quote_id)
    .bind(policy_number)
    .bind(provider)
    .bind(product_type)
    .bind(premium)
    .bind(commission)
    .bind(policy_data)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

pub async fn get_policy_by_id(pool: &DbPool, id: &str) -> Result<Option<Policy>, sqlx::Error> {
    sqlx::query_as::<_, Policy>("SELECT * FROM policies WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn list_policies_by_user(
    pool: &DbPool,
    user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<Policy>, sqlx::Error> {
    sqlx::query_as::<_, Policy>(
        r#"
        SELECT * FROM policies
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

pub async fn count_policies(pool: &DbPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM policies")
        .fetch_one(pool)
        .await?;
    Ok(row.get("count"))
}

pub async fn sum_revenue(pool: &DbPool) -> Result<f64, sqlx::Error> {
    let row = sqlx::query("SELECT COALESCE(SUM(premium), 0.0) as total FROM policies WHERE status = 'active'")
        .fetch_one(pool)
        .await?;
    Ok(row.get("total"))
}

pub async fn sum_commission(pool: &DbPool) -> Result<f64, sqlx::Error> {
    let row = sqlx::query("SELECT COALESCE(SUM(commission), 0.0) as total FROM policies WHERE status = 'active'")
        .fetch_one(pool)
        .await?;
    Ok(row.get("total"))
}

