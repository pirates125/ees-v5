use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub http_addr: String,
    pub log_level: String,
    
    // Sompo
    pub sompo_base_url: String,
    pub sompo_username: String,
    pub sompo_password: String,
    pub sompo_secret_key: String,
    
    // Browser
    pub webdriver_url: String,
    pub headless: bool,
    pub proxy_url: Option<String>,
    pub user_agent: String,
    pub accept_language: String,
    pub timezone: String,
    
    // Timeouts
    pub request_timeout_ms: u64,
    pub login_timeout_ms: u64,
    pub retry_max: u32,
    pub retry_backoff_ms: u64,
    
    // Session
    pub session_dir: String,
    
    // Metrics
    pub enable_metrics: bool,
    
    // Database (optional)
    pub database_url: Option<String>,
    
    // JWT
    pub jwt_secret: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();
        
        Ok(Config {
            http_addr: env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8099".to_string()),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            
            sompo_base_url: env::var("SOMPO_BASE_URL")
                .unwrap_or_else(|_| "https://ejento.somposigorta.com.tr/dashboard/login".to_string()),
            sompo_username: env::var("SOMPO_USERNAME").unwrap_or_default(),
            sompo_password: env::var("SOMPO_PASSWORD").unwrap_or_default(),
            sompo_secret_key: env::var("SOMPO_SECRET_KEY").unwrap_or_default(),
            
            webdriver_url: env::var("WEBDRIVER_URL")
                .unwrap_or_else(|_| "http://localhost:9515".to_string()),
            headless: env::var("HEADLESS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            proxy_url: env::var("PROXY_URL").ok().filter(|s| !s.is_empty()),
            user_agent: env::var("USER_AGENT").unwrap_or_else(|_| {
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36".to_string()
            }),
            accept_language: env::var("ACCEPT_LANGUAGE")
                .unwrap_or_else(|_| "tr-TR,tr;q=0.9".to_string()),
            timezone: env::var("TIMEZONE")
                .unwrap_or_else(|_| "Europe/Istanbul".to_string()),
            
            request_timeout_ms: env::var("REQUEST_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(90000),
            login_timeout_ms: env::var("LOGIN_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(45000),
            retry_max: env::var("RETRY_MAX")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(2),
            retry_backoff_ms: env::var("RETRY_BACKOFF_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1500),
            
            session_dir: env::var("SESSION_DIR")
                .unwrap_or_else(|_| "/data/sessions".to_string()),
            
            enable_metrics: env::var("ENABLE_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            
            database_url: env::var("DATABASE_URL").ok(),
            jwt_secret: env::var("JWT_SECRET").ok(),
        })
    }
}

