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
    
    let current_url = client.current_url().await
        .map_err(|e| ApiError::WebDriverError(format!("URL alınamadı: {}", e)))?;
    tracing::info!("✅ Sompo sayfası yüklendi: {}", current_url);
    
    // Önce spesifik XPath'i dene (Python'dan gelen)
    let mut username_filled = false;
    tracing::info!("🔍 Username input aranıyor (XPath)...");
    if let Ok(elem) = client.find(Locator::XPath(SompoSelectors::USERNAME_XPATH)).await {
        tracing::info!("✅ Username input bulundu (XPath)");
        if let Ok(_) = elem.send_keys(&config.sompo_username).await {
            tracing::info!("✅ Username dolduruldu (XPath): {}", mask_sensitive(&config.sompo_username));
            username_filled = true;
        } else {
            tracing::warn!("⚠️ Username gönderilemedi (XPath)");
        }
    } else {
        tracing::warn!("⚠️ Username input bulunamadı (XPath), CSS deneniyor...");
    }
    
    // Başarısız olduysa CSS selector'ları dene
    if !username_filled {
        tracing::info!("🔍 Username input aranıyor (CSS selectors)...");
        username_filled = try_fill_input(client, SompoSelectors::USERNAME_INPUTS, &config.sompo_username).await?;
        if !username_filled {
            tracing::error!("❌ Username input hiçbir selector ile bulunamadı!");
            return Err(ApiError::LoginFailed("Username input bulunamadı".to_string()));
        }
        tracing::info!("✅ Username dolduruldu (CSS): {}", mask_sensitive(&config.sompo_username));
    }
    
    // Password için aynı strateji
    let mut password_filled = false;
    tracing::info!("🔍 Password input aranıyor (XPath)...");
    if let Ok(elem) = client.find(Locator::XPath(SompoSelectors::PASSWORD_XPATH)).await {
        tracing::info!("✅ Password input bulundu (XPath)");
        if let Ok(_) = elem.send_keys(&config.sompo_password).await {
            tracing::info!("✅ Password dolduruldu (XPath)");
            password_filled = true;
        } else {
            tracing::warn!("⚠️ Password gönderilemedi (XPath)");
        }
    } else {
        tracing::warn!("⚠️ Password input bulunamadı (XPath), CSS deneniyor...");
    }
    
    if !password_filled {
        tracing::info!("🔍 Password input aranıyor (CSS selectors)...");
        password_filled = try_fill_input(client, SompoSelectors::PASSWORD_INPUTS, &config.sompo_password).await?;
        if !password_filled {
            tracing::error!("❌ Password input hiçbir selector ile bulunamadı!");
            return Err(ApiError::LoginFailed("Password input bulunamadı".to_string()));
        }
        tracing::info!("✅ Password dolduruldu (CSS)");
    }
    
    // Login butonuna tıkla
    tracing::info!("🔍 Login butonu aranıyor...");
    let login_clicked = try_click_button(client, SompoSelectors::LOGIN_BUTTONS).await?;
    if !login_clicked {
        tracing::error!("❌ Login butonu hiçbir selector ile bulunamadı!");
        return Err(ApiError::LoginFailed("Login butonu bulunamadı".to_string()));
    }
    tracing::info!("✅ Login butonu tıklandı");
    
    // Login işleminin tamamlanmasını bekle
    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
    
    let current_url = client.current_url().await
        .map_err(|e| ApiError::WebDriverError(format!("URL alınamadı: {}", e)))?;
    
    tracing::info!("📍 Login sonrası URL: {}", current_url);
    
    // Hata mesajı kontrolü
    if let Ok(error_elem) = client.find(Locator::Css(".error-message, .alert-danger, .text-danger, [role='alert']")).await {
        if let Ok(error_text) = error_elem.text().await {
            if !error_text.trim().is_empty() {
                tracing::error!("❌ Login hatası: {}", error_text);
                return Err(ApiError::LoginFailed(format!("Login hatası: {}", error_text)));
            }
        }
    }
    
    // OTP kontrolü
    if let Ok(otp_found) = check_otp_required(client).await {
        if otp_found {
            tracing::info!("🔐 OTP ekranı tespit edildi");
            handle_otp(client, &config.sompo_secret_key).await?;
            
            // OTP sonrası URL kontrol et
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            let post_otp_url = client.current_url().await
                .map_err(|e| ApiError::WebDriverError(format!("URL alınamadı: {}", e)))?;
            tracing::info!("📍 OTP sonrası URL: {}", post_otp_url);
        } else {
            tracing::info!("ℹ️ OTP ekranı bulunamadı");
        }
    }
    
    // Son URL kontrolü
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    let final_url = client.current_url().await
        .map_err(|e| ApiError::WebDriverError(format!("URL alınamadı: {}", e)))?;
    
    tracing::info!("📍 Final URL: {}", final_url);
    
    // Hala login sayfasındaysak hata
    if final_url.as_str().to_lowercase().contains("login") {
        // Sayfa kaynağını logla (debugging için)
        if let Ok(source) = client.source().await {
            tracing::debug!("📄 Sayfa kaynağı (ilk 500 karakter): {}", &source.chars().take(500).collect::<String>());
        }
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
        tracing::debug!("  → Deneniyor: {}", selector);
        match client.find(Locator::Css(selector)).await {
            Ok(elem) => {
                tracing::info!("  ✅ Element bulundu: {}", selector);
                elem.send_keys(value).await
                    .map_err(|e| ApiError::WebDriverError(e.to_string()))?;
                return Ok(true);
            }
            Err(e) => {
                tracing::debug!("  ✗ Bulunamadı: {} ({})", selector, e);
                continue;
            },
        }
    }
    Ok(false)
}

async fn try_click_button(client: &Client, selectors: &[&str]) -> Result<bool, ApiError> {
    for selector in selectors {
        tracing::debug!("  → Deneniyor: {}", selector);
        match client.find(Locator::Css(selector)).await {
            Ok(elem) => {
                tracing::info!("  ✅ Buton bulundu: {}", selector);
                elem.click().await
                    .map_err(|e| ApiError::WebDriverError(e.to_string()))?;
                return Ok(true);
            }
            Err(e) => {
                tracing::debug!("  ✗ Bulunamadı: {} ({})", selector, e);
                continue;
            },
        }
    }
    Ok(false)
}

async fn check_otp_required(client: &Client) -> Result<bool, ApiError> {
    tracing::info!("🔍 OTP ekranı kontrol ediliyor...");
    for selector in SompoSelectors::OTP_INPUTS {
        tracing::debug!("  → OTP selector deneniyor: {}", selector);
        if client.find(Locator::Css(selector)).await.is_ok() {
            tracing::info!("  ✅ OTP input bulundu: {}", selector);
            return Ok(true);
        }
    }
    
    // XPath ile de dene
    let otp_xpaths = [
        "//input[@type='text' and contains(@placeholder, 'OTP')]",
        "//input[@type='text' and contains(@placeholder, 'kod')]",
        "//input[contains(@name, 'otp')]",
        "//input[contains(@id, 'otp')]",
    ];
    
    for xpath in otp_xpaths {
        tracing::debug!("  → OTP XPath deneniyor: {}", xpath);
        if client.find(Locator::XPath(xpath)).await.is_ok() {
            tracing::info!("  ✅ OTP input bulundu (XPath): {}", xpath);
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

