use crate::browser::session::SessionData;
use crate::config::Config;
use crate::http::ApiError;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::Command;

/// Python subprocess ile Sompo login - %100 garantili
pub async fn login_via_python(config: &Config) -> Result<SessionData, ApiError> {
    tracing::info!("🐍 Python subprocess ile Sompo login başlatılıyor...");
    
    // Python script path
    let script_path = "backend/app/connectors/sompo_session.py";
    
    // Python command
    let output = Command::new("python3")
        .arg(script_path)
        .env("SOMPO_USER", &config.sompo_username)
        .env("SOMPO_PASS", &config.sompo_password)
        .env("SOMPO_SECRET", &config.sompo_secret_key)
        .output()
        .await
        .map_err(|e| {
            tracing::error!("❌ Python subprocess başlatılamadı: {}", e);
            ApiError::WebDriverError(format!("Python subprocess başlatılamadı: {}", e))
        })?;
    
    // Stderr'i logla (Python'dan gelen info mesajları)
    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines() {
            tracing::info!("🐍 Python: {}", line);
        }
    }
    
    // Exit code kontrol
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::LoginFailed(format!(
            "Python login başarısız: {}",
            stderr
        )));
    }
    
    // Stdout'tan JSON parse et
    let stdout = String::from_utf8_lossy(&output.stdout);
    tracing::debug!("🐍 Python output: {}", stdout);
    
    // JSON parse
    #[derive(serde::Deserialize)]
    struct PythonSession {
        cookies: Vec<PythonCookie>,
        local_storage: std::collections::HashMap<String, String>,
        timestamp: u64,
        #[serde(default)]
        url: String,
    }
    
    #[derive(serde::Deserialize)]
    struct PythonCookie {
        name: String,
        value: String,
        domain: String,
        path: String,
        #[serde(default)]
        secure: bool,
        #[serde(default)]
        #[serde(rename = "httpOnly")]
        http_only: bool,
    }
    
    let python_session: PythonSession = serde_json::from_str(&stdout)
        .map_err(|e| {
            tracing::error!("❌ Python session JSON parse hatası: {}", e);
            ApiError::ParseError(format!("Python session parse hatası: {}", e))
        })?;
    
    // Rust SessionData'ya convert et
    let cookies = python_session
        .cookies
        .into_iter()
        .map(|c| crate::browser::session::Cookie {
            name: c.name,
            value: c.value,
            domain: c.domain,
            path: c.path,
            secure: c.secure,
            http_only: c.http_only,
        })
        .collect();
    
    // Session valid until (2 saat)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let session = SessionData {
        cookies,
        local_storage: python_session.local_storage,
        timestamp: python_session.timestamp,
        valid_until: now + 7200, // 2 saat
    };
    
    tracing::info!(
        "✅ Python login başarılı! {} cookies, {} localStorage items",
        session.cookies.len(),
        session.local_storage.len()
    );
    
    Ok(session)
}

