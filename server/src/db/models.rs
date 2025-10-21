use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub name: String,
    pub phone: Option<String>,
    pub role: String,
    pub created_at: String,
    pub last_login: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Quote {
    pub id: String,
    pub user_id: String,
    pub request_id: String,
    pub request_data: serde_json::Value,
    pub provider: String,
    pub premium: f64,  // SQLite uses REAL (f64) instead of Decimal
    pub response_data: serde_json::Value,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Policy {
    pub id: String,
    pub user_id: String,
    pub quote_id: Option<String>,
    pub policy_number: String,
    pub provider: String,
    pub product_type: String,
    pub premium: f64,  // SQLite uses REAL (f64) instead of Decimal
    pub commission: Option<f64>,  // SQLite uses REAL (f64) instead of Decimal
    pub status: String,
    pub policy_data: serde_json::Value,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub pdf_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivityLog {
    pub id: String,
    pub user_id: String,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub total_quotes: i64,
    pub total_policies: i64,
    pub total_revenue: f64,  // SQLite uses REAL (f64) instead of Decimal
    pub total_commission: f64,  // SQLite uses REAL (f64) instead of Decimal
    pub active_providers: i64,
}

