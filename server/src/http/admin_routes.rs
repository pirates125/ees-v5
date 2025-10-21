use crate::auth::Claims;
use crate::db::{logs, models::AdminStats, policies, quotes, users};
use crate::http::{ApiError, AppState};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use serde::Deserialize;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn get_users_handler(
    State(state): State<AppState>,
    _claims: Extension<Claims>,
    Query(params): Query<PaginationQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let user_list = users::list_users(&state.db_pool, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    let total = users::count_users(&state.db_pool)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "users": user_list,
            "total": total,
            "limit": params.limit,
            "offset": params.offset,
        })),
    ))
}

pub async fn get_user_handler(
    State(state): State<AppState>,
    _claims: Extension<Claims>,
    Path(user_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let uid = Uuid::from_str(&user_id)
        .map_err(|_| ApiError::FormValidation("Geçersiz user ID".to_string()))?;
    
    let user = users::get_user_by_id(&state.db_pool, uid)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?
        .ok_or_else(|| ApiError::FormValidation("Kullanıcı bulunamadı".to_string()))?;
    
    Ok((StatusCode::OK, Json(user)))
}

pub async fn get_activity_logs_handler(
    State(state): State<AppState>,
    _claims: Extension<Claims>,
    Query(params): Query<PaginationQuery>,
) -> Result<impl IntoResponse, ApiError> {
    let logs_list = logs::list_activity_logs(&state.db_pool, None, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "logs": logs_list,
            "total": logs_list.len(),
        })),
    ))
}

pub async fn get_admin_stats_handler(
    State(state): State<AppState>,
    _claims: Extension<Claims>,
) -> Result<impl IntoResponse, ApiError> {
    let total_users = users::count_users(&state.db_pool)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    let total_quotes = quotes::count_quotes(&state.db_pool)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    let total_policies = policies::count_policies(&state.db_pool)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    let total_revenue = policies::sum_revenue(&state.db_pool)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    let total_commission = policies::sum_commission(&state.db_pool)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    let stats = AdminStats {
        total_users,
        total_quotes,
        total_policies,
        total_revenue,
        total_commission,
        active_providers: 1, // Sompo
    };
    
    Ok((StatusCode::OK, Json(stats)))
}

