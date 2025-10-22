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
        "--window-size=1920,1080".to_string(),  // Daha yaygın çözünürlük
        format!("--lang={}", config.accept_language.split(',').next().unwrap_or("tr-TR")),
        "--disable-web-security".to_string(),
        "--disable-features=IsolateOrigins,site-per-process".to_string(),
        "--disable-site-isolation-trials".to_string(),
        // Bot detection'a karşı ek ayarlar
        "--disable-features=VizDisplayCompositor".to_string(),
        "--disable-blink-features=AutomationControlled".to_string(),
        "--exclude-switches=enable-automation".to_string(),
        "--disable-infobars".to_string(),
        "--start-maximized".to_string(),
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
    
    // Advanced anti-detection scripts
    let anti_detection_script = format!(
        r#"
        // WebDriver property'sini gizle
        Object.defineProperty(navigator, 'webdriver', {{
            get: () => undefined
        }});
        
        // User Agent override
        Object.defineProperty(navigator, 'userAgent', {{
            get: () => '{}'
        }});
        
        // Chrome runtime'ı ekle (bot detection için)
        window.navigator.chrome = {{
            runtime: {{}}
        }};
        
        // Permissions API
        const originalQuery = window.navigator.permissions.query;
        window.navigator.permissions.query = (parameters) => (
            parameters.name === 'notifications' ?
                Promise.resolve({{ state: Notification.permission }}) :
                originalQuery(parameters)
        );
        
        // Plugin array (gerçek tarayıcı gibi)
        Object.defineProperty(navigator, 'plugins', {{
            get: () => [1, 2, 3, 4, 5]
        }});
        
        // Languages
        Object.defineProperty(navigator, 'languages', {{
            get: () => ['tr-TR', 'tr', 'en-US', 'en']
        }});
        "#,
        config.user_agent
    );
    
    if let Err(e) = client.execute(&anti_detection_script, vec![]).await {
        tracing::warn!("⚠️ Anti-detection script başarısız: {:?}", e);
    } else {
        tracing::info!("✅ Anti-detection script uygulandı");
    }
    
    tracing::info!("WebDriver bağlantısı başarılı");
    
    Ok(client)
}

