use crate::config::Config;
use fantoccini::{Client, ClientBuilder};
use serde_json::json;

pub async fn create_webdriver_client(config: &Config) -> Result<Client, fantoccini::error::NewSessionError> {
    let mut caps = serde_json::Map::new();
    
    // Chrome options
    let mut chrome_opts = serde_json::Map::new();
    
    let mut args = vec![
        "--disable-blink-features=AutomationControlled".to_string(),
        "--no-sandbox".to_string(),
        "--disable-dev-shm-usage".to_string(),
        "--disable-gpu".to_string(),
        "--window-size=1366,768".to_string(),
        format!("--lang={}", config.accept_language.split(',').next().unwrap_or("tr-TR")),
    ];
    
    if config.headless {
        args.push("--headless".to_string());
        args.push("--disable-software-rasterizer".to_string());
    }
    
    // Proxy desteği
    if let Some(proxy_url) = &config.proxy_url {
        args.push(format!("--proxy-server={}", proxy_url));
    }
    
    chrome_opts.insert("args".to_string(), json!(args));
    
    // User agent
    chrome_opts.insert("excludeSwitches".to_string(), json!(["enable-automation"]));
    
    // Experimental options to hide automation
    let mut prefs = serde_json::Map::new();
    prefs.insert("credentials_enable_service".to_string(), json!(false));
    prefs.insert("profile.password_manager_enabled".to_string(), json!(false));
    chrome_opts.insert("prefs".to_string(), json!(prefs));
    
    caps.insert("goog:chromeOptions".to_string(), json!(chrome_opts));
    
    // Standard capabilities
    caps.insert("browserName".to_string(), json!("chrome"));
    caps.insert("acceptInsecureCerts".to_string(), json!(true));
    
    tracing::info!("WebDriver bağlantısı oluşturuluyor: {}", config.webdriver_url);
    
    let client = ClientBuilder::native()
        .capabilities(caps)
        .connect(&config.webdriver_url)
        .await?;
    
    // User agent override
    let user_agent_script = format!(
        r#"
        Object.defineProperty(navigator, 'webdriver', {{
            get: () => undefined
        }});
        Object.defineProperty(navigator, 'userAgent', {{
            get: () => '{}'
        }});
        "#,
        config.user_agent
    );
    
    if let Err(e) = client.execute(&user_agent_script, vec![]).await {
        tracing::warn!("User agent override başarısız: {:?}", e);
    }
    
    tracing::info!("WebDriver bağlantısı başarılı");
    
    Ok(client)
}

