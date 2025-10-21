use crate::browser::session::SessionManager;
use crate::config::Config;
use crate::http::ApiError;
use crate::providers::quick::selectors::QuickSelectors;
use fantoccini::{Client, Locator};
use std::sync::Arc;

pub async fn login_to_quick(
    client: &Client,
    _config: Arc<Config>,
    _session_manager: &SessionManager,
) -> Result<(), ApiError> {
    let username = std::env::var("QUICK_USERNAME").unwrap_or_default();
    let password = std::env::var("QUICK_PASSWORD").unwrap_or_default();
    let url = std::env::var("QUICK_URL")
        .unwrap_or_else(|_| "https://www.quicksigorta.com.tr/agent/login".to_string());
    
    tracing::info!("🔍 Quick'e bağlanılıyor: {}", url);
    
    client.goto(&url).await
        .map_err(|e| ApiError::WebDriverError(format!("Sayfa yüklenemedi: {}", e)))?;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    // Username
    for selector in QuickSelectors::USERNAME_INPUTS {
        if let Ok(elem) = client.find(Locator::Css(selector)).await {
            if let Ok(_) = elem.send_keys(&username).await {
                tracing::info!("✅ Quick username dolduruldu");
                break;
            }
        }
    }
    
    // Password
    for selector in QuickSelectors::PASSWORD_INPUTS {
        if let Ok(elem) = client.find(Locator::Css(selector)).await {
            if let Ok(_) = elem.send_keys(&password).await {
                tracing::info!("✅ Quick password dolduruldu");
                break;
            }
        }
    }
    
    // Login button
    for selector in QuickSelectors::LOGIN_BUTTONS {
        if let Ok(elem) = client.find(Locator::Css(selector)).await {
            if let Ok(_) = elem.click().await {
                tracing::info!("✅ Quick login butonu tıklandı");
                break;
            }
        }
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
    
    tracing::info!("✅ Quick login tamamlandı");
    
    Ok(())
}

