use crate::auth::{admin_middleware, auth_middleware, Claims};
use crate::db::{logs, policies, quotes};
use crate::http::admin_routes::{
    get_activity_logs_handler, get_admin_stats_handler, get_user_handler, get_users_handler,
};
use crate::http::auth_routes::{login_handler, register_handler};
use crate::http::user_routes::{change_password_handler, update_profile_handler};
use crate::http::{ApiError, AppState, HealthResponse, QuoteRequest};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use std::str::FromStr;
use std::time::SystemTime;
use uuid::Uuid;

pub fn create_router(state: AppState) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .route("/api/v1/auth/login", post(login_handler))
        .route("/api/v1/auth/register", post(register_handler));
    
    // Protected routes (auth required)
    let protected_routes = Router::new()
        .route("/api/v1/providers", get(list_providers_handler))
        .route("/api/v1/quote", post(quote_all_handler))
        .route("/api/v1/quote/:provider", post(quote_single_handler))
        .route("/api/v1/quotes/compare", post(compare_quotes_handler))
        .route("/api/v1/quotes", get(list_user_quotes_handler))
        .route("/api/v1/policies", post(create_policy_handler))
        .route("/api/v1/policies", get(list_user_policies_handler))
        .route("/api/v1/users/profile", axum::routing::put(update_profile_handler))
        .route("/api/v1/users/password", axum::routing::put(change_password_handler))
        .layer(middleware::from_fn_with_state(
            state.jwt_secret.clone(),
            auth_middleware,
        ))
        .with_state(state.clone());
    
    // Admin routes
    let admin_routes = Router::new()
        .route("/api/v1/admin/users", get(get_users_handler))
        .route("/api/v1/admin/users/:id", get(get_user_handler))
        .route("/api/v1/admin/logs", get(get_activity_logs_handler))
        .route("/api/v1/admin/stats", get(get_admin_stats_handler))
        .layer(middleware::from_fn(admin_middleware))
        .layer(middleware::from_fn_with_state(
            state.jwt_secret.clone(),
            auth_middleware,
        ));
    
    // Tüm route'ları birleştir ve state ekle
    public_routes
        .merge(protected_routes)
        .merge(admin_routes)
        .with_state(state)
}

async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = SystemTime::now()
        .duration_since(state.start_time)
        .unwrap()
        .as_secs();
    
    let response = HealthResponse {
        ok: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        timestamp: Utc::now(),
    };
    
    (StatusCode::OK, Json(response))
}

async fn metrics_handler() -> impl IntoResponse {
    // Basit Prometheus formatı
    let metrics = format!(
        "# HELP sigorta_server_up Server aktif mi\n\
         # TYPE sigorta_server_up gauge\n\
         sigorta_server_up 1\n\
         # HELP sigorta_requests_total Toplam istek sayısı\n\
         # TYPE sigorta_requests_total counter\n\
         sigorta_requests_total 0\n"
    );
    
    (StatusCode::OK, metrics)
}

async fn list_providers_handler(State(state): State<AppState>) -> impl IntoResponse {
    let providers_info = state.registry.get_providers_info();
    (StatusCode::OK, Json(providers_info))
}

async fn quote_all_handler(
    State(state): State<AppState>,
    Json(request): Json<QuoteRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!("📥 Tüm provider'lardan teklif istendi: request_id={}", request.quote_meta.request_id);
    
    let active_providers = state.registry.get_active_providers();
    
    if active_providers.is_empty() {
        return Err(ApiError::ProviderInactive(
            "Hiç aktif provider yok".to_string()
        ));
    }
    
    let mut quotes = Vec::new();
    let mut errors = Vec::new();
    
    // Her provider'dan sırayla teklif al (paralel yapmak için tokio::spawn kullanılabilir)
    for provider in active_providers {
        tracing::info!("🔄 Provider: {} - teklif alınıyor...", provider.name());
        
        match provider.fetch_quote(request.clone()).await {
            Ok(quote) => {
                tracing::info!("✅ {} - Teklif başarılı: {} TRY", provider.name(), quote.premium.gross);
                quotes.push(quote);
            }
            Err(e) => {
                tracing::error!("❌ {} - Hata: {}", provider.name(), e);
                errors.push(serde_json::json!({
                    "provider": provider.name(),
                    "error": e.to_string()
                }));
            }
        }
    }
    
    if quotes.is_empty() {
        return Err(ApiError::Unknown(
            format!("Hiç teklif alınamadı. Hatalar: {:?}", errors)
        ));
    }
    
    // İlk teklifi dön (multi-quote response için başka model gerekir)
    Ok((StatusCode::OK, Json(quotes)))
}

