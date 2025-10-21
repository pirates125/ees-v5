use crate::db::models::ActivityLog;
use crate::db::DbPool;
use uuid::Uuid;

pub async fn log_activity(
    pool: &DbPool,
    user_id: Uuid,
    action: &str,
    entity_type: Option<&str>,
    entity_id: Option<Uuid>,
    metadata: Option<serde_json::Value>,
    ip_address: Option<&str>,
) -> Result<ActivityLog, sqlx::Error> {
    sqlx::query_as::<_, ActivityLog>(
        r#"
        INSERT INTO activity_logs (user_id, action, entity_type, entity_id, metadata, ip_address)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(metadata)
    .bind(ip_address)
    .fetch_one(pool)
    .await
}

pub async fn list_activity_logs(
    pool: &DbPool,
    user_id: Option<Uuid>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActivityLog>, sqlx::Error> {
    if let Some(uid) = user_id {
        sqlx::query_as::<_, ActivityLog>(
            r#"
            SELECT * FROM activity_logs
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(uid)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, ActivityLog>(
            r#"
            SELECT * FROM activity_logs
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}

