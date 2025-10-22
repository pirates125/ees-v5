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
    let mut password_elem_ref = None;
    tracing::info!("🔍 Password input aranıyor (XPath)...");
    if let Ok(elem) = client.find(Locator::XPath(SompoSelectors::PASSWORD_XPATH)).await {
        tracing::info!("✅ Password input bulundu (XPath)");
        if let Ok(_) = elem.send_keys(&config.sompo_password).await {
            tracing::info!("✅ Password dolduruldu (XPath)");
            password_filled = true;
            password_elem_ref = Some(elem);
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
    
    // Enter tuşuna bas (bazı formlar sadece Enter ile submit olur)
    if let Some(pwd_elem) = password_elem_ref {
        tracing::info!("⌨️ Password field'a Enter tuşu basılıyor...");
        if let Ok(_) = pwd_elem.send_keys("\n").await {
            tracing::info!("✅ Enter tuşu basıldı");
            tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        }
    } else {
        // Eleman referansı yoksa tekrar bul
        if let Ok(pwd) = client.find(Locator::XPath(SompoSelectors::PASSWORD_XPATH)).await {
            tracing::info!("⌨️ Password field'a Enter tuşu basılıyor...");
            if let Ok(_) = pwd.send_keys("\n").await {
                tracing::info!("✅ Enter tuşu basıldı");
                tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
            }
        }
    }
    
    // Login butonuna tıkla
    tracing::info!("🔍 Login butonu aranıyor...");
    let login_clicked = try_click_button(client, SompoSelectors::LOGIN_BUTTONS).await?;
    if !login_clicked {
        tracing::error!("❌ Login butonu hiçbir selector ile bulunamadı!");
        return Err(ApiError::LoginFailed("Login butonu bulunamadı".to_string()));
    }
    tracing::info!("✅ Login butonu tıklandı");
    
    // Buton tıklandıktan hemen sonra JS tetikleniyor mu kontrol et
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // JavaScript ile butonu manuel tetikle (SPA için)
    let js_click_button = r#"
        const btn = document.querySelector('button[type="submit"]');
        if (btn) {
            console.log('Button manuel tıklanıyor...');
            btn.click();
            return 'clicked';
        }
        return 'button not found';
    "#;
    
    match client.execute(js_click_button, vec![]).await {
        Ok(result) => {
            tracing::info!("🔧 JavaScript button click: {:?}", result);
        }
        Err(e) => {
            tracing::warn!("⚠️ JavaScript button click başarısız: {}", e);
        }
    }
    
    // Network request'leri bekle
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
    
    // XHR/Fetch request'lerini kontrol et
    let js_check_requests = r#"
        const perfEntries = performance.getEntriesByType('resource')
            .filter(e => e.initiatorType === 'xmlhttprequest' || e.initiatorType === 'fetch')
            .slice(-5)
            .map(e => e.name);
        return JSON.stringify(perfEntries);
    "#;
    
    match client.execute(js_check_requests, vec![]).await {
        Ok(result) => {
            tracing::info!("🌐 Recent XHR/Fetch requests: {:?}", result);
        }
        Err(e) => {
            tracing::debug!("XHR check failed: {}", e);
        }
    }
    
    // JavaScript hataları kontrol et
    let js_check_errors = r#"
        if (window.jsErrors && window.jsErrors.length > 0) {
            return JSON.stringify(window.jsErrors);
        }
        return 'no errors tracked';
    "#;
    
    match client.execute(js_check_errors, vec![]).await {
        Ok(result) => {
            tracing::info!("⚠️ JavaScript errors: {:?}", result);
        }
        Err(_) => {}
    }
    
    // Screenshot al
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    if let Ok(screenshot) = client.screenshot().await {
        tracing::info!("📸 Screenshot alındı ({} bytes)", screenshot.len());
        if let Ok(_) = std::fs::write("sompo_after_login_click.png", screenshot) {
            tracing::info!("💾 Screenshot kaydedildi: sompo_after_login_click.png");
        }
    }
    
    // Login işleminin tamamlanmasını bekle (daha uzun süre)
    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
    
    let current_url = client.current_url().await
        .map_err(|e| ApiError::WebDriverError(format!("URL alınamadı: {}", e)))?;
    
    tracing::info!("📍 Login sonrası URL: {}", current_url);
    
    // Hata mesajı kontrolü (geniş selector listesi)
    let error_selectors = [
        ".error-message", ".alert-danger", ".text-danger", "[role='alert']",
        ".error", ".alert", ".warning", ".invalid-feedback",
        "p.text-red-500", "div.text-red-600", "span.error",
    ];
    
    for selector in error_selectors {
        if let Ok(error_elem) = client.find(Locator::Css(selector)).await {
            if let Ok(error_text) = error_elem.text().await {
                if !error_text.trim().is_empty() {
                    tracing::error!("❌ Login hatası bulundu ({}): {}", selector, error_text);
                    return Err(ApiError::LoginFailed(format!("Login hatası: {}", error_text)));
                }
            }
        }
    }
    
    // Sayfadaki tüm visible text'i al (hata mesajı aramak için)
    if let Ok(body) = client.find(Locator::Css("body")).await {
        if let Ok(body_text) = body.text().await {
            let lowercase_text = body_text.to_lowercase();
            if lowercase_text.contains("hatalı") || 
               lowercase_text.contains("yanlış") || 
               lowercase_text.contains("geçersiz") ||
               lowercase_text.contains("incorrect") ||
               lowercase_text.contains("invalid") {
                tracing::error!("❌ Sayfada hata metni tespit edildi: {}", 
                    body_text.lines().take(5).collect::<Vec<_>>().join(" | "));
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
            tracing::debug!("📄 Sayfa kaynağı (ilk 2000 karakter): {}", &source.chars().take(2000).collect::<String>());
            
            // Body text'i de al
            if let Ok(body) = client.find(Locator::Css("body")).await {
                if let Ok(body_text) = body.text().await {
                    tracing::info!("📝 Sayfa görünür metni: {}", 
                        body_text.lines()
                            .filter(|line| !line.trim().is_empty())
                            .take(10)
                            .collect::<Vec<_>>()
                            .join(" | "));
                }
            }
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
    let current_url = client.current_url().await
        .map_err(|e| ApiError::WebDriverError(format!("URL alınamadı: {}", e)))?;
    
    tracing::info!("🔍 OTP ekranı kontrol ediliyor... URL: {}", current_url);
    
    // URL'de authenticator varsa kesinlikle OTP ekranı
    if current_url.as_str().contains("authenticator") {
        tracing::info!("✅ Google Authenticator URL tespit edildi!");
        
        // Tüm input'ları listele (normal DOM + iframe + shadow DOM)
        let js_find_inputs = r#"
            function findAllInputs() {
                const inputs = [];
                
                // 1. Normal DOM
                const normalInputs = Array.from(document.querySelectorAll('input'));
                normalInputs.forEach(inp => {
                    inputs.push({
                        type: inp.type,
                        name: inp.name,
                        id: inp.id,
                        placeholder: inp.placeholder,
                        class: inp.className,
                        location: 'normal_dom'
                    });
                });
                
                // 2. iframe'ler içinde
                const iframes = document.querySelectorAll('iframe');
                iframes.forEach((iframe, idx) => {
                    try {
                        const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
                        const iframeInputs = Array.from(iframeDoc.querySelectorAll('input'));
                        iframeInputs.forEach(inp => {
                            inputs.push({
                                type: inp.type,
                                name: inp.name,
                                id: inp.id,
                                placeholder: inp.placeholder,
                                class: inp.className,
                                location: 'iframe_' + idx
                            });
                        });
                    } catch (e) {
                        // Cross-origin iframe
                    }
                });
                
                // 3. Shadow DOM (recursive)
                function findInShadow(root, path) {
                    const shadowInputs = Array.from(root.querySelectorAll('input'));
                    shadowInputs.forEach(inp => {
                        inputs.push({
                            type: inp.type,
                            name: inp.name,
                            id: inp.id,
                            placeholder: inp.placeholder,
                            class: inp.className,
                            location: 'shadow_' + path
                        });
                    });
                    
                    const allNodes = root.querySelectorAll('*');
                    allNodes.forEach((node, idx) => {
                        if (node.shadowRoot) {
                            findInShadow(node.shadowRoot, path + '_' + idx);
                        }
                    });
                }
                findInShadow(document, 'root');
                
                return inputs;
            }
            return findAllInputs();
        "#;
        
        match client.execute(js_find_inputs, vec![]).await {
            Ok(result) => {
                tracing::info!("📋 Sayfadaki tüm input'lar (DOM + iframe + shadow): {:?}", result);
            }
            Err(e) => {
                tracing::warn!("Input listesi alınamadı: {}", e);
            }
        }
        
        return Ok(true);
    }
    
    for selector in SompoSelectors::OTP_INPUTS {
        tracing::debug!("  → OTP selector deneniyor: {}", selector);
        if client.find(Locator::Css(selector)).await.is_ok() {
            tracing::info!("  ✅ OTP input bulundu: {}", selector);
            return Ok(true);
        }
    }
    
    // XPath ile de dene
    let otp_xpaths = [
        "//input[@type='text']",  // Genel text input
        "//input[@type='tel']",    // Tel input
        "//input[@type='number']", // Number input
        "//input",                 // Herhangi bir input
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
        tracing::error!("❌ SOMPO_SECRET_KEY yapılandırılmamış!");
        tracing::info!("📱 Google Authenticator Secret Key gerekli!");
        tracing::info!("   .env dosyasına şunu ekleyin:");
        tracing::info!("   SOMPO_SECRET_KEY=YOUR_TOTP_SECRET_KEY");
        return Err(ApiError::HumanActionRequired(
            "SOMPO_SECRET_KEY yapılandırılmamış - Manuel OTP girişi gerekli. 30 saniye içinde manuel olarak girin!".to_string()
        ));
    }
    
    // TOTP kodu üret
    let totp = totp_lite::totp_custom::<totp_lite::Sha1>(30, 6, secret_key.as_bytes(), 0);
    tracing::info!("🔢 OTP kodu üretildi: {}", totp);
    
    // Screenshot al (OTP ekranı)
    if let Ok(screenshot) = client.screenshot().await {
        if let Ok(_) = std::fs::write("sompo_otp_screen.png", screenshot) {
            tracing::info!("💾 OTP ekranı screenshot'u kaydedildi: sompo_otp_screen.png");
        }
    }
    
    // Tüm VISIBLE input'ları say (6 ayrı input olabilir) - iframe ve shadow DOM dahil
    let js_count_inputs = r#"
        function countVisibleInputs() {
            const allInputs = [];
            
            // Normal DOM
            allInputs.push(...Array.from(document.querySelectorAll('input')));
            
            // iframe içindeki input'lar
            const iframes = document.querySelectorAll('iframe');
            iframes.forEach(iframe => {
                try {
                    const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
                    allInputs.push(...Array.from(iframeDoc.querySelectorAll('input')));
                } catch (e) {}
            });
            
            // Shadow DOM içindeki input'lar
            function findInShadow(root) {
                const shadowInputs = [];
                shadowInputs.push(...Array.from(root.querySelectorAll('input')));
                const allNodes = root.querySelectorAll('*');
                allNodes.forEach(node => {
                    if (node.shadowRoot) {
                        shadowInputs.push(...findInShadow(node.shadowRoot));
                    }
                });
                return shadowInputs;
            }
            allInputs.push(...findInShadow(document));
            
            // SADECE visible ve enabled olanları say
            const visibleInputs = allInputs.filter(input => {
                if (input.type === 'hidden') return false;
                if (input.disabled) return false;
                if (input.readOnly) return false;
                
                const style = window.getComputedStyle(input);
                if (style.display === 'none') return false;
                if (style.visibility === 'hidden') return false;
                if (style.opacity === '0') return false;
                
                const rect = input.getBoundingClientRect();
                if (rect.width === 0 || rect.height === 0) return false;
                
                return true;
            });
            
            return visibleInputs.length;
        }
        return countVisibleInputs();
    "#;
    
    let input_count = match client.execute(js_count_inputs, vec![]).await {
        Ok(result) => {
            tracing::info!("📊 Sayfada {} input bulundu", result);
            result
        }
        Err(_) => serde_json::Value::Number(serde_json::Number::from(0))
    };
    
    // Eğer 6 input varsa, tek tek doldurmak gerekebilir (Google Authenticator UI pattern)
    if let Some(count) = input_count.as_u64() {
        if count >= 6 {
            tracing::info!("🔢 6+ input tespit edildi, tek tek doldurma deneniyor...");
            
            let js_fill_separate = format!(r#"
                function findAllOtpInputs() {{
                    const allInputs = [];
                    
                    // 1. Normal DOM
                    allInputs.push(...Array.from(document.querySelectorAll('input')));
                    
                    // 2. iframe içinde
                    const iframes = document.querySelectorAll('iframe');
                    iframes.forEach(iframe => {{
                        try {{
                            const iframeDoc = iframe.contentDocument || iframe.contentWindow.document;
                            allInputs.push(...Array.from(iframeDoc.querySelectorAll('input')));
                        }} catch (e) {{
                            // Cross-origin iframe
                        }}
                    }});
                    
                    // 3. Shadow DOM içinde (recursive)
                    function findInShadow(root) {{
                        const shadowInputs = [];
                        shadowInputs.push(...Array.from(root.querySelectorAll('input')));
                        
                        const allNodes = root.querySelectorAll('*');
                        allNodes.forEach(node => {{
                            if (node.shadowRoot) {{
                                shadowInputs.push(...findInShadow(node.shadowRoot));
                            }}
                        }});
                        
                        return shadowInputs;
                    }}
                    allInputs.push(...findInShadow(document));
                    
                    // SADECE visible ve enabled input'ları filtrele (KRITIK!)
                    const visibleInputs = allInputs.filter(input => {{
                        // Gizli veya disabled olanları atla
                        if (input.type === 'hidden') return false;
                        if (input.disabled) return false;
                        if (input.readOnly) return false;
                        
                        // Display:none veya visibility:hidden olanları atla
                        const style = window.getComputedStyle(input);
                        if (style.display === 'none') return false;
                        if (style.visibility === 'hidden') return false;
                        if (style.opacity === '0') return false;
                        
                        // Boyutu 0 olanları atla
                        const rect = input.getBoundingClientRect();
                        if (rect.width === 0 || rect.height === 0) return false;
                        
                        return true;
                    }});
                    
                    console.log('📊 Toplam input:', allInputs.length, 'Visible:', visibleInputs.length);
                    return visibleInputs;
                }}
                
                const allInputs = findAllOtpInputs();
                const code = '{}';
                let filled = 0;
                
                // Normal DOM'daki input'ları tercih et (shadow DOM duplicate olabilir)
                const normalDomInputs = Array.from(document.querySelectorAll('input[type="text"]')).filter(inp => {{
                    if (inp.type === 'hidden' || inp.disabled || inp.readOnly) return false;
                    const style = window.getComputedStyle(inp);
                    if (style.display === 'none' || style.visibility === 'hidden' || style.opacity === '0') return false;
                    const rect = inp.getBoundingClientRect();
                    if (rect.width === 0 || rect.height === 0) return false;
                    return true;
                }});
                
                // Eğer normal DOM'da yeterli input varsa onu kullan, yoksa tümünü kullan
                const inputs = normalDomInputs.length >= 6 ? normalDomInputs.slice(0, 6) : allInputs.slice(0, 6);
                
                console.log('📊 Toplam:', allInputs.length, '| Normal DOM:', normalDomInputs.length, '| Kullanılacak:', inputs.length);
                
                for (let i = 0; i < Math.min(inputs.length, code.length); i++) {{
                    const input = inputs[i];
                    
                    // Focus
                    input.focus();
                    
                    // Set value
                    input.value = code[i];
                    
                    // Trigger multiple events
                    input.dispatchEvent(new Event('input', {{ bubbles: true, cancelable: true }}));
                    input.dispatchEvent(new Event('change', {{ bubbles: true, cancelable: true }}));
                    input.dispatchEvent(new KeyboardEvent('keydown', {{ key: code[i], code: 'Digit' + code[i], bubbles: true }}));
                    input.dispatchEvent(new KeyboardEvent('keyup', {{ key: code[i], code: 'Digit' + code[i], bubbles: true }}));
                    
                    // Auto-focus next input (some UIs do this)
                    if (i < inputs.length - 1) {{
                        inputs[i + 1].focus();
                    }}
                    
                    filled++;
                }}
                
                // Focus last input
                if (inputs.length > 0) {{
                    inputs[inputs.length - 1].focus();
                }}
                
                return {{ filled: filled, total: inputs.length }};
            "#, totp);
            
            match client.execute(&js_fill_separate, vec![]).await {
                Ok(result) => {
                    tracing::info!("✅ {} input JavaScript ile dolduruldu: {:?}", count, result);
                    
                    // Submit butonunu bul ve tıkla
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    
                    let js_find_and_click_button = r#"
                        // Önce button'u bul
                        const buttons = Array.from(document.querySelectorAll('button'));
                        let submitBtn = buttons.find(btn => 
                            btn.textContent.includes('Doğrula') || 
                            btn.textContent.includes('Onayla') || 
                            btn.textContent.includes('Gönder') ||
                            btn.textContent.includes('Submit') ||
                            btn.type === 'submit'
                        );
                        
                        if (submitBtn) {
                            submitBtn.click();
                            return 'button_clicked';
                        }
                        
                        // Buton yoksa Enter bas
                        const inputs = document.querySelectorAll('input');
                        if (inputs.length > 0) {
                            const lastInput = inputs[inputs.length - 1];
                            lastInput.focus();
                            
                            // Enter event
                            lastInput.dispatchEvent(new KeyboardEvent('keydown', { 
                                key: 'Enter', 
                                code: 'Enter', 
                                keyCode: 13, 
                                which: 13, 
                                bubbles: true,
                                cancelable: true
                            }));
                            
                            lastInput.dispatchEvent(new KeyboardEvent('keypress', { 
                                key: 'Enter', 
                                code: 'Enter', 
                                keyCode: 13, 
                                which: 13, 
                                bubbles: true,
                                cancelable: true
                            }));
                            
                            lastInput.dispatchEvent(new KeyboardEvent('keyup', { 
                                key: 'Enter', 
                                code: 'Enter', 
                                keyCode: 13, 
                                which: 13, 
                                bubbles: true,
                                cancelable: true
                            }));
                            
                            return 'enter_pressed';
                        }
                        
                        return 'no_action';
                    "#;
                    
                    match client.execute(js_find_and_click_button, vec![]).await {
                        Ok(result) => {
                            tracing::info!("🔧 OTP submit action: {:?}", result);
                        }
                        Err(e) => {
                            tracing::warn!("⚠️ Submit action başarısız: {}", e);
                        }
                    }
                    
                    // OTP submit butonu kontrolü geç, doğrulama bekle
                    tracing::info!("⏳ OTP doğrulaması bekleniyor...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
                    
                    // OTP sonrası URL kontrol et
                    let post_otp_url = client.current_url().await
                        .map_err(|e| ApiError::WebDriverError(format!("URL alınamadı: {}", e)))?;
                    
                    tracing::info!("📍 OTP sonrası URL (6 input yöntemi): {}", post_otp_url);
                    
                    if post_otp_url.as_str().contains("authenticator") {
                        tracing::warn!("⚠️ 6 input yöntemi başarısız, standart yönteme geçiliyor...");
                    } else {
                        tracing::info!("✅ OTP doğrulaması başarılı (6 input yöntemi)!");
                        return Ok(());
                    }
                }
                Err(e) => {
                    tracing::warn!("⚠️ JavaScript ile doldurma başarısız: {}", e);
                }
            }
        }
    }
    
    // Standart yöntem: Tek bir input'a tüm kodu gir
    tracing::info!("🔍 Tek input'a tüm kodu girme deneniyor...");
    let generic_selectors = [
        "input[type='text']",
        "input[type='tel']",
        "input[type='number']",
        "input",
    ];
    
    let mut otp_filled = false;
    for selector in generic_selectors {
        if let Ok(elem) = client.find(Locator::Css(selector)).await {
            tracing::info!("🔍 OTP input bulundu: {}", selector);
            if let Ok(_) = elem.send_keys(&totp).await {
                tracing::info!("✅ OTP kodu girildi: {}", selector);
                otp_filled = true;
                
                // Enter tuşu bas
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                if let Ok(_) = elem.send_keys("\n").await {
                    tracing::info!("⌨️ Enter tuşu basıldı");
                }
                break;
            }
        }
    }
    
    if !otp_filled {
        // Fallback: Standart selector'lar
        otp_filled = try_fill_input(client, SompoSelectors::OTP_INPUTS, &totp).await?;
        if !otp_filled {
            tracing::error!("❌ OTP input hiçbir selector ile bulunamadı!");
            return Err(ApiError::HumanActionRequired("OTP input bulunamadı - 30 saniye içinde manuel olarak girin!".to_string()));
        }
    }
    
    tracing::info!("✅ OTP kodu girildi");
    
    // OTP submit butonu
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    let otp_submitted = try_click_button(client, SompoSelectors::OTP_SUBMIT_BUTTONS).await?;
    if otp_submitted {
        tracing::info!("✅ OTP submit edildi");
    } else {
        tracing::warn!("⚠️ OTP submit butonu bulunamadı (Enter tuşu zaten basıldı)");
    }
    
    // OTP doğrulamasının tamamlanmasını bekle
    tracing::info!("⏳ OTP doğrulaması bekleniyor...");
    tokio::time::sleep(tokio::time::Duration::from_millis(5000)).await;
    
    // OTP sonrası URL ve sayfa durumunu kontrol et
    let post_otp_url = client.current_url().await
        .map_err(|e| ApiError::WebDriverError(format!("URL alınamadı: {}", e)))?;
    
    tracing::info!("📍 OTP sonrası URL: {}", post_otp_url);
    
    // Hala OTP sayfasındaysa hata mesajı kontrol et
    if post_otp_url.as_str().contains("authenticator") {
        tracing::warn!("⚠️ Hala OTP sayfasında! Hata mesajı kontrol ediliyor...");
        
        // Sayfa metnini al
        if let Ok(body) = client.find(Locator::Css("body")).await {
            if let Ok(body_text) = body.text().await {
                tracing::info!("📝 OTP sayfası metni: {}", 
                    body_text.lines()
                        .filter(|line| !line.trim().is_empty())
                        .take(10)
                        .collect::<Vec<_>>()
                        .join(" | "));
                
                // Hata mesajı var mı?
                let lowercase = body_text.to_lowercase();
                if lowercase.contains("hatalı") || 
                   lowercase.contains("yanlış") || 
                   lowercase.contains("geçersiz") ||
                   lowercase.contains("incorrect") {
                    tracing::error!("❌ OTP hatalı! Sayfa metni: {}", body_text);
                    return Err(ApiError::LoginFailed("OTP doğrulama başarısız - kod hatalı veya süresi dolmuş".to_string()));
                }
            }
        }
        
        // Screenshot al
        if let Ok(screenshot) = client.screenshot().await {
            if let Ok(_) = std::fs::write("sompo_otp_failed.png", screenshot) {
                tracing::info!("💾 OTP başarısız screenshot'u: sompo_otp_failed.png");
            }
        }
        
        return Err(ApiError::LoginFailed("OTP doğrulama başarısız - hala OTP sayfasında".to_string()));
    }
    
    tracing::info!("✅ OTP doğrulaması başarılı!");
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
    
    // LocalStorage'ı al (Python kodundan gelen özellik)
    let js_get_local_storage = r#"
        try {
            const items = {};
            for (let i = 0; i < localStorage.length; i++) {
                const key = localStorage.key(i);
                items[key] = localStorage.getItem(key);
            }
            return JSON.stringify(items);
        } catch (e) {
            return JSON.stringify({});
        }
    "#;
    
    let local_storage = match client.execute(js_get_local_storage, vec![]).await {
        Ok(result) => {
            if let Some(json_str) = result.as_str() {
                match serde_json::from_str::<std::collections::HashMap<String, String>>(json_str) {
                    Ok(map) => {
                        tracing::info!("💾 LocalStorage alındı: {} items", map.len());
                        map
                    }
                    Err(e) => {
                        tracing::warn!("LocalStorage parse hatası: {}", e);
                        std::collections::HashMap::new()
                    }
                }
            } else {
                std::collections::HashMap::new()
            }
        }
        Err(e) => {
            tracing::warn!("LocalStorage alınamadı: {}", e);
            std::collections::HashMap::new()
        }
    };
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Session 1 saat geçerli
    let valid_until = now + 3600;
    
    let session = SessionData {
        cookies,
        local_storage,
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
    
    // LocalStorage'ı yükle (Python kodundan gelen özellik)
    if !session.local_storage.is_empty() {
        let local_storage_json = serde_json::to_string(&session.local_storage)
            .unwrap_or_else(|_| "{}".to_string());
        
        let js_set_local_storage = format!(r#"
            try {{
                const data = {};
                for (const [key, value] of Object.entries(data)) {{
                    localStorage.setItem(key, value);
                }}
                return true;
            }} catch (e) {{
                return false;
            }}
        "#, local_storage_json);
        
        match client.execute(&js_set_local_storage, vec![]).await {
            Ok(_) => {
                tracing::info!("💾 LocalStorage yüklendi: {} items", session.local_storage.len());
            }
            Err(e) => {
                tracing::warn!("LocalStorage yüklenemedi: {}", e);
            }
        }
    }
    
    // Sayfayı yenile
    client.refresh().await
        .map_err(|e| format!("Sayfa yenilenemedi: {}", e))?;
    
    Ok(())
}

