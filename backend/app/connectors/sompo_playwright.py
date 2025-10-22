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
                    
                    # "Ana Sayfayı Yenile" butonunu ara ve tıkla
                    print(f"[INFO] 🔄 'Ana Sayfayı Yenile' butonu aranıyor...", file=sys.stderr)
                    
                    js_refresh_button = """
                        (() => {
                            const buttons = Array.from(document.querySelectorAll('button, a'));
                            const refreshBtn = buttons.find(b => 
                                b.offsetParent !== null && 
                                ((b.textContent || '').toLowerCase().includes('ana sayfa') ||
                                 (b.textContent || '').toLowerCase().includes('yenile'))
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
                            print(f"[INFO] 'Ana Sayfayı Yenile' tıklandı: {result.get('text', 'unknown')}", file=sys.stderr)
                            refresh_clicked = True
                            await page.wait_for_timeout(3000)
                        else:
                            print(f"[WARNING] Refresh butonu bulunamadı, page.reload() deneniyor", file=sys.stderr)
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
                                print(f"[INFO] 'Ana Sayfayı Yenile' tıklandı (2. deneme)", file=sys.stderr)
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
                        
                        # "Ana Sayfayı Yenile" butonunu ara
                        js_refresh_button = """
                            (() => {
                                const buttons = Array.from(document.querySelectorAll('button, a'));
                                const refreshBtn = buttons.find(b => 
                                    b.offsetParent !== null && 
                                    ((b.textContent || '').toLowerCase().includes('ana sayfa') ||
                                     (b.textContent || '').toLowerCase().includes('yenile'))
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
                                print(f"[INFO] 'Ana Sayfayı Yenile' tıklandı", file=sys.stderr)
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
            
            # Dashboard screenshot al
            await page.screenshot(path="debug_dashboard_ready.png", full_page=True)
            print(f"[DEBUG] Dashboard screenshot: debug_dashboard_ready.png", file=sys.stderr)
            
            # Dashboard'daki tüm linkleri logla
            try:
                links_and_buttons = await page.evaluate("""
                    () => {
                        const elements = Array.from(document.querySelectorAll('a, button'));
                        return elements.slice(0, 30).map(el => ({
                            tag: el.tagName,
                            text: (el.textContent || '').trim().substring(0, 80),
                            href: el.href || '',
                            visible: el.offsetParent !== null
                        })).filter(e => e.visible && e.text);
                    }
                """)
                print(f"[DEBUG] Dashboard elementleri ({len(links_and_buttons)}):", file=sys.stderr)
                for i, elem in enumerate(links_and_buttons[:15]):
                    print(f"  {i+1}. {elem['tag']}: '{elem['text']}'", file=sys.stderr)
            except:
                pass
            
            # "YENİ İŞ TEKLİFİ" butonuna tıkla (modal açılır)
            print(f"[INFO] YENİ İŞ TEKLİFİ butonuna tıklanıyor...", file=sys.stderr)
            
            # Playwright native selectors dene (has-text)
            new_offer_clicked = False
            selectors = [
                'button:has-text("YENİ İŞ TEKLİFİ")',
                'button:has-text("Yeni İş Teklifi")',
                'button:has-text("yeni iş")',
            ]
            
            for selector in selectors:
                try:
                    btn = await page.query_selector(selector)
                    if btn:
                        await btn.click()
                        print(f"[INFO] YENİ İŞ TEKLİFİ tıklandı ✅ (selector: {selector})", file=sys.stderr)
                        new_offer_clicked = True
                        await page.wait_for_timeout(2000)
                        break
                except Exception as e:
                    print(f"[DEBUG] {selector} denendi, hata: {str(e)[:50]}", file=sys.stderr)
                    continue
            
            if not new_offer_clicked:
                print(f"[WARNING] YENİ İŞ TEKLİFİ butonu bulunamadı, JavaScript ile deneniyor", file=sys.stderr)
                
                js_new_offer = """
                    (() => {
                        const buttons = Array.from(document.querySelectorAll('button'));
                        const newOfferBtn = buttons.find(b => 
                            b.offsetParent !== null && 
                            (b.textContent || '').toLowerCase().includes('yeni')
                        );
                        
                        if (newOfferBtn) {
                            newOfferBtn.scrollIntoView({block: 'center'});
                            newOfferBtn.click();
                            return {success: true, text: (newOfferBtn.textContent || '').trim()};
                        }
                        
                        return {success: false};
                    })()
                """
                
                try:
                    result = await page.evaluate(js_new_offer)
                    if result.get('success'):
                        print(f"[INFO] Button tıklandı (JS): {result.get('text', 'unknown')}", file=sys.stderr)
                        new_offer_clicked = True
                        await page.wait_for_timeout(2000)
                    else:
                        print(f"[ERROR] YENİ İŞ TEKLİFİ butonu bulunamadı ❌", file=sys.stderr)
                except Exception as e:
                    print(f"[ERROR] JavaScript click hatası: {str(e)[:100]}", file=sys.stderr)
            
            # Modal açılınca Trafik/Kasko seç
            print(f"[INFO] Modal'da {product_type.capitalize()} seçiliyor...", file=sys.stderr)
            
            # Modal'ın açılmasını bekle
            await page.wait_for_timeout(1000)
            
            # Playwright native selectors (Trafik/Kasko)
            product_clicked = False
            product_selectors = [
                f'button:has-text("{product_type.capitalize()}")',
                f'a:has-text("{product_type.capitalize()}")',
                f'button:has-text("{product_type.upper()}")',
                f'a:has-text("{product_type.upper()}")',
            ]
            
            for selector in product_selectors:
                try:
                    btn = await page.query_selector(selector)
                    if btn:
                        await btn.click()
                        print(f"[INFO] {product_type.capitalize()} seçildi ✅ (selector: {selector})", file=sys.stderr)
                        product_clicked = True
                        await page.wait_for_timeout(2000)
                        break
                except Exception as e:
                    print(f"[DEBUG] {selector} denendi, hata: {str(e)[:50]}", file=sys.stderr)
                    continue
            
            if not product_clicked:
                print(f"[WARNING] Playwright selector başarısız, JavaScript ile deneniyor", file=sys.stderr)
                
                js_select_product = f"""
                    (() => {{
                        const productType = '{product_type}';
                        
                        // Modal içindeki tüm elementleri ara
                        const allElements = Array.from(document.querySelectorAll('div, button, a, span'));
                        
                        for (const el of allElements) {{
                            const text = (el.textContent || '').toLowerCase().trim();
                            
                            // "Trafik" veya "Kasko" yazısını içeren element
                            if (text === productType || text.includes(productType)) {{
                                // Tıklanabilir mi?
                                if (el.tagName === 'BUTTON' || el.tagName === 'A' || el.onclick || el.getAttribute('role') === 'button') {{
                                    el.scrollIntoView({{block: 'center'}});
                                    el.click();
                                    return {{success: true, text: text.substring(0, 50)}};
                                }}
                                
                                // Parent'ı dene
                                const parent = el.parentElement;
                                if (parent && (parent.tagName === 'BUTTON' || parent.tagName === 'A' || parent.onclick)) {{
                                    parent.scrollIntoView({{block: 'center'}});
                                    parent.click();
                                    return {{success: true, text: text.substring(0, 50)}};
                                }}
                            }}
                        }}
                        
                        return {{success: false}};
                    }})()
                """
                
                try:
                    result = await page.evaluate(js_select_product)
                    if result.get('success'):
                        print(f"[INFO] {product_type.capitalize()} seçildi (JS): {result.get('text', 'unknown')}", file=sys.stderr)
                        product_clicked = True
                        await page.wait_for_timeout(2000)
                    else:
                        print(f"[ERROR] Modal'da {product_type} bulunamadı ❌", file=sys.stderr)
                except Exception as e:
                    print(f"[ERROR] Ürün seçimi hatası: {str(e)[:100]}", file=sys.stderr)
            
            if not product_clicked:
                await page.screenshot(path="debug_modal_not_found.png", full_page=True)
                print(f"[ERROR] Ürün seçilemedi, screenshot: debug_modal_not_found.png", file=sys.stderr)
            
            # Sayfa yüklensin ve URL değişimini bekle
            try:
                # URL değişimi bekle (trafik/kasko form sayfası)
                await page.wait_for_url(lambda url: "trafik" in url.lower() or "kasko" in url.lower() or url != page.url, timeout=10000)
                print(f"[INFO] Form sayfasına geçildi: {page.url}", file=sys.stderr)
            except:
                print(f"[WARNING] URL değişmedi, devam ediliyor: {page.url}", file=sys.stderr)
            
            await page.wait_for_load_state("networkidle", timeout=10000)
            
            # Form screenshot
            await page.screenshot(path="debug_before_form.png", full_page=True)
            print(f"[DEBUG] Form sayfası screenshot: debug_before_form.png", file=sys.stderr)
            print(f"[DEBUG] Current URL: {page.url}", file=sys.stderr)
            
            # Form doldur - Plaka ve TCKN
            print(f"[INFO] Form dolduruluyor: Plaka={plate}, TCKN={tckn}", file=sys.stderr)
            
            # Tüm input'ları logla
            try:
                inputs = await page.evaluate("""
                    () => {
                        const inputs = Array.from(document.querySelectorAll('input:not([type="hidden"])'));
                        return inputs.filter(i => i.offsetParent !== null).map(inp => ({
                            name: inp.name || '',
                            placeholder: inp.placeholder || '',
                            type: inp.type || '',
                            id: inp.id || ''
                        }));
                    }
                """)
                print(f"[DEBUG] Görünen input'lar ({len(inputs)}):", file=sys.stderr)
                for i, inp in enumerate(inputs[:10]):
                    print(f"  {i+1}. type={inp['type']}, name={inp['name']}, placeholder={inp['placeholder']}", file=sys.stderr)
            except:
                pass
            
            # JavaScript ile form doldur
            js_fill_form = f"""
                (() => {{
                    const plate = '{plate}';
                    const tckn = '{tckn}';
                    
                    const inputs = Array.from(document.querySelectorAll('input:not([type="hidden"])'));
                    const visibleInputs = inputs.filter(i => i.offsetParent !== null && !i.disabled);
                    
                    let plataDone = false;
                    let tcknDone = false;
                    
                    for (const inp of visibleInputs) {{
                        const placeholder = (inp.placeholder || '').toLowerCase();
                        const name = (inp.name || '').toLowerCase();
                        const label = inp.labels && inp.labels[0] ? inp.labels[0].textContent.toLowerCase() : '';
                        
                        // Plaka
                        if (!plataDone && (placeholder.includes('plak') || name.includes('plak') || label.includes('plak'))) {{
                            inp.focus();
                            inp.value = plate;
                            inp.dispatchEvent(new Event('input', {{bubbles: true}}));
                            inp.dispatchEvent(new Event('change', {{bubbles: true}}));
                            plataDone = true;
                            continue;
                        }}
                        
                        // TCKN
                        if (!tcknDone && (placeholder.includes('tc') || name.includes('tc') || label.includes('tc') || 
                                          placeholder.includes('kimlik') || name.includes('kimlik') || placeholder.includes('vkn'))) {{
                            inp.focus();
                            inp.value = tckn;
                            inp.dispatchEvent(new Event('input', {{bubbles: true}}));
                            inp.dispatchEvent(new Event('change', {{bubbles: true}}));
                            tcknDone = true;
                            continue;
                        }}
                    }}
                    
                    return {{plaka: plataDone, tckn: tcknDone}};
                }})()
            """
            
            try:
                form_result = await page.evaluate(js_fill_form)
                if form_result.get('plaka'):
                    print(f"[INFO] Plaka dolduruldu ✅", file=sys.stderr)
                else:
                    print(f"[WARNING] Plaka input bulunamadı ❌", file=sys.stderr)
                
                if form_result.get('tckn'):
                    print(f"[INFO] TCKN dolduruldu ✅", file=sys.stderr)
                else:
                    print(f"[WARNING] TCKN input bulunamadı ❌", file=sys.stderr)
            except Exception as e:
                print(f"[ERROR] Form doldurma hatası: {str(e)[:100]}", file=sys.stderr)
            
            await page.wait_for_timeout(2000)
            
            # Submit button
            print(f"[INFO] Submit butonu aranıyor...", file=sys.stderr)
            
            # Tüm button'ları logla
            try:
                buttons = await page.evaluate("""
                    () => {
                        const buttons = Array.from(document.querySelectorAll('button:not([disabled])'));
                        return buttons.filter(b => b.offsetParent !== null).map(btn => ({
                            text: (btn.textContent || '').trim().substring(0, 50),
                            type: btn.type || '',
                            class: btn.className || ''
                        }));
                    }
                """)
                print(f"[DEBUG] Görünen button'lar ({len(buttons)}):", file=sys.stderr)
                for i, btn in enumerate(buttons[:10]):
                    print(f"  {i+1}. type={btn['type']}, text='{btn['text']}'", file=sys.stderr)
            except:
                pass
            
            # JavaScript ile submit button bul ve tıkla
            js_submit = """
                (() => {
                    const keywords = ['teklif', 'sorgula', 'hesapla', 'devam', 'ara'];
                    const buttons = Array.from(document.querySelectorAll('button:not([disabled])'));
                    const visibleButtons = buttons.filter(b => b.offsetParent !== null);
                    
                    for (const btn of visibleButtons) {
                        const text = (btn.textContent || btn.innerText || '').toLowerCase();
                        
                        if (keywords.some(kw => text.includes(kw))) {
                            btn.scrollIntoView({block: 'center'});
                            btn.click();
                            
                            return {
                                success: true,
                                text: text.substring(0, 50)
                            };
                        }
                    }
                    
                    return {success: false};
                })()
            """
            
            submit_clicked = False
            try:
                submit_result = await page.evaluate(js_submit)
                if submit_result.get('success'):
                    print(f"[INFO] Submit butonu tıklandı: {submit_result.get('text', 'unknown')}", file=sys.stderr)
                    submit_clicked = True
                else:
                    print(f"[WARNING] Submit butonu bulunamadı ❌", file=sys.stderr)
            except Exception as e:
                print(f"[ERROR] Submit hatası: {str(e)[:100]}", file=sys.stderr)
            
            # Sonuçları bekle
            print(f"[INFO] Sonuçlar bekleniyor...", file=sys.stderr)
            await page.wait_for_timeout(5000)
            await page.wait_for_load_state("networkidle", timeout=20000)
            
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

