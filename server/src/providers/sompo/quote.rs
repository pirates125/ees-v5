use crate::browser::{create_webdriver_client, SessionManager};
use crate::config::Config;
use crate::http::{ApiError, QuoteRequest, QuoteResponse};
use crate::providers::sompo::login::login_to_sompo;
use crate::providers::sompo::parser::parse_quote_from_page;
use crate::providers::sompo::selectors::SompoSelectors;
use fantoccini::Locator;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn fetch_sompo_quote(
    config: Arc<Config>,
    request: QuoteRequest,
) -> Result<QuoteResponse, ApiError> {
    let scrape_start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    tracing::info!("🚀 Sompo quote işlemi başlatıldı: request_id={}", request.quote_meta.request_id);
    
    // WebDriver client oluştur
    let client = create_webdriver_client(&config)
        .await
        .map_err(|e| ApiError::WebDriverError(format!("WebDriver bağlantısı başarısız: {}", e)))?;
    
    // Session manager
    let session_manager = SessionManager::new(&config.session_dir);
    
    // Login
    if let Err(e) = login_to_sompo(&client, config.clone(), &session_manager).await {
        let _ = client.close().await;
        return Err(e);
    }
    
    // Ürün tipine göre sayfaya git
    let product_type = match request.coverage.product_type {
        crate::http::models::ProductType::Trafik => "trafik",
        crate::http::models::ProductType::Kasko => "kasko",
        _ => {
            let _ = client.close().await;
            return Err(ApiError::FormValidation("Desteklenmeyen ürün tipi".to_string()));
        }
    };
    
    tracing::info!("🚗 Ürün türü: {}", product_type);
    
    // Önce "YENİ İŞ TEKLİFİ" butonuna tıkla (dashboard'dayız)
    tracing::info!("🔍 'YENİ İŞ TEKLİFİ' butonu aranıyor...");
    let js_click_new_quote = r#"
        const buttons = Array.from(document.querySelectorAll('button'));
        for (const btn of buttons) {
            const text = (btn.innerText || '').toUpperCase();
            if (text.includes('YENİ') && text.includes('İŞ') && text.includes('TEKLİF')) {
                btn.click();
                return { found: true, text: btn.innerText };
            }
        }
        return { found: false };
    "#;
    
    let mut new_quote_clicked = false;
    match client.execute(js_click_new_quote, vec![]).await {
        Ok(result) => {
            tracing::info!("🔧 YENİ İŞ TEKLİFİ button search: {:?}", result);
            if let Some(obj) = result.as_object() {
                if obj.get("found").and_then(|v| v.as_bool()).unwrap_or(false) {
                    tracing::info!("✅ YENİ İŞ TEKLİFİ butonuna tıklandı");
                    new_quote_clicked = true;
                    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                }
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ YENİ İŞ TEKLİFİ butonu JavaScript hatası: {}", e);
        }
    }
    
    if !new_quote_clicked {
        tracing::warn!("⚠️ YENİ İŞ TEKLİFİ butonu bulunamadı, direkt ürün seçimine geçiliyor");
    }
    
    // YENİ İŞ TEKLİFİ butonuna tıkladıktan sonra ne oldu? Kontrol et
    if let Ok(current_url) = client.current_url().await {
        tracing::info!("📍 YENİ İŞ TEKLİFİ sonrası URL: {}", current_url);
    }
    
    // QR Kod Sıfırlama popup'ı varsa kapat
    let js_close_qr_popup = r#"
        const buttons = Array.from(document.querySelectorAll('button'));
        for (const btn of buttons) {
            const text = (btn.innerText || '').toLowerCase();
            if (text.includes('hayır') || text.includes('kapat') || text.includes('iptal')) {
                btn.click();
                return { closed: true, text: btn.innerText };
            }
        }
        return { closed: false };
    "#;
    
    if let Ok(result) = client.execute(js_close_qr_popup, vec![]).await {
        if let Some(obj) = result.as_object() {
            if obj.get("closed").and_then(|v| v.as_bool()).unwrap_or(false) {
                tracing::info!("✅ Popup kapatıldı");
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
        }
    }
    
    // Modal/popup içindeki metinleri logla
    let js_get_modal_text = r#"
        // Modal, dialog, popup elementlerini ara
        const modals = document.querySelectorAll('[role="dialog"], .modal, .popup, .p-dialog, .p-overlay-content');
        if (modals.length > 0) {
            const modalText = Array.from(modals).map(m => m.innerText).join(' | ');
            return { hasModal: true, text: modalText.substring(0, 500) };
        }
        
        // Modal yoksa genel sayfa metni
        const allText = document.body.innerText || '';
        return { hasModal: false, text: allText.substring(0, 500) };
    "#;
    
    if let Ok(result) = client.execute(js_get_modal_text, vec![]).await {
        if let Some(obj) = result.as_object() {
            let has_modal = obj.get("hasModal").and_then(|v| v.as_bool()).unwrap_or(false);
            let text = obj.get("text").and_then(|v| v.as_str()).unwrap_or("");
            if has_modal {
                tracing::info!("📝 Modal/popup içeriği: {}", text);
            } else {
                tracing::info!("📝 Sayfa metni: {}", text);
            }
        }
    }
    
    // Ürün sayfasına git (Trafik/Kasko seçimi)
    tracing::info!("🔍 {} ürünü seçiliyor (modal/popup içinde aranıyor)...", product_type);
    
    // JavaScript ile ürün seçimi - önce modal içinde, sonra sayfada
    let js_select_product = format!(r#"
        const productName = '{}';
        
        // 1. Önce modal/dialog/popup içinde ara
        const modals = document.querySelectorAll('[role="dialog"], .modal, .popup, .p-dialog, .p-overlay-content, .p-sidebar');
        for (const modal of modals) {{
            // Modal içindeki tüm tıklanabilir elementleri ara
            const clickables = modal.querySelectorAll('button, a, .card, .product-item, [role="button"], div[onclick]');
            for (const elem of clickables) {{
                const text = (elem.innerText || elem.textContent || '').toLowerCase();
                // "trafik" kelimesini içeriyorsa VE yanlış bir şey değilse
                if (text.includes(productName) && 
                    !text.includes('kamyon') && 
                    !text.includes('paket') && 
                    !text.includes('indirim') &&
                    text.length < 50) {{ // Kısa metinleri tercih et (başlıklar)
                    elem.click();
                    return {{ found: true, text: text, location: 'modal', tag: elem.tagName }};
                }}
            }}
        }}
        
        // 2. Modal içinde bulamadıysa, genel sayfada ara
        const allClickables = document.querySelectorAll('button, a, .card, .product-card, [role="button"]');
        for (const elem of allClickables) {{
            const text = (elem.innerText || elem.textContent || '').toLowerCase();
            if (text === productName || 
                (text.includes(productName) && text.length < 30 && !text.includes('paket'))) {{
                elem.click();
                return {{ found: true, text: text, location: 'page', tag: elem.tagName }};
            }}
        }}
        
        return {{ found: false }};
    "#, product_type);
    
    let mut product_selected = false;
    match client.execute(&js_select_product, vec![]).await {
        Ok(result) => {
            tracing::info!("🔧 {} ürün seçimi: {:?}", product_type, result);
            if let Some(obj) = result.as_object() {
                if obj.get("found").and_then(|v| v.as_bool()).unwrap_or(false) {
                    tracing::info!("✅ {} ürünü seçildi", product_type);
                    product_selected = true;
                    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                }
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ {} ürün seçimi hatası: {}", product_type, e);
        }
    }
    
    // JavaScript başarısızsa CSS selectors dene
    if !product_selected {
        tracing::info!("🔍 CSS selectors ile {} ürünü aranıyor...", product_type);
        let product_selectors = match product_type {
            "trafik" => SompoSelectors::TRAFIK_LINKS,
            "kasko" => SompoSelectors::KASKO_LINKS,
            _ => SompoSelectors::TRAFIK_LINKS,
        };
        
        for selector in product_selectors {
            if let Ok(elem) = client.find(Locator::Css(selector)).await {
                if let Ok(_) = elem.click().await {
                    tracing::info!("✅ Ürün sayfasına gidildi: {}", selector);
                    product_selected = true;
                    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                    break;
                }
            }
        }
    }
    
    if !product_selected {
        tracing::warn!("⚠️ Ürün seçimi başarısız, mevcut sayfada devam ediliyor");
    }
    
    // Form doldurma - Plaka
    let plate = &request.vehicle.plate;
    tracing::info!("🚗 Plaka: {}", plate);
    
    let mut plate_filled = false;
    for selector in SompoSelectors::PLATE_INPUTS {
        if let Ok(elem) = client.find(Locator::Css(selector)).await {
            if let Ok(_) = elem.send_keys(plate).await {
                tracing::info!("✅ Plaka dolduruldu: {}", selector);
                plate_filled = true;
                break;
            }
        }
    }
    
    if !plate_filled {
        tracing::warn!("⚠️ Plaka input bulunamadı");
    }
    
    // TCKN doldur
    let tckn = &request.insured.tckn;
    for selector in SompoSelectors::TCKN_INPUTS {
        if let Ok(elem) = client.find(Locator::Css(selector)).await {
            if let Ok(_) = elem.send_keys(tckn).await {
                tracing::info!("✅ TCKN dolduruldu");
                break;
            }
        }
    }
    
    // Ek alanlar (varsa)
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Form submit - CSS selectors ile dene
    let mut form_submitted = false;
    for selector in SompoSelectors::FORM_SUBMIT_BUTTONS {
        if let Ok(elem) = client.find(Locator::Css(selector)).await {
            if let Ok(_) = elem.click().await {
                tracing::info!("✅ Form submit edildi: {}", selector);
                form_submitted = true;
                break;
            }
        }
    }
    
    // CSS selectors başarısızsa, JavaScript ile button bul ve tıkla
    if !form_submitted {
        tracing::info!("🔍 CSS selectors başarısız, JavaScript ile button aranıyor...");
        
        let js_find_submit = r#"
            // Teklif/Sorgula/Hesapla gibi buttonları bul
            const keywords = ['teklif', 'sorgula', 'hesapla', 'devam', 'submit'];
            const buttons = Array.from(document.querySelectorAll('button, input[type="submit"], a.btn'));
            
            for (const btn of buttons) {
                const text = btn.innerText || btn.value || '';
                if (keywords.some(kw => text.toLowerCase().includes(kw))) {
                    btn.click();
                    return { found: true, text: text };
                }
            }
            
            // Form içindeki herhangi bir submit button
            const formSubmit = document.querySelector('form button[type="submit"]');
            if (formSubmit) {
                formSubmit.click();
                return { found: true, text: 'form[submit]' };
            }
            
            return { found: false };
        "#;
        
        match client.execute(js_find_submit, vec![]).await {
            Ok(result) => {
                tracing::info!("🔧 JavaScript button search sonucu: {:?}", result);
                if let Some(obj) = result.as_object() {
                    if obj.get("found").and_then(|v| v.as_bool()).unwrap_or(false) {
                        form_submitted = true;
                        tracing::info!("✅ JavaScript ile form submit edildi!");
                    }
                }
            }
            Err(e) => {
                tracing::warn!("⚠️ JavaScript button search hatası: {}", e);
            }
        }
    }
    
    if !form_submitted {
        // Screenshot al (debug için)
        if let Ok(png_data) = client.screenshot().await {
            let screenshot_path = format!("sompo_form_submit_error.png");
            if let Ok(_) = std::fs::write(&screenshot_path, &png_data) {
                tracing::info!("📸 Form screenshot kaydedildi: {}", screenshot_path);
            }
        }
        
        // Sayfadaki tüm buttonları logla
        let js_list_buttons = r#"
            const buttons = Array.from(document.querySelectorAll('button, input[type="submit"]'));
            return buttons.map(b => ({
                tag: b.tagName,
                type: b.type || '',
                text: b.innerText || b.value || '',
                class: b.className || ''
            })).slice(0, 10);
        "#;
        
        if let Ok(buttons) = client.execute(js_list_buttons, vec![]).await {
            tracing::info!("📋 Sayfadaki buttonlar: {:?}", buttons);
        }
        
        let _ = client.close().await;
        return Err(ApiError::FormValidation("Form submit butonu bulunamadı".to_string()));
    }
    
    // Sonuçların yüklenmesini bekle
    tracing::info!("⏳ Sonuçlar bekleniyor...");
    tokio::time::sleep(tokio::time::Duration::from_millis(8000)).await;
    
    // Loading göstergesi kaybolsun
    for _retry in 0..10 {
        let mut loading_found = false;
        for selector in SompoSelectors::LOADING_INDICATORS {
            if client.find(Locator::Css(selector)).await.is_ok() {
                loading_found = true;
                break;
            }
        }
        
        if !loading_found {
            break;
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
    
    // Fiyatı parse et
    let result = parse_quote_from_page(&client, request.quote_meta.request_id.clone(), scrape_start).await;
    
    // Screenshot al (debug için)
    if let Err(_) = &result {
        let screenshot_path = format!("./screenshots/sompo_error_{}.png", uuid::Uuid::new_v4());
        if let Ok(png_data) = client.screenshot().await {
            if let Ok(_) = std::fs::write(&screenshot_path, &png_data) {
                tracing::info!("📸 Error screenshot: {}", screenshot_path);
            }
        }
    }
    
    // Browser'ı kapat
    let _ = client.close().await;
    
    result
}

