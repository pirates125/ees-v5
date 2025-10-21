use crate::db::models::User;
use crate::db::DbPool;
use sqlx::Row;
use uuid::Uuid;

pub async fn create_user(
    pool: &DbPool,
    email: &str,
    password_hash: &str,
    name: &str,
    role: &str,
) -> Result<User, sqlx::Error> {
    let id = Uuid::new_v4();
    
    sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, email, password_hash, name, role)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(id.to_string())
    .bind(email)
    .bind(password_hash)
    .bind(name)
    .bind(role)
    .fetch_one(pool)
    .await
}

pub async fn get_user_by_id(pool: &DbPool, id: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_user_by_email(pool: &DbPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(pool)
        .await
}

pub async fn list_users(pool: &DbPool, limit: i64, offset: i64) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

pub async fn update_last_login(pool: &DbPool, user_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE users SET last_login = datetime('now') WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn count_users(pool: &DbPool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT COUNT(*) as count FROM users")
        .fetch_one(pool)
        .await?;
    Ok(row.get("count"))
}

pub async fn update_user_profile(
    pool: &DbPool,
    user_id: &str,
    name: Option<String>,
    phone: Option<String>,
) -> Result<User, sqlx::Error> {
    let user = get_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

    let updated_name = name.unwrap_or(user.name);
    let updated_phone = phone.or(user.phone);

    sqlx::query_as::<_, User>(
        r#"
        UPDATE users 
        SET name = $1, phone = $2
        WHERE id = $3
        RETURNING *
        "#,
    )
    .bind(updated_name)
    .bind(updated_phone)
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn change_password(
    pool: &DbPool,
    user_id: &str,
    current_password: &str,
    new_password: &str,
) -> Result<(), sqlx::Error> {
    use crate::auth::{hash_password, verify_password};

    let user = get_user_by_id(pool, user_id)
        .await?
        .ok_or_else(|| sqlx::Error::RowNotFound)?;

    // Verify current password
    if !verify_password(current_password, &user.password_hash)
        .map_err(|_| sqlx::Error::RowNotFound)?
    {
        return Err(sqlx::Error::RowNotFound);
    }

    // Hash new password
    let new_hash = hash_password(new_password).map_err(|_| sqlx::Error::RowNotFound)?;

    // Update password
    sqlx::query("UPDATE users SET password_hash = $1 WHERE id = $2")
        .bind(new_hash)
        .bind(user_id)
        .execute(pool)
        .await?;

    Ok(())
}

