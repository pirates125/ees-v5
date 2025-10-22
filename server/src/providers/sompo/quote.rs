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
    
    // JavaScript ile ürün seçimi - Playwright-style click ile
    let js_select_product = format!(r#"
        const productName = '{}';
        
        // Playwright-style click function (daha güçlü)
        function playwrightClick(element) {{
            // 1. Scroll into view
            element.scrollIntoView({{ block: 'center', behavior: 'smooth' }});
            
            // 2. Focus
            element.focus();
            
            // 3. Mouse events (Playwright sırası)
            const rect = element.getBoundingClientRect();
            const x = rect.left + rect.width / 2;
            const y = rect.top + rect.height / 2;
            
            ['mousedown', 'mouseup', 'click'].forEach(eventType => {{
                element.dispatchEvent(new MouseEvent(eventType, {{
                    view: window,
                    bubbles: true,
                    cancelable: true,
                    clientX: x,
                    clientY: y
                }}));
            }});
            
            // 4. Pointer events (modern)
            ['pointerdown', 'pointerup'].forEach(eventType => {{
                element.dispatchEvent(new PointerEvent(eventType, {{
                    view: window,
                    bubbles: true,
                    cancelable: true,
                    clientX: x,
                    clientY: y
                }}));
            }});
            
            // 5. Enter key fallback
            element.dispatchEvent(new KeyboardEvent('keydown', {{ key: 'Enter', bubbles: true }}));
            element.dispatchEvent(new KeyboardEvent('keyup', {{ key: 'Enter', bubbles: true }}));
        }}
        
        // Modal içinde "Trafik" kartını bul
        const modals = document.querySelectorAll('[role="dialog"], .modal, .popup, .p-dialog, .p-overlay-content, .p-sidebar');
        
        for (const modal of modals) {{
            const buttons = Array.from(modal.querySelectorAll('button:not([disabled])'));
            
            for (const btn of buttons) {{
                const btnText = (btn.textContent || btn.innerText || '').toLowerCase().trim();
                
                if (btnText.includes('teklif') || btnText.includes('al')) {{
                    let container = btn.parentElement;
                    let depth = 0;
                    
                    while (container && depth < 7) {{
                        const containerText = (container.textContent || container.innerText || '').toLowerCase();
                        
                        if (containerText.includes(productName) && 
                            !containerText.includes('kamyon') &&
                            !containerText.includes('paket') &&
                            !containerText.includes('indirim') &&
                            containerText.length < 200) {{
                            
                            const buttonsInContainer = container.querySelectorAll('button');
                            
                            if (buttonsInContainer.length <= 3) {{
                                // Playwright-style click!
                                playwrightClick(btn);
                                
                                return {{ 
                                    found: true, 
                                    text: containerText.substring(0, 50), 
                                    buttonText: btnText,
                                    depth: depth,
                                    clickMethod: 'playwright_style'
                                }};
                            }}
                        }}
                        
                        container = container.parentElement;
                        depth++;
                    }}
                }}
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
                    tracing::info!("✅ {} ürünü butonu tıklandı", product_type);
                    product_selected = true;
                    
                    // Sayfa/modal değişimini bekle
                    tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
                    
                    // URL değişti mi kontrol et
                    if let Ok(new_url) = client.current_url().await {
                        tracing::info!("📍 Ürün seçimi sonrası URL: {}", new_url);
                    }
                    
                    // Modal kapandı mı kontrol et
                    let js_check_modal = r#"
                        const modals = document.querySelectorAll('[role="dialog"], .modal, .popup, .p-dialog, .p-overlay-content');
                        const visibleModals = Array.from(modals).filter(m => {
                            const style = window.getComputedStyle(m);
                            return style.display !== 'none' && style.visibility !== 'hidden';
                        });
                        return { modalCount: visibleModals.length };
                    "#;
                    
                    if let Ok(modal_result) = client.execute(js_check_modal, vec![]).await {
                        tracing::info!("🔧 Modal kontrolü: {:?}", modal_result);
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ {} ürün seçimi hatası: {}", product_type, e);
        }
    }
    
    // Ürün seçildiyse, modal kapanana kadar bekle
    if product_selected {
        tracing::info!("⏳ Modal kapanması bekleniyor...");
        
        for i in 0..20 {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            let js_check_modal_closed = r#"
                const modals = document.querySelectorAll('[role="dialog"], .modal, .popup, .p-dialog, .p-overlay-content');
                const visibleModals = Array.from(modals).filter(m => {
                    const style = window.getComputedStyle(m);
                    return style.display !== 'none' && style.visibility !== 'hidden' && style.opacity !== '0';
                });
                return visibleModals.length === 0;
            "#;
            
            if let Ok(result) = client.execute(js_check_modal_closed, vec![]).await {
                if result.as_bool().unwrap_or(false) {
                    tracing::info!("✅ Modal kapandı! ({}.5 saniye sonra)", i / 2);
                    break;
                }
            }
            
            if i == 19 {
                tracing::warn!("⚠️ Modal kapanma timeout! 10 saniye beklendi.");
            }
        }
        
        // Sayfa yüklensin diye ek bekle
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        tracing::info!("✅ Sayfa hazır, form doldurmaya başlanıyor");
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
    
    // Form doldurmadan önce sayfa içeriğini kontrol et
    let js_check_page = r#"
        const bodyText = document.body.innerText || '';
        const inputCount = document.querySelectorAll('input').length;
        const formCount = document.querySelectorAll('form').length;
        return {
            hasPlaka: bodyText.toLowerCase().includes('plaka'),
            hasTCKN: bodyText.toLowerCase().includes('tckn') || bodyText.toLowerCase().includes('kimlik'),
            inputCount: inputCount,
            formCount: formCount,
            firstInputs: Array.from(document.querySelectorAll('input')).slice(0, 5).map(i => ({
                type: i.type,
                name: i.name || '',
                placeholder: i.placeholder || '',
                id: i.id || ''
            }))
        };
    "#;
    
    if let Ok(page_check) = client.execute(js_check_page, vec![]).await {
        tracing::info!("📋 Sayfa durumu: {:?}", page_check);
    }
    
    // Form doldurma - Plaka (JavaScript ile akıllıca bul)
    let plate = &request.vehicle.plate;
    tracing::info!("🚗 Plaka: {}", plate);
    
    let js_fill_plate = format!(r#"
        const plateValue = '{}';
        
        // Plaka input'unu akıllıca bul
        const inputs = Array.from(document.querySelectorAll('input:not([type="hidden"]):not([disabled])'));
        
        for (const input of inputs) {{
            const name = (input.name || '').toLowerCase();
            const id = (input.id || '').toLowerCase();
            const placeholder = (input.placeholder || '').toLowerCase();
            const label = input.labels?.[0]?.textContent?.toLowerCase() || '';
            
            // Plaka ile ilgili input'u bul
            if (name.includes('plaka') || name.includes('plate') ||
                id.includes('plaka') || id.includes('plate') ||
                placeholder.includes('plaka') || placeholder.includes('plate') ||
                label.includes('plaka')) {{
                
                input.focus();
                input.value = plateValue;
                input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                
                return {{ 
                    found: true, 
                    name: input.name, 
                    id: input.id, 
                    placeholder: input.placeholder 
                }};
            }}
        }}
        
        return {{ found: false }};
    "#, plate);
    
    let mut plate_filled = false;
    match client.execute(&js_fill_plate, vec![]).await {
        Ok(result) => {
            tracing::info!("🔧 Plaka JavaScript sonucu: {:?}", result);
            if let Some(obj) = result.as_object() {
                if obj.get("found").and_then(|v| v.as_bool()).unwrap_or(false) {
                    tracing::info!("✅ Plaka dolduruldu (JavaScript)");
                    plate_filled = true;
                }
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ Plaka JavaScript hatası: {}", e);
        }
    }
    
    if !plate_filled {
        tracing::warn!("⚠️ Plaka input bulunamadı");
    }
    
    // TCKN doldur (JavaScript ile akıllıca bul)
    let tckn = &request.insured.tckn;
    tracing::info!("🔑 TCKN: {}", tckn);
    
    let js_fill_tckn = format!(r#"
        const tcknValue = '{}';
        
        // TCKN input'unu akıllıca bul
        const inputs = Array.from(document.querySelectorAll('input:not([type="hidden"]):not([disabled])'));
        
        for (const input of inputs) {{
            const name = (input.name || '').toLowerCase();
            const id = (input.id || '').toLowerCase();
            const placeholder = (input.placeholder || '').toLowerCase();
            const label = input.labels?.[0]?.textContent?.toLowerCase() || '';
            
            // TCKN/TC/Kimlik ile ilgili input'u bul
            if (name.includes('tckn') || name.includes('tcno') || name.includes('kimlik') ||
                id.includes('tckn') || id.includes('tcno') || id.includes('kimlik') ||
                placeholder.includes('tckn') || placeholder.includes('tc') || placeholder.includes('kimlik') ||
                label.includes('tckn') || label.includes('tc kimlik')) {{
                
                input.focus();
                input.value = tcknValue;
                input.dispatchEvent(new Event('input', {{ bubbles: true }}));
                input.dispatchEvent(new Event('change', {{ bubbles: true }}));
                
                return {{ 
                    found: true, 
                    name: input.name, 
                    id: input.id, 
                    placeholder: input.placeholder 
                }};
            }}
        }}
        
        return {{ found: false }};
    "#, tckn);
    
    match client.execute(&js_fill_tckn, vec![]).await {
        Ok(result) => {
            tracing::info!("🔧 TCKN JavaScript sonucu: {:?}", result);
            if let Some(obj) = result.as_object() {
                if obj.get("found").and_then(|v| v.as_bool()).unwrap_or(false) {
                    tracing::info!("✅ TCKN dolduruldu (JavaScript)");
                } else {
                    tracing::warn!("⚠️ TCKN input bulunamadı");
                }
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ TCKN JavaScript hatası: {}", e);
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

