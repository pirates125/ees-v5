#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Sompo Playwright Scraper - Subprocess için
Çalışan sompo.py kodunu adapt ettik
"""

import asyncio
import json
import os
import sys
import time
import pyotp
from playwright.async_api import async_playwright, TimeoutError as PWTimeout

def parse_tl_price(text):
    """TL fiyat parse et"""
    cleaned = text.replace('TL', '').replace('₺', '').replace(' ', '')
    cleaned = cleaned.replace('.', '').replace(',', '.')
    try:
        return float(cleaned.strip())
    except:
        return 0.0

async def main():
    # Environment variables
    username = os.getenv("SOMPO_USER", "")
    password = os.getenv("SOMPO_PASS", "")
    secret_key = os.getenv("SOMPO_SECRET", "")
    
    # Quote request (JSON from stdin)
    try:
        request = json.loads(sys.argv[1]) if len(sys.argv) > 1 else {}
    except:
        request = {}
    
    plate = request.get("plate", "34ABC123")
    tckn = request.get("tckn", "12345678901")
    product_type = request.get("product_type", "trafik")
    
    if not username or not password:
        print(json.dumps({"error": "SOMPO_USER ve SOMPO_PASS gerekli"}), file=sys.stderr)
        sys.exit(1)
    
    start_time = time.time()
    
    async with async_playwright() as p:
        # Browser başlat - anti-bot arguments
        browser = await p.chromium.launch(
            headless=False,  # VDS'de RDP ile görebilmek için
            args=[
                '--disable-blink-features=AutomationControlled',
                '--no-sandbox',
                '--disable-dev-shm-usage',
                '--disable-web-security',
                '--disable-features=IsolateOrigins,site-per-process',
                '--disable-site-isolation-trials',
                '--start-maximized',
                '--disable-infobars',
                '--window-size=1920,1080'
            ]
        )
        
        # Context - realistic browser fingerprint
        context = await browser.new_context(
            viewport={'width': 1920, 'height': 1080},
            user_agent='Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36',
            locale='tr-TR',
            timezone_id='Europe/Istanbul',
            extra_http_headers={
                'Accept-Language': 'tr-TR,tr;q=0.9,en-US;q=0.8,en;q=0.7',
                'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8'
            }
        )
        
        page = await context.new_page()
        
        # Anti-detection: webdriver property'yi gizle
        await page.add_init_script("""
            Object.defineProperty(navigator, 'webdriver', {
                get: () => undefined
            });
            
            Object.defineProperty(window, 'navigator', {
                value: new Proxy(navigator, {
                    has: (target, key) => (key === 'webdriver' ? false : key in target),
                    get: (target, key) =>
                        key === 'webdriver' ? undefined : typeof target[key] === 'function' ? target[key].bind(target) : target[key]
                })
            });
            
            window.navigator.chrome = { runtime: {} };
            Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
            Object.defineProperty(navigator, 'languages', { get: () => ['tr-TR', 'tr', 'en-US', 'en'] });
        """)
        
        try:
            print(f"[INFO] Sompo scraping başlatıldı: {product_type} - {plate}", file=sys.stderr)
            
            # ==================== LOGIN ====================
            await page.goto("https://ejento.somposigorta.com.tr/dashboard/login", timeout=30000)
            print(f"[INFO] Login sayfası yüklendi", file=sys.stderr)
            
            # Form bekle
            await page.wait_for_selector('form', timeout=15000)
            
            # Username - Human-like typing (type yavaş yavaş)
            username_selector = 'input[type="text"], input[name="username"]'
            await page.click(username_selector)  # Focus
            await page.wait_for_timeout(500)
            await page.type(username_selector, username, delay=100)  # 100ms per char (human-like)
            
            # Validation
            await page.wait_for_timeout(300)
            input_value = await page.input_value(username_selector)
            if input_value == username:
                print(f"[INFO] Username girildi: {username}", file=sys.stderr)
            else:
                print(f"[WARNING] Username validation failed! Expected: {username}, Got: {input_value}", file=sys.stderr)
            
            # Password - Human-like typing
            password_selector = 'input[type="password"]'
            await page.click(password_selector)  # Focus
            await page.wait_for_timeout(500)
            await page.type(password_selector, password, delay=120)  # 120ms per char
            
            # Validation (length check)
            await page.wait_for_timeout(300)
            input_value = await page.input_value(password_selector)
            if len(input_value) == len(password):
                print(f"[INFO] Password girildi (len={len(password)})", file=sys.stderr)
            else:
                print(f"[WARNING] Password validation failed! Expected len: {len(password)}, Got: {len(input_value)}", file=sys.stderr)
            
            # Login - Enter key (daha natural)
            await page.press(password_selector, 'Enter')
            print(f"[INFO] Enter tuşuna basıldı (login)", file=sys.stderr)
            
            # URL değişimini bekle - daha esnek
            await page.wait_for_timeout(3000)  # 3 saniye bekle
            
            current_url = page.url
            print(f"[INFO] URL after login: {current_url}", file=sys.stderr)
            
            # Sayfa içeriğinde hata var mı?
            page_content = await page.content()
            
            # Hata mesajlarını ara
            error_messages = []
            try:
                # Visible error messages
                error_elements = await page.query_selector_all('.error, .alert, .message, [class*="error"], [class*="alert"]')
                for el in error_elements:
                    if await el.is_visible():
                        text = await el.text_content()
                        if text and text.strip():
                            error_messages.append(text.strip())
            except:
                pass
            
            if error_messages:
                print(f"[ERROR] Sayfa hata mesajları: {error_messages}", file=sys.stderr)
                await page.screenshot(path="debug_login_error.png")
            elif "hata" in page_content.lower() or "error" in page_content.lower():
                # Genel hata kontrolü
                await page.screenshot(path="debug_login_error.png")
                print(f"[WARNING] Login sayfasında hata mesajı olabilir", file=sys.stderr)
                
                # Sayfa title'ı logla
                title = await page.title()
                print(f"[DEBUG] Page title: {title}", file=sys.stderr)
            
            # OTP ekranı?
            if "authenticator" in current_url or "google-authenticator" in current_url or "otp" in current_url.lower():
                print(f"[INFO] OTP ekranı tespit edildi", file=sys.stderr)
                
                if not secret_key:
                    print(json.dumps({"error": "SOMPO_SECRET gerekli"}), file=sys.stderr)
                    sys.exit(1)
                
                # TOTP üret
                otp = pyotp.TOTP(secret_key).now()
                print(f"[INFO] OTP üretildi", file=sys.stderr)
                
                # OTP input bul ve doldur
                otp_input = await page.query_selector('input[placeholder*="OTP"], input[placeholder*="Kod"], input[type="text"]')
                if otp_input:
                    await page.fill('input[placeholder*="OTP"], input[placeholder*="Kod"], input[type="text"]', otp)
                    print(f"[INFO] OTP girildi", file=sys.stderr)
                    
                    # URL değişimini bekle (auto-submit)
                    try:
                        await page.wait_for_url(lambda url: "authenticator" not in url, timeout=20000)
                        print(f"[INFO] OTP başarılı!", file=sys.stderr)
                    except:
                        print(f"[WARNING] OTP timeout", file=sys.stderr)
                else:
                    print(f"[ERROR] OTP input bulunamadı", file=sys.stderr)
                    sys.exit(1)
            elif "login" in current_url:
                # Hala login sayfasındaysa - credentials veya bot detection
                await page.screenshot(path="debug_still_login.png")
                print(f"[ERROR] Hala login sayfasında - credentials yanlış veya bot detection", file=sys.stderr)
                print(f"[DEBUG] Screenshot: debug_still_login.png", file=sys.stderr)
                print(json.dumps({"error": "Login başarısız - credentials kontrol edin"}), file=sys.stderr)
                sys.exit(1)
            
            # Dashboard kontrolü - daha esnek
            try:
                await page.wait_for_url(lambda url: "dashboard" in url and "login" not in url, timeout=15000)
                dashboard_url = page.url
                print(f"[INFO] Dashboard'a ulaşıldı: {dashboard_url}", file=sys.stderr)
                
                # Bot detection kontrolü
                if "/bot" in dashboard_url:
                    print(f"[WARNING] Bot detection sayfası tespit edildi", file=sys.stderr)
                    await page.screenshot(path="debug_bot_detection.png")
                    
                    # "ANA SAYFAYI YÜKLE" butonunu ara ve tıkla
                    print(f"[INFO] 🔄 'ANA SAYFAYI YÜKLE' butonu aranıyor...", file=sys.stderr)
                    
                    js_refresh_button = """
                        (() => {
                            const buttons = Array.from(document.querySelectorAll('button, a'));
                            const refreshBtn = buttons.find(b => 
                                b.offsetParent !== null && 
                                (b.textContent || '').includes('ANA SAYFAYI YÜKLE')
                            );
                            
                            if (refreshBtn) {
                                refreshBtn.scrollIntoView({block: 'center'});
                                refreshBtn.click();
                                return {success: true, text: (refreshBtn.textContent || '').trim()};
                            }
                            
                            return {success: false};
                        })()
                    """
                    
                    refresh_clicked = False
                    try:
                        result = await page.evaluate(js_refresh_button)
                        if result.get('success'):
                            print(f"[INFO] 'ANA SAYFAYI YÜKLE' tıklandı ✅", file=sys.stderr)
                            refresh_clicked = True
                            await page.wait_for_timeout(3000)
                        else:
                            print(f"[WARNING] 'ANA SAYFAYI YÜKLE' butonu bulunamadı, page.reload() deneniyor", file=sys.stderr)
                            await page.reload(wait_until="networkidle", timeout=15000)
                            await page.wait_for_timeout(2000)
                    except Exception as e:
                        print(f"[WARNING] Refresh hatası: {str(e)[:100]}, page.reload() deneniyor", file=sys.stderr)
                        await page.reload(wait_until="networkidle", timeout=15000)
                        await page.wait_for_timeout(2000)
                    
                    current_url = page.url
                    print(f"[DEBUG] URL after refresh: {current_url}", file=sys.stderr)
                    
                    if "/bot" in current_url:
                        print(f"[WARNING] Hala bot sayfasında, 2. deneme...", file=sys.stderr)
                        
                        # 2. deneme
                        try:
                            result = await page.evaluate(js_refresh_button)
                            if result.get('success'):
                                print(f"[INFO] 'ANA SAYFAYI YÜKLE' tıklandı (2. deneme)", file=sys.stderr)
                                await page.wait_for_timeout(3000)
                            else:
                                await page.reload(wait_until="networkidle", timeout=15000)
                                await page.wait_for_timeout(2000)
                        except:
                            await page.reload(wait_until="networkidle", timeout=15000)
                            await page.wait_for_timeout(2000)
                        
                        final_url = page.url
                        print(f"[DEBUG] URL after 2nd refresh: {final_url}", file=sys.stderr)
                        
                        if "/bot" in final_url:
                            print(f"[ERROR] 2 refresh sonrası hala bot sayfasında: {final_url}", file=sys.stderr)
                            await page.screenshot(path="debug_bot_still_there.png")
                            print(json.dumps({"error": "Bot detection - 2 refresh sonrası hala bot sayfası"}), file=sys.stderr)
                            sys.exit(1)
                        else:
                            print(f"[INFO] ✅ Bot sayfası bypass edildi! Yeni URL: {final_url}", file=sys.stderr)
                    else:
                        print(f"[INFO] ✅ Bot sayfası bypass edildi! Yeni URL: {current_url}", file=sys.stderr)
                
            except:
                # Timeout ama dashboard'da olabiliriz
                current_url = page.url
                if "dashboard" in current_url and "login" not in current_url:
                    print(f"[INFO] Dashboard'a ulaşıldı (timeout ama URL doğru): {current_url}", file=sys.stderr)
                    
                    # Bot detection kontrolü
                    if "/bot" in current_url:
                        print(f"[WARNING] Bot detection sayfası (timeout branch)", file=sys.stderr)
                        
                        # "ANA SAYFAYI YÜKLE" butonunu ara
                        js_refresh_button = """
                            (() => {
                                const buttons = Array.from(document.querySelectorAll('button, a'));
                                const refreshBtn = buttons.find(b => 
                                    b.offsetParent !== null && 
                                    (b.textContent || '').includes('ANA SAYFAYI YÜKLE')
                                );
                                
                                if (refreshBtn) {
                                    refreshBtn.scrollIntoView({block: 'center'});
                                    refreshBtn.click();
                                    return {success: true, text: (refreshBtn.textContent || '').trim()};
                                }
                                
                                return {success: false};
                            })()
                        """
                        
                        try:
                            result = await page.evaluate(js_refresh_button)
                            if result.get('success'):
                                print(f"[INFO] 'ANA SAYFAYI YÜKLE' tıklandı ✅", file=sys.stderr)
                                await page.wait_for_timeout(3000)
                            else:
                                await page.reload(wait_until="networkidle", timeout=15000)
                                await page.wait_for_timeout(2000)
                        except:
                            await page.reload(wait_until="networkidle", timeout=15000)
                            await page.wait_for_timeout(2000)
                        
                        new_url = page.url
                        if "/bot" in new_url:
                            print(f"[INFO] 2. deneme...", file=sys.stderr)
                            try:
                                result = await page.evaluate(js_refresh_button)
                                if result.get('success'):
                                    await page.wait_for_timeout(3000)
                                else:
                                    await page.reload(wait_until="networkidle", timeout=15000)
                                    await page.wait_for_timeout(2000)
                            except:
                                await page.reload(wait_until="networkidle", timeout=15000)
                                await page.wait_for_timeout(2000)
                            
                            final_url = page.url
                            if "/bot" in final_url:
                                print(f"[ERROR] 2 refresh sonrası hala bot sayfasında", file=sys.stderr)
                                print(json.dumps({"error": "Bot detection - 2 refresh sonrası hala bot sayfası"}), file=sys.stderr)
                                sys.exit(1)
                            else:
                                print(f"[INFO] ✅ Bot sayfası bypass edildi!", file=sys.stderr)
                        else:
                            print(f"[INFO] ✅ Bot sayfası bypass edildi!", file=sys.stderr)
                else:
                    await page.screenshot(path="debug_dashboard_timeout.png")
                    print(f"[ERROR] Dashboard'a ulaşılamadı: {current_url}", file=sys.stderr)
                    print(json.dumps({"error": f"Dashboard timeout: {current_url}"}), file=sys.stderr)
                    sys.exit(1)
            
            # ==================== QUOTE ====================
            
            # Popup'ları kapat
            print(f"[INFO] Popup'lar kapatılıyor...", file=sys.stderr)
            
            js_close_popups = """
                (() => {
                    let closed = 0;
                    
                    // "Tamam", "İleri", "Kapat", "X" butonlarını bul
                    const closeButtons = Array.from(document.querySelectorAll('button, a, [role="button"]'));
                    
                    for (const btn of closeButtons) {
                        const text = (btn.textContent || '').toLowerCase().trim();
                        const ariaLabel = (btn.getAttribute('aria-label') || '').toLowerCase();
                        
                        if (text === 'tamam' || text === 'hayır' || text === 'kapat' || 
                            text === 'x' || text === '×' || ariaLabel.includes('close')) {
                            
                            if (btn.offsetParent !== null) {
                                btn.click();
                                closed++;
                            }
                        }
                    }
                    
                    return {closed: closed};
                })()
            """
            
            try:
                result = await page.evaluate(js_close_popups)
                if result.get('closed', 0) > 0:
                    print(f"[INFO] {result['closed']} popup kapatıldı ✅", file=sys.stderr)
                    await page.wait_for_timeout(500)
            except Exception as e:
                print(f"[WARNING] Popup kapatma hatası: {str(e)[:50]}", file=sys.stderr)
            
            # "YENİ İŞ TEKLİFİ" butonuna tıkla (modal açılır)
            print(f"[INFO] YENİ İŞ TEKLİFİ butonuna tıklanıyor...", file=sys.stderr)
            
            js_new_offer = """
                (() => {
                    const buttons = Array.from(document.querySelectorAll('button'));
                    const newOfferBtn = buttons.find(b => 
                        b.offsetParent !== null && 
                        (b.textContent || '').includes('YENİ İŞ TEKLİFİ')
                    );
                    
                    if (newOfferBtn) {
                        newOfferBtn.click();
                        return {success: true};
                    }
                    
                    return {success: false};
                })()
            """
            
            try:
                result = await page.evaluate(js_new_offer)
                if result.get('success'):
                    print(f"[INFO] YENİ İŞ TEKLİFİ tıklandı ✅", file=sys.stderr)
                    await page.wait_for_timeout(800)
                else:
                    print(f"[ERROR] YENİ İŞ TEKLİFİ butonu bulunamadı ❌", file=sys.stderr)
            except Exception as e:
                print(f"[ERROR] JavaScript click hatası: {str(e)[:100]}", file=sys.stderr)
            
            # Modal'da Trafik kartı altındaki "TEKLİF AL" butonunu bul
            print(f"[INFO] Modal'da {product_type.capitalize()} kartı 'TEKLİF AL' butonu aranıyor...", file=sys.stderr)
            await page.wait_for_timeout(500)
            
            # Trafik kartını bul ve altındaki TEKLİF AL'a bas
            js_find_product_card = f"""
                (() => {{
                    const productName = '{product_type.capitalize()}';
                    
                    // Tüm div'leri ara
                    const allDivs = Array.from(document.querySelectorAll('div'));
                    
                    for (const div of allDivs) {{
                        const text = (div.textContent || '').trim();
                        
                        // "Trafik" veya "Kasko" yazısını içeren div (kart)
                        if (text.includes(productName) && div.offsetParent !== null) {{
                            // Bu div içindeki veya altındaki "TEKLİF AL" butonunu ara
                            const buttons = Array.from(div.querySelectorAll('button, a'));
                            const teklifBtn = buttons.find(b => 
                                (b.textContent || '').includes('TEKLİF AL') && 
                                b.offsetParent !== null
                            );
                            
                            if (teklifBtn) {{
                                teklifBtn.scrollIntoView({{block: 'center'}});
                                teklifBtn.click();
                                return {{success: true, card: productName, button: 'TEKLİF AL'}};
                            }}
                            
                            // Parent div'deki button'ları da dene
                            const parentDiv = div.parentElement;
                            if (parentDiv) {{
                                const parentButtons = Array.from(parentDiv.querySelectorAll('button, a'));
                                const parentTeklifBtn = parentButtons.find(b => 
                                    (b.textContent || '').includes('TEKLİF AL') && 
                                    b.offsetParent !== null
                                );
                                
                                if (parentTeklifBtn) {{
                                    parentTeklifBtn.scrollIntoView({{block: 'center'}});
                                    parentTeklifBtn.click();
                                    return {{success: true, card: productName, button: 'TEKLİF AL (parent)'}};
                                }}
                            }}
                        }}
                    }}
                    
                    return {{success: false}};
                }})()
            """
            
            teklif_al_clicked = False
            try:
                result = await page.evaluate(js_find_product_card)
                if result.get('success'):
                    print(f"[INFO] {result['card']} kartı '{result['button']}' tıklandı ✅", file=sys.stderr)
                    teklif_al_clicked = True
                    await page.wait_for_timeout(1000)
                else:
                    print(f"[ERROR] {product_type.capitalize()} kartı 'TEKLİF AL' butonu bulunamadı ❌", file=sys.stderr)
            except Exception as e:
                print(f"[ERROR] JavaScript click hatası: {str(e)[:100]}", file=sys.stderr)
            
            # Form sayfasına geçişi bekle (cosmos.sompojaoan.com.tr)
            print(f"[INFO] Form sayfasına geçiş bekleniyor (cosmos domain)...", file=sys.stderr)
            
            try:
                # URL değişimi bekle - cosmos domain'e geçmeli
                await page.wait_for_url(lambda url: "cosmos" in url or "ejento.somposigorta.com.tr" not in url, timeout=10000)
                print(f"[INFO] Form sayfasına geçildi: {page.url}", file=sys.stderr)
            except:
                print(f"[WARNING] URL timeout, current: {page.url}", file=sys.stderr)
            
            # Network idle + loading spinner bekleme
            print(f"[INFO] Form yükleniyor...", file=sys.stderr)
            await page.wait_for_load_state("networkidle", timeout=15000)
            
            # Loading spinner bitmesini bekle
            print(f"[INFO] Loading spinner bekleniyor...", file=sys.stderr)
            try:
                # Loading, spinner, overlay gibi elementlerin kaybolmasını bekle
                await page.wait_for_function("""
                    () => {
                        const loadingElements = document.querySelectorAll('.loading, .spinner, .overlay, [class*="loading"], [class*="spinner"]');
                        return Array.from(loadingElements).every(el => el.offsetParent === null || window.getComputedStyle(el).display === 'none');
                    }
                """, timeout=10000)
                print(f"[INFO] Loading tamamlandı ✅", file=sys.stderr)
            except:
                print(f"[WARNING] Loading timeout (devam ediliyor)", file=sys.stderr)
            
            # Ekstra bekleme - form elementleri render olsun
            await page.wait_for_timeout(2000)  # 1s -> 2s
            
            # Form screenshot
            await page.screenshot(path="debug_form_page.png", full_page=True)
            print(f"[DEBUG] Form sayfası screenshot: debug_form_page.png", file=sys.stderr)
            print(f"[DEBUG] Form URL: {page.url}", file=sys.stderr)
            
            # Checkbox'ların yüklenmesini bekle
            print(f"[INFO] Form elementleri bekleniyor...", file=sys.stderr)
            try:
                # Checkbox'ların DOM'a yüklenmesini bekle
                await page.wait_for_selector('input[type="checkbox"]', timeout=5000)
                print(f"[INFO] Checkbox'lar yüklendi ✅", file=sys.stderr)
            except:
                print(f"[WARNING] Checkbox timeout (devam ediliyor)", file=sys.stderr)
            
            await page.wait_for_timeout(500)
            
            # Debug: Tüm checkbox'ları logla
            try:
                all_checkboxes = await page.evaluate("""
                    (() => {
                        const checkboxes = Array.from(document.querySelectorAll('input[type="checkbox"]'));
                        return checkboxes.map(cb => ({
                            checked: cb.checked,
                            label: cb.labels && cb.labels[0] ? cb.labels[0].textContent.trim() : '',
                            nextText: cb.nextSibling ? cb.nextSibling.textContent.trim() : '',
                            visible: cb.offsetParent !== null
                        }));
                    })()
                """)
                print(f"[DEBUG] Checkbox'lar ({len(all_checkboxes)}):", file=sys.stderr)
                for i, cb in enumerate(all_checkboxes):
                    print(f"  {i+1}. {'☑' if cb['checked'] else '☐'} {cb['label'] or cb['nextText']} (visible: {cb['visible']})", file=sys.stderr)
            except:
                pass
            
            # Trafik/Kasko checkbox'ını seç
            print(f"[INFO] {product_type.capitalize()} checkbox'ı seçiliyor...", file=sys.stderr)
            
            js_select_checkbox = f"""
                (() => {{
                    const productType = '{product_type}';  // "trafik" veya "kasko"
                    const checkboxes = Array.from(document.querySelectorAll('input[type="checkbox"]'));
                    
                    let trafikCb = null;
                    let kaskoCb = null;
                    
                    // Trafik ve Kasko checkbox'larını bul
                    for (const cb of checkboxes) {{
                        const label = cb.labels && cb.labels[0] ? cb.labels[0].textContent : '';
                        const nextText = cb.nextSibling ? cb.nextSibling.textContent : '';
                        const text = (label + nextText).toLowerCase();
                        
                        if (text.includes('trafik') && !text.includes('kasko')) {{
                            trafikCb = cb;
                        }} else if (text.includes('kasko')) {{
                            kaskoCb = cb;
                        }}
                    }}
                    
                    // Doğru checkbox'ı seç
                    if (productType === 'trafik') {{
                        // Kasko'yu kaldır, Trafik'i seç
                        if (kaskoCb && kaskoCb.checked) {{
                            kaskoCb.click();
                        }}
                        if (trafikCb && !trafikCb.checked) {{
                            trafikCb.click();
                            return {{success: true, selected: 'Trafik', deselected: 'Kasko'}};
                        }}
                        return {{success: trafikCb !== null, selected: 'Trafik'}};
                    }} else if (productType === 'kasko') {{
                        // Trafik'i kaldır, Kasko'yu seç
                        if (trafikCb && trafikCb.checked) {{
                            trafikCb.click();
                        }}
                        if (kaskoCb && !kaskoCb.checked) {{
                            kaskoCb.click();
                            return {{success: true, selected: 'Kasko', deselected: 'Trafik'}};
                        }}
                        return {{success: kaskoCb !== null, selected: 'Kasko'}};
                    }}
                    
                    return {{success: false}};
                }})()
            """
            
            try:
                result = await page.evaluate(js_select_checkbox)
                if result.get('success'):
                    msg = f"{result.get('selected', product_type.capitalize())} seçildi"
                    if result.get('deselected'):
                        msg += f" ({result['deselected']} kaldırıldı)"
                    print(f"[INFO] {msg} ✅", file=sys.stderr)
                else:
                    print(f"[WARNING] {product_type.capitalize()} checkbox bulunamadı ❌", file=sys.stderr)
            except Exception as e:
                print(f"[WARNING] Checkbox seçimi hatası: {str(e)[:50]}", file=sys.stderr)
            
            # Text input'ların yüklenmesini bekle
            try:
                await page.wait_for_selector('input[type="text"], input:not([type])', timeout=5000)
                print(f"[INFO] Text input'lar yüklendi ✅", file=sys.stderr)
            except:
                print(f"[WARNING] Text input timeout (devam ediliyor)", file=sys.stderr)
            
            await page.wait_for_timeout(500)
            
            # Debug: Tüm text input'ları logla
            try:
                all_inputs = await page.evaluate("""
                    (() => {
                        const inputs = Array.from(document.querySelectorAll('input[type="text"], input:not([type])'));
                        return inputs.map(inp => ({
                            placeholder: inp.placeholder || '',
                            name: inp.name || '',
                            value: inp.value || '',
                            visible: inp.offsetParent !== null,
                            disabled: inp.disabled
                        }));
                    })()
                """)
                print(f"[DEBUG] Text input'lar ({len(all_inputs)}):", file=sys.stderr)
                for i, inp in enumerate(all_inputs[:10]):
                    status = "✓" if inp['visible'] and not inp['disabled'] else "✗"
                    print(f"  {i+1}. {status} name={inp['name']}, placeholder={inp['placeholder']}, value={inp['value'][:20]}", file=sys.stderr)
            except:
                pass
            
            # Araç Plakası doldur
            print(f"[INFO] Araç Plakası dolduruluyor: {plate}", file=sys.stderr)
            
            js_fill_plate = f"""
                (() => {{
                    const plate = '{plate}';
                    
                    // Tüm text input'ları bul
                    const inputs = Array.from(document.querySelectorAll('input[type="text"], input:not([type])'));
                    
                    for (const inp of inputs) {{
                        // Input'un etrafındaki text'leri kontrol et
                        const placeholder = (inp.placeholder || '').toLowerCase();
                        const name = (inp.name || '').toLowerCase();
                        const id = (inp.id || '').toLowerCase();
                        
                        // Label kontrolü (multiple yöntem)
                        let labelText = '';
                        
                        // 1. labels property
                        if (inp.labels && inp.labels[0]) {{
                            labelText = inp.labels[0].textContent.toLowerCase();
                        }}
                        
                        // 2. previousElementSibling (Araç Plakası: gibi)
                        let prev = inp.previousElementSibling;
                        while (prev && !labelText) {{
                            const text = (prev.textContent || '').toLowerCase();
                            if (text.includes('araç') || text.includes('plak')) {{
                                labelText = text;
                                break;
                            }}
                            prev = prev.previousElementSibling;
                        }}
                        
                        // 3. Parent element içindeki text
                        if (!labelText && inp.parentElement) {{
                            const parentText = (inp.parentElement.textContent || '').toLowerCase();
                            if (parentText.includes('araç plaka')) {{
                                labelText = parentText;
                            }}
                        }}
                        
                        // Araç Plakası mı?
                        if (placeholder.includes('plak') || name.includes('plak') || id.includes('plak') ||
                            labelText.includes('araç') && labelText.includes('plak')) {{
                            
                            // Görünür mü?
                            if (inp.offsetParent !== null && !inp.disabled) {{
                                inp.scrollIntoView({{block: 'center'}});
                                inp.focus();
                                inp.value = plate;
                                inp.dispatchEvent(new Event('input', {{bubbles: true}}));
                                inp.dispatchEvent(new Event('change', {{bubbles: true}}));
                                inp.dispatchEvent(new Event('blur', {{bubbles: true}}));
                                
                                return {{success: true, field: 'Araç Plakası', method: 'found'}};
                            }}
                        }}
                    }}
                    
                    return {{success: false}};
                }})()
            """
            
            try:
                result = await page.evaluate(js_fill_plate)
                if result.get('success'):
                    print(f"[INFO] {result['field']} dolduruldu ✅", file=sys.stderr)
                    
                    # Validation: Value gerçekten set edildi mi?
                    await page.wait_for_timeout(300)
                    plate_value = await page.evaluate(f"""
                        (() => {{
                            const inputs = Array.from(document.querySelectorAll('input[type="text"], input:not([type])'));
                            for (const inp of inputs) {{
                                const placeholder = (inp.placeholder || '').toLowerCase();
                                const name = (inp.name || '').toLowerCase();
                                if (placeholder.includes('plak') || name.includes('plak')) {{
                                    return inp.value;
                                }}
                            }}
                            return '';
                        }})()
                    """)
                    
                    if plate_value == plate:
                        print(f"[INFO] Plaka validation OK: {plate_value} ✅", file=sys.stderr)
                    else:
                        print(f"[WARNING] Plaka validation FAILED: expected={plate}, got={plate_value} ❌", file=sys.stderr)
                else:
                    print(f"[WARNING] Araç Plakası input bulunamadı ❌", file=sys.stderr)
            except Exception as e:
                print(f"[ERROR] Plaka doldurma hatası: {str(e)[:100]}", file=sys.stderr)
            
            await page.wait_for_timeout(500)
            
            # "Teklif Oluştur" butonu
            print(f"[INFO] 'Teklif Oluştur' butonu aranıyor...", file=sys.stderr)
            
            js_submit = """
                (() => {
                    // Tüm button ve input[type="button"] elementlerini ara
                    const buttons = Array.from(document.querySelectorAll('button, input[type="button"], input[type="submit"], a[role="button"]'));
                    
                    for (const btn of buttons) {
                        const text = (btn.textContent || btn.value || '').trim();
                        
                        if (text.includes('Teklif Oluştur') || text.includes('TEKLİF OLUŞTUR') ||
                            text.includes('Teklif Al') || text.includes('TEKLİF AL') ||
                            text.includes('Sorgula') || text.includes('SORGULA')) {
                            
                            // Görünür mü?
                            if (btn.offsetParent !== null && !btn.disabled) {
                                btn.scrollIntoView({block: 'center'});
                                
                                // Sayfanın en altına scroll (buton aşağıda olabilir)
                                window.scrollTo(0, document.body.scrollHeight);
                                
                                setTimeout(() => {
                                    btn.click();
                                }, 200);
                                
                                return {
                                    success: true,
                                    text: text.substring(0, 50)
                                };
                            }
                        }
                    }
                    
                    return {success: false};
                })()
            """
            
            try:
                submit_result = await page.evaluate(js_submit)
                await page.wait_for_timeout(300)  # setTimeout için bekle
                
                if submit_result.get('success'):
                    print(f"[INFO] '{submit_result.get('text', 'Teklif Oluştur')}' tıklandı ✅", file=sys.stderr)
                else:
                    print(f"[WARNING] 'Teklif Oluştur' butonu bulunamadı ❌", file=sys.stderr)
                    # Debug: Tüm butonları logla
                    all_buttons = await page.evaluate("""
                        (() => {
                            const btns = Array.from(document.querySelectorAll('button, input[type="button"], input[type="submit"]'));
                            return btns.filter(b => b.offsetParent !== null).map(b => ({
                                text: (b.textContent || b.value || '').trim().substring(0, 50),
                                type: b.type || b.tagName
                            }));
                        })()
                    """)
                    print(f"[DEBUG] Görünen button'lar ({len(all_buttons)}):", file=sys.stderr)
                    for i, btn in enumerate(all_buttons[:10]):
                        print(f"  {i+1}. {btn['type']}: '{btn['text']}'", file=sys.stderr)
            except Exception as e:
                print(f"[ERROR] Submit hatası: {str(e)[:100]}", file=sys.stderr)
            
            # Sonuçları bekle
            print(f"[INFO] Sonuçlar bekleniyor...", file=sys.stderr)
            await page.wait_for_timeout(3000)  # 5s -> 3s (hızlandırma)
            await page.wait_for_load_state("networkidle", timeout=15000)  # 20s -> 15s (hızlandırma)
            
            # ==================== PARSE ====================
            
            # Sonuç sayfası screenshot
            await page.screenshot(path="debug_results.png", full_page=True)
            print(f"[DEBUG] Sonuç sayfası screenshot: debug_results.png", file=sys.stderr)
            
            # Tüm fiyat-like elementleri logla
            print(f"[INFO] Fiyat aranıyor...", file=sys.stderr)
            try:
                price_candidates = await page.evaluate("""
                    () => {
                        const elements = Array.from(document.querySelectorAll('div, span, p, td'));
                        const tlRegex = /(\\d{1,3}(\\.\\d{3})*(,\\d{2})?\\s*(TL|₺))/;
                        
                        return elements
                            .filter(el => el.offsetParent !== null && tlRegex.test(el.textContent))
                            .slice(0, 15)
                            .map(el => ({
                                text: (el.textContent || '').trim().substring(0, 100),
                                class: el.className || '',
                                tag: el.tagName
                            }));
                    }
                """)
                print(f"[DEBUG] Fiyat adayları ({len(price_candidates)}):", file=sys.stderr)
                for i, pc in enumerate(price_candidates):
                    print(f"  {i+1}. {pc['tag']}.{pc['class']}: '{pc['text']}'", file=sys.stderr)
            except:
                pass
            
            # JavaScript ile fiyat bul - en yüksek değer
            js_find_price = """
                (() => {
                    const tlRegex = /(\\d{1,3}(\\.\\d{3})*(,\\d{2})?\\s*(TL|₺))/g;
                    const elements = Array.from(document.querySelectorAll('div, span, p, td, [class*="prem"], [class*="prim"], [class*="price"], [class*="fiyat"]'));
                    
                    let maxPrice = 0;
                    let maxPriceText = '';
                    
                    for (const el of elements) {
                        if (el.offsetParent !== null) {
                            const text = el.textContent || '';
                            const matches = text.match(tlRegex);
                            
                            if (matches && matches.length > 0) {
                                for (const match of matches) {
                                    // TL fiyatı parse et
                                    const cleanedMatch = match.replace(/TL|₺/g, '').replace(/\\./g, '').replace(',', '.').trim();
                                    const price = parseFloat(cleanedMatch);
                                    
                                    if (price > 100 && price < 100000 && price > maxPrice) {
                                        maxPrice = price;
                                        maxPriceText = match;
                                    }
                                }
                            }
                        }
                    }
                    
                    return {
                        success: maxPrice > 0,
                        price: maxPriceText,
                        value: maxPrice
                    };
                })()
            """
            
            price = 0.0
            price_text = ""
            
            try:
                price_result = await page.evaluate(js_find_price)
                
                if price_result.get('success'):
                    price_text = price_result.get('price')
                    price = price_result.get('value')
                    print(f"[INFO] Fiyat bulundu: {price_text} (={price} TL) ✅", file=sys.stderr)
                else:
                    print(f"[ERROR] Fiyat bulunamadı ❌", file=sys.stderr)
                    await page.screenshot(path="debug_no_price.png", full_page=True)
                    print(json.dumps({"error": "Fiyat elementi bulunamadı"}), file=sys.stderr)
                    sys.exit(1)
            except Exception as e:
                print(f"[ERROR] Fiyat parse hatası: {str(e)[:100]}", file=sys.stderr)
                await page.screenshot(path="debug_price_error.png", full_page=True)
                print(json.dumps({"error": f"Fiyat parse hatası: {str(e)[:100]}"}), file=sys.stderr)
                sys.exit(1)
            
            print(f"[INFO] Fiyat bulundu: {price_text} -> {price} TL", file=sys.stderr)
            
            # ==================== RESPONSE ====================
            
            elapsed_ms = int((time.time() - start_time) * 1000)
            
            net = price / 1.18
            taxes = price - net
            
            response = {
                "success": True,
                "company": "Sompo",
                "product_type": product_type,
                "premium": {
                    "net": round(net, 2),
                    "gross": round(price, 2),
                    "taxes": round(taxes, 2),
                    "currency": "TRY"
                },
                "installments": [
                    {"count": 1, "per_installment": round(price, 2), "total": round(price, 2)},
                    {"count": 3, "per_installment": round(price / 3, 2), "total": round(price, 2)}
                ],
                "coverages": [
                    {
                        "code": "TRAFIK_ZORUNLU" if product_type == "trafik" else "KASKO_TAM",
                        "name": "Zorunlu Trafik Sigortası" if product_type == "trafik" else "Tam Kasko",
                        "limit": None,
                        "included": True
                    }
                ],
                "warnings": [],
                "timings": {
                    "scrape_ms": elapsed_ms
                }
            }
            
            # JSON output (stdout)
            print(json.dumps(response, ensure_ascii=False))
            
            print(f"[INFO] Scraping tamamlandı: {elapsed_ms}ms", file=sys.stderr)
            
        except PWTimeout as e:
            await page.screenshot(path="debug_timeout_playwright.png")
            print(f"[ERROR] Timeout: {str(e)}", file=sys.stderr)
            print(json.dumps({"error": f"Timeout: {str(e)}"}), file=sys.stderr)
            sys.exit(1)
        except Exception as e:
            await page.screenshot(path="debug_error_playwright.png")
            print(f"[ERROR] Exception: {str(e)}", file=sys.stderr)
            print(json.dumps({"error": str(e)}), file=sys.stderr)
            sys.exit(1)
        finally:
            await browser.close()
            print(f"[INFO] Browser kapatıldı", file=sys.stderr)

if __name__ == "__main__":
    asyncio.run(main())

