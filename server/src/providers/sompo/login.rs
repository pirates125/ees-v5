use crate::browser::session::{Cookie, SessionData, SessionManager};
use crate::config::Config;
use crate::http::ApiError;
use crate::providers::sompo::selectors::SompoSelectors;
use crate::utils::mask_sensitive;
use fantoccini::{Client, Locator};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn login_to_sompo(
    client: &Client,
    config: Arc<Config>,
    session_manager: &SessionManager,
) -> Result<(), ApiError> {
    tracing::info!("🔍 Sompo'ya bağlanılıyor: {}", config.sompo_base_url);
    tracing::info!("👤 Kullanıcı: {}", mask_sensitive(&config.sompo_username));
    
    // Önce session cache'i kontrol et
    if let Some(session) = session_manager.load_session("sompo") {
        tracing::info!("📦 Cached session bulundu, yükleniyor...");
        
        // Session'ı yükle
        if let Err(e) = restore_session(client, &session, &config.sompo_base_url).await {
            tracing::warn!("⚠️ Session restore başarısız: {}, yeniden login...", e);
            session_manager.clear_session("sompo").ok();
        } else {
            // Session başarıyla yüklendi, dashboard'da mıyız kontrol et
            if is_logged_in(client).await {
                tracing::info!("✅ Session geçerli, login atlandı");
                return Ok(());
            } else {
                tracing::warn!("⚠️ Session geçersiz, yeniden login...");
                session_manager.clear_session("sompo").ok();
            }
        }
    }
    
    // Login sayfasına git
    client
        .goto(&config.sompo_base_url)
        .await
        .map_err(|e| ApiError::WebDriverError(format!("Sayfa yüklenemedi: {}", e)))?;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    tracing::info!("✅ Sompo sayfası yüklendi");
    
    // Önce spesifik XPath'i dene (Python'dan gelen)
    let mut username_filled = false;
    if let Ok(elem) = client.find(Locator::XPath(SompoSelectors::USERNAME_XPATH)).await {
        if let Ok(_) = elem.send_keys(&config.sompo_username).await {
            tracing::info!("✅ Username dolduruldu (XPath)");
            username_filled = true;
        }
    }
    
    // Başarısız olduysa CSS selector'ları dene
    if !username_filled {
        username_filled = try_fill_input(client, SompoSelectors::USERNAME_INPUTS, &config.sompo_username).await?;
        if !username_filled {
            return Err(ApiError::LoginFailed("Username input bulunamadı".to_string()));
        }
        tracing::info!("✅ Username dolduruldu (CSS)");
    }
    
    // Password için aynı strateji
    let mut password_filled = false;
    if let Ok(elem) = client.find(Locator::XPath(SompoSelectors::PASSWORD_XPATH)).await {
        if let Ok(_) = elem.send_keys(&config.sompo_password).await {
            tracing::info!("✅ Password dolduruldu (XPath)");
            password_filled = true;
        }
    }
    
    if !password_filled {
        password_filled = try_fill_input(client, SompoSelectors::PASSWORD_INPUTS, &config.sompo_password).await?;
        if !password_filled {
            return Err(ApiError::LoginFailed("Password input bulunamadı".to_string()));
        }
        tracing::info!("✅ Password dolduruldu (CSS)");
    }
    
    // Login butonuna tıkla
    let login_clicked = try_click_button(client, SompoSelectors::LOGIN_BUTTONS).await?;
    if !login_clicked {
        return Err(ApiError::LoginFailed("Login butonu bulunamadı".to_string()));
    }
    tracing::info!("✅ Login butonu tıklandı");
    
    // Login işleminin tamamlanmasını bekle
    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
    
    // OTP kontrolü
    if let Ok(otp_found) = check_otp_required(client).await {
        if otp_found {
            tracing::info!("🔐 OTP ekranı tespit edildi");
            handle_otp(client, &config.sompo_secret_key).await?;
        }
    }
    
    // Login başarısını doğrula
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    let current_url = client.current_url().await
        .map_err(|e| ApiError::WebDriverError(format!("URL alınamadı: {}", e)))?;
    
    tracing::info!("📍 Mevcut URL: {}", current_url);
    
    // Hala login sayfasındaysak hata
    if current_url.as_str().to_lowercase().contains("login") {
        return Err(ApiError::LoginFailed("Login başarısız - hala login sayfasında".to_string()));
    }
    
    // Dashboard göstergelerini kontrol et
    if !is_logged_in(client).await {
        return Err(ApiError::LoginFailed("Login doğrulanamadı".to_string()));
    }
    
    tracing::info!("✅ Login başarılı!");
    
    // Session'ı kaydet
    save_current_session(client, session_manager).await?;
    
    Ok(())
}

