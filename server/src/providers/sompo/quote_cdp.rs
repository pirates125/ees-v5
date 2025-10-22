use crate::browser::{create_cdp_browser, inject_anti_detection, wait_for_navigation, wait_for_network_idle};
use crate::config::Config;
use crate::http::{ApiError, Coverage, Installment, PremiumDetail, QuoteRequest, QuoteResponse, Timings};
use chromiumoxide::Page;
use data_encoding::BASE32;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use totp_lite::{totp_custom, Sha1};

pub async fn fetch_sompo_quote_cdp(
    config: Arc<Config>,
    request: QuoteRequest,
) -> Result<QuoteResponse, ApiError> {
    let scrape_start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    tracing::info!("🚀 Sompo CDP quote başlatıldı: request_id={}", request.quote_meta.request_id);
    
    // Browser başlat
    let mut browser = create_cdp_browser(&config)
        .await
        .map_err(|e| ApiError::WebDriverError(format!("CDP Browser başlatılamadı: {}", e)))?;
    
    // Yeni sayfa aç
    let page = browser.new_page("about:blank")
        .await
        .map_err(|e| ApiError::WebDriverError(format!("Page oluşturulamadı: {}", e)))?;
    
    // Anti-detection
    inject_anti_detection(&page).await.ok();
    
    // Login
    login_to_sompo_cdp(&page, &config).await?;
    
    // Quote al
    let result = get_quote_cdp(&page, &request, scrape_start).await;
    
    // Browser kapat
    let _ = browser.close().await;
    
    result
}

async fn login_to_sompo_cdp(
    page: &Page,
    config: &Config,
) -> Result<(), ApiError> {
    tracing::info!("🔐 Sompo login başlatılıyor (CDP)...");
    
    // Login sayfasına git
    let login_url = format!("{}/login", config.sompo_base_url.trim_end_matches('/'));
    page.goto(&login_url)
        .await
        .map_err(|e| ApiError::WebDriverError(format!("Login sayfası yüklenemedi: {}", e)))?;
    
    tracing::info!("✅ Login sayfası yüklendi");
    
    // Wait for page load
    wait_for_network_idle(page, 5).await.ok();
    
    // Username
    tracing::info!("🔍 Username input aranıyor...");
    let username_input = page.find_element("input[type='text'], input[name='username']")
        .await
        .map_err(|e| ApiError::WebDriverError(format!("Username input bulunamadı: {}", e)))?;
    
    username_input.click().await.ok();
    username_input.type_str(&config.sompo_username)
        .await
        .map_err(|e| ApiError::WebDriverError(format!("Username yazılamadı: {}", e)))?;
    
    tracing::info!("✅ Username dolduruldu");
    
    // Password
    let password_input = page.find_element("input[type='password']")
        .await
        .map_err(|e| ApiError::WebDriverError(format!("Password input bulunamadı: {}", e)))?;
    
    password_input.click().await.ok();
    password_input.type_str(&config.sompo_password)
        .await
        .map_err(|e| ApiError::WebDriverError(format!("Password yazılamadı: {}", e)))?;
    
    tracing::info!("✅ Password dolduruldu");
    
    // Login button
    let login_btn = page.find_element("button[type='submit']")
        .await
        .map_err(|e| ApiError::WebDriverError(format!("Login button bulunamadı: {}", e)))?;
    
    login_btn.click().await
        .map_err(|e| ApiError::WebDriverError(format!("Login button tıklanamadı: {}", e)))?;
    
    tracing::info!("✅ Login button tıklandı");
    
    // Wait for navigation
    wait_for_navigation(page, 10).await.ok();
    wait_for_network_idle(page, 5).await.ok();
    
    // OTP kontrolü
    if let Ok(Some(url)) = page.url().await {
        if url.contains("authenticator") {
            tracing::info!("🔐 OTP ekranı tespit edildi");
            handle_otp_cdp(page, config).await?;
        }
    }
    
    // Dashboard kontrolü
    if let Ok(Some(url)) = page.url().await {
        if url.contains("dashboard") && !url.contains("login") {
            tracing::info!("✅ Sompo login başarılı!");
            return Ok(());
        }
    }
    
    Err(ApiError::LoginFailed("Login başarısız - dashboard'a ulaşılamadı".to_string()))
}