async fn quote_single_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(provider_name): Path<String>,
    Json(request): Json<QuoteRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!("📥 {} provider'dan teklif istendi: request_id={}", provider_name, request.quote_meta.request_id);
    
    let provider = state
        .registry
        .get_provider(&provider_name)
        .ok_or_else(|| ApiError::ProviderInactive(format!("Provider bulunamadı: {}", provider_name)))?;
    
    if !provider.is_active() {
        return Err(ApiError::ProviderInactive(format!(
            "{} aktif değil: {}",
            provider.name(),
            provider.inactive_reason().unwrap_or_default()
        )));
    }
    
    let quote = provider.fetch_quote(request.clone()).await?;
    
    // Database'e kaydet
    let _ = quotes::create_quote(
        &state.db_pool,
        &claims.sub,
        &quote.request_id,
        serde_json::to_value(&request).unwrap_or_default(),
        &quote.company,
        quote.premium.gross,
        serde_json::to_value(&quote).unwrap_or_default(),
    )
    .await;
    
    tracing::info!("✅ {} - Teklif başarılı: {} TRY", provider.name(), quote.premium.gross);
    
    Ok((StatusCode::OK, Json(quote)))
}

async fn compare_quotes_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<QuoteRequest>,
) -> Result<impl IntoResponse, ApiError> {
    tracing::info!("🔍 Tüm provider'lardan karşılaştırmalı teklif istendi");
    
    let quotes = state.aggregator.fetch_all_quotes(request.clone()).await?;
    
    // Database'e kaydet
    for quote in &quotes {
        let _ = quotes::create_quote(
            &state.db_pool,
            &claims.sub,
            &quote.request_id,
            serde_json::to_value(&request).unwrap_or_default(),
            &quote.company,
            quote.premium.gross,
            serde_json::to_value(&quote).unwrap_or_default(),
        )
        .await;
    }
    
    Ok((StatusCode::OK, Json(quotes)))
}

#[derive(Debug, Deserialize)]
struct PaginationParams {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    20
}

async fn list_user_quotes_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApiError> {
    let quotes_list = quotes::list_quotes_by_user(&state.db_pool, &claims.sub, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    let total = quotes::count_quotes_by_user(&state.db_pool, &claims.sub)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "quotes": quotes_list,
            "total": total,
        })),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatePolicyRequest {
    quote_id: String,
    payment_method: String,
    installment_count: u8,
}

async fn create_policy_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreatePolicyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    // Quote'u bul
    let quote = quotes::get_quote_by_id(&state.db_pool, &req.quote_id)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?
        .ok_or_else(|| ApiError::FormValidation("Quote bulunamadı".to_string()))?;
    
    // Poliçe numarası oluştur
    let policy_number = format!("POL-{}-{}", chrono::Utc::now().timestamp(), uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
    
    // Komisyon hesapla (%10 varsayım)
    let commission = quote.premium * 0.10;
    
    let policy = policies::create_policy(
        &state.db_pool,
        &claims.sub,
        Some(&req.quote_id),
        &policy_number,
        &quote.provider,
        "trafik", // quote.response_data'dan alınabilir
        quote.premium,
        Some(commission),
        quote.response_data.clone(),
        None,
    )
    .await
    .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    // Activity log
    let _ = logs::log_activity(
        &state.db_pool,
        &claims.sub,
        "policy_created",
        Some("policy"),
        Some(policy.id.clone()),
        Some(serde_json::json!({
            "provider": policy.provider,
            "premium": policy.premium,
        })),
        None,
    )
    .await;
    
    Ok((StatusCode::CREATED, Json(policy)))
}

async fn list_user_policies_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, ApiError> {
    let policies_list = policies::list_policies_by_user(&state.db_pool, &claims.sub, params.limit, params.offset)
        .await
        .map_err(|e| ApiError::Unknown(e.to_string()))?;
    
    Ok((StatusCode::OK, Json(policies_list)))
}

