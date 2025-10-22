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
    // Retry logic - Python script gibi, başarısızsa 1 kez daha session temizleyerek dene
    let mut attempts = 0;
    loop {
        match try_fetch_sompo_quote_internal(config.clone(), request.clone(), attempts).await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < 1 => {
                tracing::warn!("⚠️ Deneme {} başarısız: {}", attempts + 1, e);
                tracing::info!("🔄 Session temizleniyor, tekrar deneniyor...");
                
                // Session temizle
                let session_manager = SessionManager::new(&config.session_dir);
                session_manager.clear_session("sompo").ok();
                
                attempts += 1;
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
            Err(e) => {
                tracing::error!("❌ Tüm denemeler başarısız oldu: {}", e);
                return Err(e);
            }
        }
    }
}

// Internal: Tek deneme
async fn try_fetch_sompo_quote_internal(
    config: Arc<Config>,
    request: QuoteRequest,
    attempt: usize,
) -> Result<QuoteResponse, ApiError> {
    let scrape_start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    tracing::info!("🚀 Sompo quote işlemi başlatıldı (deneme {}): request_id={}", attempt + 1, request.quote_meta.request_id);
    
    // WebDriver client oluştur
    let client = create_webdriver_client(&config)
        .await
        .map_err(|e| ApiError::WebDriverError(format!("WebDriver bağlantısı başarısız: {}", e)))?;
    
    // Session manager
    let session_manager = SessionManager::new(&config.session_dir);
    
    // İlk denemede session temizle
    if attempt == 0 {
        session_manager.clear_session("sompo").ok();
        tracing::info!("🧹 Session temizlendi, temiz başlangıç");
    }
    
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
                    
                    // URL değişimini bekle (Playwright benzeri)
                    let js_wait_navigation = r#"
                        return new Promise((resolve) => {
                            const initialUrl = window.location.href;
                            let checks = 0;
                            const interval = setInterval(() => {
                                checks++;
                                if (window.location.href !== initialUrl) {
                                    clearInterval(interval);
                                    resolve({ navigated: true, newUrl: window.location.href, after: checks * 250 });
                                } else if (checks >= 40) {  // 10 saniye
                                    clearInterval(interval);
                                    resolve({ navigated: false, newUrl: window.location.href });
                                }
                            }, 250);
                        });
                    "#;
                    
                    if let Ok(nav_result) = client.execute(js_wait_navigation, vec![]).await {
                        tracing::info!("🔧 Navigation sonucu: {:?}", nav_result);
                        if let Some(obj) = nav_result.as_object() {
                            if obj.get("navigated").and_then(|v| v.as_bool()).unwrap_or(false) {
                                tracing::info!("✅ Sayfa değişti!");
                                
                                // Network idle bekle
                                wait_for_network_idle(&client, 10).await.ok();
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ {} ürün seçimi hatası: {}", product_type, e);
        }
    }
    
    // Ürün seçildiyse devam et (URL değişimi + network idle zaten beklendi)
    if product_selected {
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
    
    // Form doldurma - Playwright-style (tek async fonksiyon)
    let plate = &request.vehicle.plate;
    let tckn = &request.insured.tckn;
    
    tracing::info!("📝 Form dolduruluyor: Plaka={}, TCKN={}", plate, tckn);
    
    let js_fill_form = format!(r#"
        (async function fillForm() {{
            const data = {{ plaka: null, tckn: null }};
            
            // Tüm görünür input'ları al
            const inputs = Array.from(document.querySelectorAll('input:not([type="hidden"])'));
            const visibleInputs = inputs.filter(inp => inp.offsetParent !== null && !inp.disabled);
            
            // 1. Plaka
            const plateSelectors = [
                'input[name*="plak"]', 'input[name*="plate"]',
                'input[placeholder*="lak"]', 'input[placeholder*="late"]',
                'input#plaka', 'input#plate'
            ];
            
            for (const sel of plateSelectors) {{
                const inp = document.querySelector(sel);
                if (inp && inp.offsetParent !== null && !inp.disabled) {{
                    inp.focus();
                    inp.value = '{}';
                    inp.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    inp.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    inp.dispatchEvent(new Event('blur', {{ bubbles: true }}));
                    data.plaka = {{ selector: sel, name: inp.name }};
                    break;
                }}
            }}
            
            // Fallback: label'a göre ara
            if (!data.plaka) {{
                for (const inp of visibleInputs) {{
                    const label = inp.labels?.[0]?.textContent?.toLowerCase() || '';
                    const name = (inp.name || '').toLowerCase();
                    const placeholder = (inp.placeholder || '').toLowerCase();
                    
                    if (label.includes('plaka') || name.includes('plak') || placeholder.includes('lak')) {{
                        inp.focus();
                        inp.value = '{}';
                        inp.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        inp.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        data.plaka = {{ selector: 'fallback', name: inp.name }};
                        break;
                    }}
                }}
            }}
            
            // 2. TCKN
            const tcknSelectors = [
                'input[name*="tckn"]', 'input[name*="tcno"]', 'input[name*="kimlik"]',
                'input[placeholder*="TCKN"]', 'input[placeholder*="TCK"]', 'input[placeholder*="Kimlik"]'
            ];
            
            for (const sel of tcknSelectors) {{
                const inp = document.querySelector(sel);
                if (inp && inp.offsetParent !== null && !inp.disabled) {{
                    inp.focus();
                    inp.value = '{}';
                    inp.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    inp.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    inp.dispatchEvent(new Event('blur', {{ bubbles: true }}));
                    data.tckn = {{ selector: sel, name: inp.name }};
                    break;
                }}
            }}
            
            // Fallback: label'a göre ara
            if (!data.tckn) {{
                for (const inp of visibleInputs) {{
                    const label = inp.labels?.[0]?.textContent?.toLowerCase() || '';
                    const name = (inp.name || '').toLowerCase();
                    const placeholder = (inp.placeholder || '').toLowerCase();
                    
                    if (label.includes('tc') || label.includes('kimlik') || 
                        name.includes('tc') || placeholder.includes('tc')) {{
                        inp.focus();
                        inp.value = '{}';
                        inp.dispatchEvent(new Event('input', {{ bubbles: true }}));
                        inp.dispatchEvent(new Event('change', {{ bubbles: true }}));
                        data.tckn = {{ selector: 'fallback', name: inp.name }};
                        break;
                    }}
                }}
            }}
            
            return data;
        }})()
    "#, plate, plate, tckn, tckn);
    
    match client.execute(&js_fill_form, vec![]).await {
        Ok(result) => {
            tracing::info!("🔧 Form doldurma sonucu: {:?}", result);
            if let Some(obj) = result.as_object() {
                if obj.get("plaka").and_then(|v| v.as_object()).is_some() {
                    tracing::info!("✅ Plaka dolduruldu");
                } else {
                    tracing::warn!("⚠️ Plaka input bulunamadı");
                }
                
                if obj.get("tckn").and_then(|v| v.as_object()).is_some() {
                    tracing::info!("✅ TCKN dolduruldu");
                } else {
                    tracing::warn!("⚠️ TCKN input bulunamadı");
                }
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ Form doldurma hatası: {}", e);
        }
    }
    
    // Ek alanlar (varsa) - Form'un işlenmesi için bekle
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    
    // Form submit - Playwright-style has-text() simülasyonu
    tracing::info!("🔍 Submit butonu aranıyor...");
    
    let js_submit = r#"
        (async function submitForm() {
            // Playwright-style click fonksiyonu
            function playwrightClick(element) {
                element.scrollIntoView({ block: 'center', behavior: 'smooth' });
                element.focus();
                
                const rect = element.getBoundingClientRect();
                const x = rect.left + rect.width / 2;
                const y = rect.top + rect.height / 2;
                
                ['mousedown', 'mouseup', 'click'].forEach(eventType => {
                    element.dispatchEvent(new MouseEvent(eventType, {
                        view: window,
                        bubbles: true,
                        cancelable: true,
                        clientX: x,
                        clientY: y
                    }));
                });
                
                ['pointerdown', 'pointerup'].forEach(eventType => {
                    element.dispatchEvent(new PointerEvent(eventType, {
                        view: window,
                        bubbles: true,
                        cancelable: true,
                        clientX: x,
                        clientY: y
                    }));
                });
                
                element.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
                element.dispatchEvent(new KeyboardEvent('keyup', { key: 'Enter', bubbles: true }));
            }
            
            // has-text() simülasyonu: keywords
            const keywords = ['teklif', 'sorgula', 'hesapla', 'devam'];
            const buttons = Array.from(document.querySelectorAll('button:not([disabled]), input[type="submit"]'));
            
            // Görünür buttonları filtrele
            const visibleButtons = buttons.filter(btn => btn.offsetParent !== null);
            
            for (const btn of visibleButtons) {
                const text = (btn.textContent || btn.value || '').toLowerCase().trim();
                
                if (keywords.some(kw => text.includes(kw))) {
                    playwrightClick(btn);
                    
                    // Response için bekle
                    await new Promise(r => setTimeout(r, 2000));
                    
                    return { submitted: true, buttonText: text };
                }
            }
            
            return { submitted: false };
        })()
    "#;
    
    let mut form_submitted = false;
    match client.execute(js_submit, vec![]).await {
        Ok(result) => {
            tracing::info!("🔧 Submit button sonucu: {:?}", result);
            if let Some(obj) = result.as_object() {
                if obj.get("submitted").and_then(|v| v.as_bool()).unwrap_or(false) {
                    let button_text = obj.get("buttonText").and_then(|v| v.as_str()).unwrap_or("unknown");
                    tracing::info!("✅ Form submit edildi: {}", button_text);
                    form_submitted = true;
                    
                    // Network idle bekle
                    wait_for_network_idle(&client, 15).await.ok();
                } else {
                    tracing::warn!("⚠️ Submit butonu bulunamadı");
                }
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ Submit button hatası: {}", e);
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

// Helper: Network idle bekle (Playwright'ın wait_for_load_state("networkidle") benzeri)
async fn wait_for_network_idle(
    client: &fantoccini::Client,
    timeout_secs: u64,
) -> Result<(), ApiError> {
    tracing::info!("⏳ Network idle bekleniyor...");
    
    for i in 0..(timeout_secs * 2) {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        let js_check = r#"
            return {
                readyState: document.readyState,
                activeRequests: performance.getEntriesByType('resource')
                    .filter(r => !r.responseEnd).length
            };
        "#;
        
        if let Ok(result) = client.execute(js_check, vec![]).await {
            if let Some(obj) = result.as_object() {
                let ready_state = obj.get("readyState").and_then(|v| v.as_str()).unwrap_or("");
                let active_reqs = obj.get("activeRequests").and_then(|v| v.as_u64()).unwrap_or(999);
                
                if ready_state == "complete" && active_reqs == 0 {
                    tracing::info!("✅ Network idle! ({}.5 saniye)", i / 2);
                    return Ok(());
                }
            }
        }
    }
    
    tracing::warn!("⚠️ Network idle timeout: {} saniye", timeout_secs);
    Ok(()) // Timeout olsa bile devam et (strict değil)
}


