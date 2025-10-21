use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub insured: InsuredInfo,
    pub vehicle: VehicleInfo,
    pub coverage: CoverageInfo,
    #[serde(default)]
    pub quote_meta: QuoteMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsuredInfo {
    pub tckn: String,
    pub name: String,
    pub birth_date: String,
    pub phone: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VehicleInfo {
    pub plate: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vin: Option<String>,
    pub brand: String,
    pub model: String,
    pub year: u16,
    pub usage: VehicleUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VehicleUsage {
    Hususi,
    Ticari,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageInfo {
    pub product_type: ProductType,
    pub start_date: String,
    #[serde(default)]
    pub addons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProductType {
    Trafik,
    Kasko,
    Konut,
    Saglik,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteMeta {
    #[serde(default = "generate_uuid")]
    pub request_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
}

fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub request_id: String,
    pub company: String,
    pub product_type: String,
    pub premium: PremiumDetail,
    #[serde(default)]
    pub installments: Vec<Installment>,
    #[serde(default)]
    pub coverages: Vec<Coverage>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw: Option<RawData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timings: Option<Timings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PremiumDetail {
    pub net: f64,
    pub gross: f64,
    pub taxes: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Installment {
    pub count: u8,
    pub per_installment: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coverage {
    pub code: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<String>,
    pub included: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_snapshot_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields_echo: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Timings {
    pub queued_ms: u64,
    pub scrape_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub ok: bool,
    pub version: String,
    pub uptime_seconds: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub supported_products: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvidersResponse {
    pub providers: Vec<ProviderInfo>,
    pub total: usize,
    pub active_count: usize,
}

// User Quotes List Response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserQuoteResponse {
    pub id: String,
    pub provider: String,
    pub premium: f64,
    pub status: String,
    pub created_at: String,
    pub request_data: serde_json::Value,
}

impl From<crate::db::models::Quote> for UserQuoteResponse {
    fn from(q: crate::db::models::Quote) -> Self {
        Self {
            id: q.id,
            provider: q.provider,
            premium: q.premium,
            status: q.status,
            created_at: q.created_at,
            request_data: q.request_data,
        }
    }
}

// User Response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    pub role: String,
}

impl From<crate::db::models::User> for UserResponse {
    fn from(u: crate::db::models::User) -> Self {
        Self {
            id: u.id.to_string(),
            email: u.email,
            name: u.name,
            phone: u.phone,
            role: u.role,
        }
    }
}

