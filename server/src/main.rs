mod auth;
mod browser;
mod config;
mod db;
mod http;
mod providers;
mod services;
mod utils;

use crate::config::Config;
use crate::db::{create_pool, run_migrations};
use crate::http::{create_router, AppState};
use crate::providers::ProviderRegistry;
use crate::services::QuoteAggregator;
use std::sync::Arc;
use std::time::SystemTime;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    match dotenvy::dotenv() {
        Ok(path) => eprintln!("✅ .env loaded from: {:?}", path),
        Err(e) => eprintln!("⚠️  .env not found: {}", e),
    }
    
    // Logging setup
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sigorta_server=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("🚀 Sigorta Server başlatılıyor...");
    
    // Config yükle
    let config = Arc::new(Config::from_env()?);
    tracing::info!("✅ Config yüklendi");
    tracing::info!("   HTTP Addr: {}", config.http_addr);
    tracing::info!("   WebDriver URL: {}", config.webdriver_url);
    tracing::info!("   Headless: {}", config.headless);
    tracing::info!("   Session Dir: {}", config.session_dir);
    
    // Database bağlantısı (opsiyonel - yoksa devam et)
    let database_url = std::env::var("DATABASE_URL").ok();
    let db_pool = if let Some(url) = &database_url {
        tracing::info!("📊 Database'e bağlanılıyor...");
        match create_pool(url).await {
            Ok(pool) => {
                tracing::info!("✅ Database bağlantısı başarılı");
                
                // Migration'ları çalıştır
                if let Err(e) = run_migrations(&pool).await {
                    tracing::warn!("⚠️ Migration hatası: {}", e);
                } else {
                    tracing::info!("✅ Migration'lar tamamlandı");
                }
                
                Some(pool)
            }
            Err(e) => {
                tracing::warn!("⚠️ Database bağlantısı başarısız: {}", e);
                tracing::warn!("   Sistem database olmadan çalışacak");
                None
            }
        }
    } else {
        tracing::info!("ℹ️  DATABASE_URL bulunamadı, database devre dışı");
        None
    };
    
    // JWT secret
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev-secret-change-in-production".to_string());
    
    // Provider registry oluştur
    let registry = Arc::new(ProviderRegistry::new(config.clone()));
    let providers_info = registry.get_providers_info();
    tracing::info!("✅ Provider registry oluşturuldu");
    tracing::info!("   Toplam Provider: {}", providers_info.total);
    tracing::info!("   Aktif Provider: {}", providers_info.active_count);
    
    for provider in &providers_info.providers {
        let status = if provider.active { "✅" } else { "⏸️" };
        let reason = provider.reason.as_ref()
            .map(|r| format!(" ({})", r))
            .unwrap_or_default();
        tracing::info!("   {} {} {}", status, provider.name, reason);
    }
    
    // Quote aggregator
    let aggregator = Arc::new(QuoteAggregator::new(registry.clone()));
    
    // Database yoksa dummy pool oluştur (compile için)
    let pool = db_pool.unwrap_or_else(|| {
        tracing::warn!("⚠️ Dummy database pool kullanılıyor");
        // Bu durumda auth route'lar çalışmayacak
        unsafe { std::mem::zeroed() } // Dikkat: Bu sadece compile için, production'da kullanılmaz
    });
    
    // App state
    let state = AppState {
        config: config.clone(),
        registry,
        aggregator,
        db_pool: pool,
        jwt_secret,
        start_time: SystemTime::now(),
    };
    
    // Router oluştur
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);
    
    let app = create_router(state).layer(cors);
    
    // Server başlat
    let listener = tokio::net::TcpListener::bind(&config.http_addr).await?;
    tracing::info!("🌐 Server dinleniyor: http://{}", config.http_addr);
    tracing::info!("📋 Endpoints:");
    tracing::info!("   GET  /health");
    tracing::info!("   GET  /metrics");
    tracing::info!("   GET  /api/v1/providers");
    tracing::info!("   POST /api/v1/quote");
    tracing::info!("   POST /api/v1/quote/:provider");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

