use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub phone: Option<String>,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Quote {
    pub id: Uuid,
    pub user_id: Uuid,
    pub request_id: String,
    pub request_data: serde_json::Value,
    pub provider: String,
    pub premium: rust_decimal::Decimal,
    pub response_data: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Policy {
    pub id: Uuid,
    pub user_id: Uuid,
    pub quote_id: Option<Uuid>,
    pub policy_number: String,
    pub provider: String,
    pub product_type: String,
    pub premium: rust_decimal::Decimal,
    pub commission: Option<rust_decimal::Decimal>,
    pub status: String,
    pub policy_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub pdf_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivityLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub total_quotes: i64,
    pub total_policies: i64,
    pub total_revenue: rust_decimal::Decimal,
    pub total_commission: rust_decimal::Decimal,
    pub active_providers: i64,
}

