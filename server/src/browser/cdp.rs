use crate::config::Config;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::error::CdpError;
use chromiumoxide::Page;
use futures::StreamExt;
use std::time::Duration;

/// CDP Browser oluştur (Playwright benzeri)
pub async fn create_cdp_browser(config: &Config) -> Result<Browser, CdpError> {
    tracing::info!("🚀 CDP Browser başlatılıyor...");
    
    // Chrome binary path
    let chrome_path = std::env::var("CHROME_PATH")
        .unwrap_or_else(|_| {
            // Default paths
            if cfg!(target_os = "windows") {
                "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string()
            } else if cfg!(target_os = "macos") {
                "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome".to_string()
            } else {
                "google-chrome".to_string()
            }
        });
    
    tracing::info!("🔍 Chrome path: {}", chrome_path);
    
    // Browser config
    let mut args = vec![
        "--disable-blink-features=AutomationControlled",
        "--no-sandbox",
        "--disable-dev-shm-usage",
        "--disable-gpu",
        "--window-size=1920,1080",
        "--disable-web-security",
        "--disable-features=IsolateOrigins,site-per-process",
        "--disable-site-isolation-trials",
        "--exclude-switches=enable-automation",
        "--disable-infobars",
    ];
    
    if config.headless {
        args.push("--headless=new");  // New headless mode
    }
    
    let proxy_arg;
    if let Some(proxy_url) = &config.proxy_url {
        proxy_arg = format!("--proxy-server={}", proxy_url);
        args.push(&proxy_arg);
    }
    
    let config_builder = BrowserConfig::builder()
        .chrome_executable(chrome_path)
        .window_size(1920, 1080)
        .args(args);
    
    let (browser, mut handler) = Browser::launch(config_builder.build().map_err(|e| {
        CdpError::Io(std::io::Error::new(std::io::ErrorKind::Other, format!("BrowserConfig build error: {}", e)))
    })?).await?;
    
    // Handler'ı background'da çalıştır (CRITICAL!)
    tokio::spawn(async move {
        while let Some(event) = handler.next().await {
            if let Err(e) = event {
                tracing::warn!("CDP event error: {:?}", e);
            }
        }
        tracing::info!("CDP handler sonlandı");
    });
    
    tracing::info!("✅ CDP Browser başlatıldı");
    
    Ok(browser)
}

/// Anti-detection script inject et
pub async fn inject_anti_detection(page: &Page) -> Result<(), CdpError> {
    let script = r#"
        // navigator.webdriver silme
        Object.defineProperty(navigator, 'webdriver', {
            get: () => false,
        });
        
        // Chrome object
        window.navigator.chrome = {
            runtime: {},
        };
        
        // Permissions API
        const originalQuery = window.navigator.permissions.query;
        window.navigator.permissions.query = (parameters) => (
            parameters.name === 'notifications' ?
                Promise.resolve({ state: Notification.permission }) :
                originalQuery(parameters)
        );
        
        // Plugins
        Object.defineProperty(navigator, 'plugins', {
            get: () => [1, 2, 3, 4, 5],
        });
        
        // Languages
        Object.defineProperty(navigator, 'languages', {
            get: () => ['tr-TR', 'tr', 'en-US', 'en'],
        });
    "#;
    
    page.evaluate(script).await?;
    tracing::debug!("✅ Anti-detection script injected");
    
    Ok(())
}

/// Wait for navigation (Playwright benzeri)
pub async fn wait_for_navigation(
    page: &Page,
    timeout_secs: u64,
) -> Result<(), CdpError> {
    let start_url = page.url().await.ok().flatten().unwrap_or_default();
    
    for _i in 0..(timeout_secs * 4) {
        tokio::time::sleep(Duration::from_millis(250)).await;
        
        if let Ok(Some(current_url)) = page.url().await {
            if current_url != start_url {
                tracing::info!("✅ Navigation: {} -> {}", start_url, current_url);
                return Ok(());
            }
        }
    }
    
    tracing::warn!("⚠️ Navigation timeout: {} saniye", timeout_secs);
    Ok(())  // Timeout olsa bile devam et
}

/// Network idle bekle
pub async fn wait_for_network_idle(
    page: &Page,
    timeout_secs: u64,
) -> Result<(), CdpError> {
    tracing::info!("⏳ Network idle bekleniyor...");
    
    for _i in 0..(timeout_secs * 2) {
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        let js = r#"
            return {
                readyState: document.readyState,
                activeRequests: performance.getEntriesByType('resource')
                    .filter(r => !r.responseEnd).length
            };
        "#;
        
        if let Ok(result) = page.evaluate(js).await {
            if let Ok(value) = result.into_value::<serde_json::Value>() {
                if let Some(obj_map) = value.as_object() {
                    let ready_state = obj_map.get("readyState").and_then(|v| v.as_str()).unwrap_or("");
                    let active_reqs = obj_map.get("activeRequests").and_then(|v| v.as_u64()).unwrap_or(999);
                    
                    if ready_state == "complete" && active_reqs == 0 {
                        tracing::info!("✅ Network idle! ({}.5 saniye)", _i / 2);
                        return Ok(());
                    }
                }
            }
        }
    }
    
    tracing::warn!("⚠️ Network idle timeout: {} saniye", timeout_secs);
    Ok(())
}

