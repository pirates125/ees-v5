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
            
            # Username - Playwright fill() otomatik temizler
            username_selector = 'input[type="text"], input[name="username"]'
            await page.fill(username_selector, username)
            
            # Validation
            input_value = await page.input_value(username_selector)
            if input_value == username:
                print(f"[INFO] Username girildi: {username}", file=sys.stderr)
            else:
                print(f"[WARNING] Username validation failed! Expected: {username}, Got: {input_value}", file=sys.stderr)
            
            # Password - Playwright fill() otomatik temizler
            password_selector = 'input[type="password"]'
            await page.fill(password_selector, password)
            
            # Validation (length check)
            input_value = await page.input_value(password_selector)
            if len(input_value) == len(password):
                print(f"[INFO] Password girildi (len={len(password)})", file=sys.stderr)
            else:
                print(f"[WARNING] Password validation failed! Expected len: {len(password)}, Got: {len(input_value)}", file=sys.stderr)
            
            # Login button
            await page.click('button[type="submit"]')
            print(f"[INFO] Login button tıklandı", file=sys.stderr)
            
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
                print(f"[INFO] Dashboard'a ulaşıldı: {page.url}", file=sys.stderr)
            except:
                # Timeout ama dashboard'da olabiliriz
                current_url = page.url
                if "dashboard" in current_url and "login" not in current_url:
                    print(f"[INFO] Dashboard'a ulaşıldı (timeout ama URL doğru): {current_url}", file=sys.stderr)
                else:
                    await page.screenshot(path="debug_dashboard_timeout.png")
                    print(f"[ERROR] Dashboard'a ulaşılamadı: {current_url}", file=sys.stderr)
                    print(json.dumps({"error": f"Dashboard timeout: {current_url}"}), file=sys.stderr)
                    sys.exit(1)
            
            # ==================== QUOTE ====================
            
            # Trafik/Kasko linki ara (Playwright'ın has-text() kullan)
            print(f"[INFO] {product_type.capitalize()} linki aranıyor...", file=sys.stderr)
            
            # Ürün linklerini dene
            product_keywords = {
                'trafik': ['Trafik', 'trafik'],
                'kasko': ['Kasko', 'kasko']
            }
            keywords = product_keywords.get(product_type.lower(), ['Trafik'])
            
            link_clicked = False
            for keyword in keywords:
                try:
                    # Playwright'ın has-text() kullan
                    link = await page.query_selector(f'a:has-text("{keyword}"), button:has-text("{keyword}")')
                    if link:
                        await link.click()
                        print(f"[INFO] {keyword} linki tıklandı", file=sys.stderr)
                        link_clicked = True
                        break
                except:
                    continue
            
            if not link_clicked:
                print(f"[WARNING] Ürün linki bulunamadı, alternatif yöntem deneniyor", file=sys.stderr)
            
            # Sayfa yüklensin
            await page.wait_for_load_state("networkidle", timeout=10000)
            
            # Form doldur - Plaka
            print(f"[INFO] Form dolduruluyor: Plaka={plate}, TCKN={tckn}", file=sys.stderr)
            
            # Plaka input selectors
            plate_selectors = [
                'input[name*="plak"]',
                'input[placeholder*="lak"]',
                'input[placeholder*="late"]'
            ]
            
            for selector in plate_selectors:
                try:
                    if await page.query_selector(selector):
                        await page.fill(selector, plate)
                        print(f"[INFO] Plaka dolduruldu: {selector}", file=sys.stderr)
                        break
                except:
                    continue
            
            # TCKN input selectors
            tckn_selectors = [
                'input[name*="tc"]',
                'input[name*="kimlik"]',
                'input[placeholder*="TC"]',
                'input[placeholder*="Kimlik"]'
            ]
            
            for selector in tckn_selectors:
                try:
                    if await page.query_selector(selector):
                        await page.fill(selector, tckn)
                        print(f"[INFO] TCKN dolduruldu: {selector}", file=sys.stderr)
                        break
                except:
                    continue
            
            # Submit button
            print(f"[INFO] Submit butonu aranıyor...", file=sys.stderr)
            
            submit_selectors = [
                'button:has-text("Teklif Al")',
                'button:has-text("Sorgula")',
                'button:has-text("Hesapla")',
                'button[type="submit"]'
            ]
            
            for selector in submit_selectors:
                try:
                    if await page.query_selector(selector):
                        await page.click(selector)
                        print(f"[INFO] Submit button tıklandı: {selector}", file=sys.stderr)
                        break
                except:
                    continue
            
            # Sonuçları bekle
            print(f"[INFO] Sonuçlar bekleniyor...", file=sys.stderr)
            await page.wait_for_timeout(10000)  # 10 saniye
            
            # ==================== PARSE ====================
            
            # Fiyat bul - Playwright selectors
            price = 0.0
            price_text = ""
            
            price_selectors = [
                '.premium',
                '.prim',
                '.amount',
                'text="TL"',  # Playwright text selector
                '*:has-text("TL")'
            ]
            
            for selector in price_selectors:
                try:
                    elements = await page.query_selector_all(selector)
                    for el in elements:
                        text = await el.text_content()
                        if text and 'TL' in text and len(text) < 50:
                            parsed = parse_tl_price(text)
                            if 100 < parsed < 100000:
                                if parsed > price:
                                    price = parsed
                                    price_text = text
                except:
                    continue
            
            if price == 0.0:
                # Screenshot al
                await page.screenshot(path="debug_no_price_playwright.png")
                print(f"[ERROR] Fiyat bulunamadı", file=sys.stderr)
                print(json.dumps({"error": "Fiyat elementi bulunamadı"}), file=sys.stderr)
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