async fn try_fill_input(client: &Client, selectors: &[&str], value: &str) -> Result<bool, ApiError> {
    for selector in selectors {
        match client.find(Locator::Css(selector)).await {
            Ok(elem) => {
                elem.send_keys(value).await
                    .map_err(|e| ApiError::WebDriverError(e.to_string()))?;
                return Ok(true);
            }
            Err(_) => continue,
        }
    }
    Ok(false)
}

async fn try_click_button(client: &Client, selectors: &[&str]) -> Result<bool, ApiError> {
    for selector in selectors {
        match client.find(Locator::Css(selector)).await {
            Ok(elem) => {
                elem.click().await
                    .map_err(|e| ApiError::WebDriverError(e.to_string()))?;
                return Ok(true);
            }
            Err(_) => continue,
        }
    }
    Ok(false)
}

async fn check_otp_required(client: &Client) -> Result<bool, ApiError> {
    for selector in SompoSelectors::OTP_INPUTS {
        if client.find(Locator::Css(selector)).await.is_ok() {
            return Ok(true);
        }
    }
    Ok(false)
}

async fn handle_otp(client: &Client, secret_key: &str) -> Result<(), ApiError> {
    if secret_key.is_empty() {
        return Err(ApiError::HumanActionRequired(
            "SOMPO_SECRET_KEY yapılandırılmamış - Manuel OTP girişi gerekli".to_string()
        ));
    }
    
    // TOTP kodu üret
    let totp = totp_lite::totp_custom::<totp_lite::Sha1>(30, 6, secret_key.as_bytes(), 0);
    tracing::info!("🔢 OTP kodu üretildi: {}", totp);
    
    // OTP input'una kodu gir
    let otp_filled = try_fill_input(client, SompoSelectors::OTP_INPUTS, &totp).await?;
    if !otp_filled {
        return Err(ApiError::HumanActionRequired("OTP input bulunamadı".to_string()));
    }
    tracing::info!("✅ OTP kodu girildi");
    
    // OTP submit butonu
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    let otp_submitted = try_click_button(client, SompoSelectors::OTP_SUBMIT_BUTTONS).await?;
    if otp_submitted {
        tracing::info!("✅ OTP submit edildi");
    }
    
    // OTP doğrulamasının tamamlanmasını bekle
    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
    
    Ok(())
}

async fn is_logged_in(client: &Client) -> bool {
    // Dashboard göstergelerini kontrol et
    for selector in SompoSelectors::DASHBOARD_INDICATORS {
        if client.find(Locator::Css(selector)).await.is_ok() {
            return true;
        }
    }
    false
}

async fn save_current_session(client: &Client, session_manager: &SessionManager) -> Result<(), ApiError> {
    // Fantoccini'den cookie'leri al
    let cookies_raw = client.get_all_cookies().await
        .map_err(|e| ApiError::WebDriverError(format!("Cookie alınamadı: {}", e)))?;
    
    let cookies: Vec<Cookie> = cookies_raw
        .into_iter()
        .map(|c| Cookie {
            name: c.name().to_string(),
            value: c.value().to_string(),
            domain: c.domain().unwrap_or("").to_string(),
            path: c.path().unwrap_or("/").to_string(),
            secure: c.secure().unwrap_or(false),
            http_only: c.http_only().unwrap_or(false),
        })
        .collect();
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Session 1 saat geçerli
    let valid_until = now + 3600;
    
    let session = SessionData {
        cookies,
        timestamp: now,
        valid_until,
    };
    
    session_manager.save_session("sompo", session)
        .map_err(|e| ApiError::Unknown(format!("Session kaydetme hatası: {}", e)))?;
    
    Ok(())
}

async fn restore_session(client: &Client, session: &SessionData, base_url: &str) -> Result<(), String> {
    // Önce domain'e git ki cookie'leri set edebilsin
    client.goto(base_url).await
        .map_err(|e| format!("Sayfa yüklenemedi: {}", e))?;
    
    // Cookie'leri yükle
    for cookie in &session.cookies {
        let cookie_value = serde_json::json!({
            "name": cookie.name,
            "value": cookie.value,
            "domain": cookie.domain,
            "path": cookie.path,
            "secure": cookie.secure,
            "httpOnly": cookie.http_only,
        });
        
        // Fantoccini'de add_cookie fonksiyonu farklı şekilde çalışıyor
        // Bu yüzden script ile ekleyeceğiz
        let script = format!(
            r#"document.cookie = "{}={}; domain={}; path={}; {}{}""#,
            cookie.name,
            cookie.value,
            cookie.domain,
            cookie.path,
            if cookie.secure { "secure; " } else { "" },
            if cookie.http_only { "httpOnly; " } else { "" }
        );
        
        if let Err(e) = client.execute(&script, vec![]).await {
            tracing::warn!("Cookie set edilemedi: {:?}", e);
        }
    }
    
    // Sayfayı yenile
    client.refresh().await
        .map_err(|e| format!("Sayfa yenilenemedi: {}", e))?;
    
    Ok(())
}

