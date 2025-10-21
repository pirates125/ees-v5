use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub cookies: Vec<Cookie>,
    pub timestamp: u64,
    pub valid_until: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
}

pub struct SessionManager {
    session_dir: PathBuf,
}

impl SessionManager {
    pub fn new(session_dir: impl Into<PathBuf>) -> Self {
        let session_dir = session_dir.into();
        std::fs::create_dir_all(&session_dir).ok();
        Self { session_dir }
    }
    
    pub fn get_session_file(&self, provider: &str) -> PathBuf {
        self.session_dir.join(format!("{}_session.json", provider))
    }
    
    pub fn load_session(&self, provider: &str) -> Option<SessionData> {
        let file_path = self.get_session_file(provider);
        
        if !file_path.exists() {
            tracing::debug!("Session dosyası bulunamadı: {:?}", file_path);
            return None;
        }
        
        let content = std::fs::read_to_string(&file_path).ok()?;
        let session: SessionData = serde_json::from_str(&content).ok()?;
        
        // Geçerlilik kontrolü
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        if now > session.valid_until {
            tracing::info!("Session süresi dolmuş: {}", provider);
            return None;
        }
        
        tracing::info!("Session yüklendi: {} ({} cookies)", provider, session.cookies.len());
        Some(session)
    }
    
    pub fn save_session(&self, provider: &str, session: SessionData) -> Result<(), std::io::Error> {
        let file_path = self.get_session_file(provider);
        let content = serde_json::to_string_pretty(&session)?;
        std::fs::write(&file_path, content)?;
        
        tracing::info!("Session kaydedildi: {} ({} cookies)", provider, session.cookies.len());
        Ok(())
    }
    
    pub fn clear_session(&self, provider: &str) -> Result<(), std::io::Error> {
        let file_path = self.get_session_file(provider);
        if file_path.exists() {
            std::fs::remove_file(&file_path)?;
            tracing::info!("Session silindi: {}", provider);
        }
        Ok(())
    }
}