async fn handle_otp_cdp(
    page: &Page,
    config: &Config,
) -> Result<(), ApiError> {
    tracing::info!("🔢 OTP işleniyor...");
    
    // Secret key al ve decode et
    let secret_key = &config.sompo_secret_key;
    
    let secret_bytes = BASE32.decode(secret_key.to_uppercase().as_bytes())
        .map_err(|e| ApiError::WebDriverError(format!("Secret key decode hatası: {}", e)))?;
    
    // TOTP üret (current, -30s, +30s)
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let otp_codes = vec![
        totp_custom::<Sha1>(30, 6, &secret_bytes, timestamp),      // Current
        totp_custom::<Sha1>(30, 6, &secret_bytes, timestamp - 30), // Previous
        totp_custom::<Sha1>(30, 6, &secret_bytes, timestamp + 30), // Next
    ];
    
    tracing::info!("🔢 OTP kodları üretildi: {:?}", otp_codes);
    
    // Her OTP'yi dene
    for (i, otp) in otp_codes.iter().enumerate() {
        tracing::info!("Deneme {}: OTP = {}", i + 1, otp);
        
        // PYTHON YAKLAŞIMI: Tek input'a tüm kodu yaz (placeholder bazlı)
        // Python: await page.fill('input[placeholder*="OTP"]', otp_code)
        
        let otp_selectors = vec![
            "input[placeholder*='OTP']",
            "input[placeholder*='Kod']",
            "input[placeholder*='Doğrulama']",
            "input[placeholder*='kod']",
            "input[name*='otp']",
        ];
        
        let mut otp_filled = false;
        
        for selector in otp_selectors {
            if let Ok(otp_input) = page.find_element(selector).await {
                tracing::info!("✅ OTP input bulundu: {}", selector);
                
                // Tüm 6 digit'i tek seferde yaz
                if otp_input.click().await.is_ok() {
                    if otp_input.type_str(otp).await.is_ok() {
                        tracing::info!("✅ OTP yazıldı: {}", otp);
                        otp_filled = true;
                        break;
                    }
                }
            }
        }
        
        // Fallback: JavaScript ile tüm input'ları dene
        if !otp_filled {
            tracing::info!("🔧 JavaScript fallback ile OTP dolduruluyor...");
            
            let js_fill_otp = format!(r#"
                const otp = '{}';
                
                // 1. Placeholder bazlı tek input
                const singleInput = document.querySelector('input[placeholder*="OTP"], input[placeholder*="Kod"], input[placeholder*="kod"]');
                if (singleInput) {{
                    singleInput.focus();
                    singleInput.value = otp;
                    singleInput.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    singleInput.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return {{ filled: true, method: 'single_input', count: 1 }};
                }}
                
                // 2. 6 ayrı input varsa (fallback)
                const inputs = Array.from(document.querySelectorAll('input[type="text"]:not([disabled])'))
                    .filter(inp => inp.offsetParent !== null)
                    .slice(0, 6);
                
                if (inputs.length === 6) {{
                    inputs.forEach((inp, idx) => {{
                        inp.value = otp[idx] || '';
                        inp.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        inp.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    }});
                    return {{ filled: true, method: 'separate_inputs', count: inputs.length }};
                }}
                
                return {{ filled: false, count: 0 }};
            "#, otp);
            
            if let Ok(result) = page.evaluate(js_fill_otp.as_str()).await {
                tracing::info!("OTP fill sonucu: {:?}", result);
                if let Ok(value) = result.into_value::<serde_json::Value>() {
                    if let Some(obj) = value.as_object() {
                        if obj.get("filled").and_then(|v| v.as_bool()).unwrap_or(false) {
                            otp_filled = true;
                        }
                    }
                }
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        // DEBUG: Screenshot + HTML logging
        if i == 0 {  // Sadece ilk denemede
            tracing::info!("🔍 OTP sayfası debug bilgileri alınıyor...");
            
            // Screenshot al
            if let Ok(screenshot) = page.screenshot(chromiumoxide::page::ScreenshotParams::builder().build()).await {
                if let Ok(_) = std::fs::write("otp_debug.png", screenshot) {
                    tracing::info!("📸 Screenshot kaydedildi: otp_debug.png");
                }
            }
            
            // Sayfa HTML'ini al
            if let Ok(content) = page.content().await {
                tracing::info!("📄 Sayfa HTML uzunluğu: {} karakter", content.len());
                
                // Button ve form elementlerini logla
                let js_debug = r#"
                    return {
                        buttons: Array.from(document.querySelectorAll('button, input[type="submit"]')).map(b => ({
                            tag: b.tagName,
                            type: b.type,
                            text: b.textContent?.trim() || b.value || '',
                            disabled: b.disabled,
                            visible: b.offsetParent !== null
                        })),
                        forms: Array.from(document.querySelectorAll('form')).length,
                        otpInputs: Array.from(document.querySelectorAll('input[type="text"]')).filter(i => i.offsetParent !== null).length,
                        url: window.location.href
                    };
                "#;
                
                if let Ok(debug_result) = page.evaluate(js_debug).await {
                    if let Ok(debug_value) = debug_result.into_value::<serde_json::Value>() {
                        tracing::info!("🔧 Debug info: {}", serde_json::to_string_pretty(&debug_value).unwrap_or_default());
                    }
                }
            }
        }
        
        // PYTHON YAKLAŞIMI: Submit button aramadan direkt URL değişimini bekle!
        // Python: WebDriverWait(driver, 20).until(EC.url_changes(LOGIN_URL))
        
        tracing::info!("⏳ OTP auto-submit bekleniyor (Python benzeri)...");
        
        // 2 saniye bekle (OTP validation için)
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        
        let auth_url = "google-authenticator-validation";
        
        // URL değişimini bekle (maksimum 15 saniye)
        for attempt in 0..30 {  // 30 * 500ms = 15 saniye
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            if let Ok(Some(current_url)) = page.url().await {
                tracing::debug!("URL check {}: {}", attempt + 1, current_url);
                
                // URL değişti mi?
                if !current_url.contains(auth_url) {
                    tracing::info!("✅ URL değişti: {}", current_url);
                    
                    // Dashboard'a ulaştık mı?
                    if current_url.contains("dashboard") {
                        tracing::info!("✅ OTP başarılı! Dashboard'a ulaşıldı");
                        
                        // Network idle bekle
                        wait_for_network_idle(page, 5).await.ok();
                        
                        return Ok(());
                    } else {
                        // Başka bir sayfaya gitti (captcha, bot detection vb.)
                        tracing::warn!("⚠️ Beklenmeyen sayfa: {}", current_url);
                        break;
                    }
                }
            }
        }
        
        // URL hala değişmediyse log
        if let Ok(Some(url)) = page.url().await {
            tracing::info!("📍 15 saniye sonra hala aynı URL: {}", url);
        }
    }
    
    Err(ApiError::LoginFailed("OTP başarısız - tüm denemeler tükendi".to_string()))
}

async fn get_quote_cdp(
    page: &Page,
    request: &QuoteRequest,
    scrape_start_ms: u64,
) -> Result<QuoteResponse, ApiError> {
    tracing::info!("📝 Quote formu dolduruluyor (CDP)...");
    
    // Trafik sayfasına git (JavaScript ile button bul ve tıkla)
    let js_click_trafik = r#"
        const keywords = ['trafik', 'traffic'];
        const buttons = Array.from(document.querySelectorAll('button, a'));
        
        for (const btn of buttons) {
            const text = (btn.textContent || btn.innerText || '').toLowerCase();
            if (keywords.some(kw => text.includes(kw)) && text.includes('teklif')) {
                btn.click();
                return { clicked: true, text: text };
            }
        }
        return { clicked: false };
    "#;
    
    if let Ok(result) = page.evaluate(js_click_trafik).await {
        tracing::info!("Trafik button: {:?}", result);
    }
    
    // Navigation + network idle bekle
    wait_for_navigation(page, 10).await.ok();
    wait_for_network_idle(page, 10).await.ok();
    
    // Form doldur - Plaka
    tracing::info!("🚗 Plaka: {}", request.vehicle.plate);
    
    let js_fill_plate = format!(r#"
        const plate = '{}';
        const inputs = Array.from(document.querySelectorAll('input[type="text"]:not([disabled])'));
        
        for (const inp of inputs) {{
            const name = (inp.name || '').toLowerCase();
            const placeholder = (inp.placeholder || '').toLowerCase();
            
            if (name.includes('plak') || placeholder.includes('lak')) {{
                inp.focus();
                inp.value = plate;
                inp.dispatchEvent(new Event('input', {{ bubbles: true }}));
                inp.dispatchEvent(new Event('change', {{ bubbles: true }}));
                inp.dispatchEvent(new Event('blur', {{ bubbles: true }}));
                return {{ filled: true, name: inp.name }};
            }}
        }}
        return {{ filled: false }};
    "#, request.vehicle.plate);
    
    if let Ok(result) = page.evaluate(js_fill_plate.as_str()).await {
        tracing::info!("Plaka fill: {:?}", result);
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    // TCKN
    tracing::info!("🔑 TCKN: {}", request.insured.tckn);
    
    let js_fill_tckn = format!(r#"
        const tckn = '{}';
        const inputs = Array.from(document.querySelectorAll('input[type="text"]:not([disabled])'));
        
        for (const inp of inputs) {{
            const name = (inp.name || '').toLowerCase();
            const placeholder = (inp.placeholder || '').toLowerCase();
            
            if (name.includes('tc') || name.includes('kimlik') || placeholder.includes('tc')) {{
                inp.focus();
                inp.value = tckn;
                inp.dispatchEvent(new Event('input', {{ bubbles: true }}));
                inp.dispatchEvent(new Event('change', {{ bubbles: true }}));
                inp.dispatchEvent(new Event('blur', {{ bubbles: true }}));
                return {{ filled: true, name: inp.name }};
            }}
        }}
        return {{ filled: false }};
    "#, request.insured.tckn);
    
    if let Ok(result) = page.evaluate(js_fill_tckn.as_str()).await {
        tracing::info!("TCKN fill: {:?}", result);
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Submit
    tracing::info!("🔍 Submit button aranıyor...");
    
    let js_submit = r#"
        const keywords = ['teklif', 'sorgula', 'hesapla'];
        const buttons = Array.from(document.querySelectorAll('button:not([disabled])'));
        
        for (const btn of buttons) {
            const text = (btn.textContent || '').toLowerCase();
            if (keywords.some(kw => text.includes(kw))) {
                btn.click();
                return { submitted: true, text: text };
            }
        }
        return { submitted: false };
    "#;
    
    if let Ok(result) = page.evaluate(js_submit).await {
        tracing::info!("Submit: {:?}", result);
    }
    
    // Results bekle
    wait_for_network_idle(page, 15).await.ok();
    
    // Fiyat parse et
    tracing::info!("💰 Fiyat parse ediliyor...");
    
    let js_parse_price = r#"
        const selectors = ['.premium', '.prim', '.price', '.fiyat', '.amount'];
        
        for (const sel of selectors) {
            const el = document.querySelector(sel);
            if (el && el.textContent.includes('TL')) {
                return { found: true, text: el.textContent, selector: sel };
            }
        }
        
        // Fallback: TL içeren tüm elementler
        const all = Array.from(document.querySelectorAll('*'));
        for (const el of all) {
            if (el.children.length === 0) {
                const text = el.textContent?.trim() || '';
                if (/\d{1,3}(\.\d{3})*(,\d{2})?\s*TL/.test(text) && text.length < 50) {
                    return { found: true, text: text, selector: 'fallback' };
                }
            }
        }
        
        return { found: false };
    "#;
    
    let price = match page.evaluate(js_parse_price).await {
        Ok(result) => {
            tracing::info!("Price parse result: {:?}", result);
            
            if let Ok(value) = result.into_value::<serde_json::Value>() {
                if let Some(obj_map) = value.as_object() {
                    if obj_map.get("found").and_then(|v| v.as_bool()).unwrap_or(false) {
                        if let Some(text) = obj_map.get("text").and_then(|v| v.as_str()) {
                            parse_tl_price(text)?
                        } else {
                            return Err(ApiError::ParseError("Fiyat text bulunamadı".to_string()));
                        }
                    } else {
                        return Err(ApiError::ParseError("Fiyat elementi bulunamadı".to_string()));
                    }
                } else {
                    return Err(ApiError::ParseError("Price parse response invalid".to_string()));
                }
            } else {
                return Err(ApiError::ParseError("Price parse response invalid".to_string()));
            }
        }
        Err(e) => {
            return Err(ApiError::ParseError(format!("Price parse hatası: {}", e)));
        }
    };
    
    tracing::info!("✅ Fiyat: {:.2} TL", price);
    
    // Response oluştur
    let net = price / 1.18;
    let taxes = price - net;
    
    let scrape_elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64 - scrape_start_ms;
    
    Ok(QuoteResponse {
        request_id: request.quote_meta.request_id.clone(),
        company: "Sompo".to_string(),
        product_type: "trafik".to_string(),
        premium: PremiumDetail {
            net: (net * 100.0).round() / 100.0,
            gross: (price * 100.0).round() / 100.0,
            taxes: (taxes * 100.0).round() / 100.0,
            currency: "TRY".to_string(),
        },
        installments: vec![
            Installment {
                count: 1,
                per_installment: price,
                total: price,
            },
        ],
        coverages: vec![
            Coverage {
                code: "TRAFIK_ZORUNLU".to_string(),
                name: "Zorunlu Trafik Sigortası".to_string(),
                limit: None,
                included: true,
            },
        ],
        warnings: vec![],
        raw: None,
        timings: Some(Timings {
            queued_ms: 0,
            scrape_ms: scrape_elapsed,
        }),
    })
}

fn parse_tl_price(text: &str) -> Result<f64, ApiError> {
    let cleaned = text
        .replace("TL", "")
        .replace("₺", "")
        .replace(" ", "")
        .replace(".", "")  // Binlik ayıracı
        .replace(",", ".") // Ondalık ayıracı
        .trim()
        .to_string();
    
    cleaned.parse::<f64>()
        .map_err(|e| ApiError::ParseError(format!("Fiyat parse hatası: {} (text: '{}')", e, text)))
}

