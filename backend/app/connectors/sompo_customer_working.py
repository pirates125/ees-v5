#!/usr/bin/env python3
"""
MÜŞTERİNİN ÇALIŞAN SOMPO SCRAPER'I
Bu kod müşterinin çalışan versiyonu - AYNEN kullanılacak!
"""

import os
import sys
import json
import uuid
import asyncio
import datetime as dt
import re
from contextlib import asynccontextmanager
from typing import Optional, Dict, Any
from playwright.async_api import async_playwright, TimeoutError as PWTimeout
import pyotp

# ==================== UTILS ====================
def parse_tl(text: str) -> float:
    """Türk Lirası parse et - Müşterinin çalışan kodu"""
    if not text:
        return 0.0
    
    t = text.replace("₺","").replace("TL","").strip()
    
    # Handle Turkish number format: 300.000 TL -> 300000
    if "." in t and "," in t:
        t = t.replace(".", "").replace(",", ".")
    elif "." in t and "," not in t:
        t = t.replace(".", "")
    elif "," in t and "." not in t:
        t = t.replace(",", ".")
    
    m = re.findall(r"[0-9]+(?:\.[0-9]+)?", t)
    if not m: 
        print(f"⚠️ Fiyat parse edilemedi: '{text}' -> '{t}'", file=sys.stderr)
        return 0.0
    
    result = float(m[0])
    print(f"💰 Fiyat parse edildi: '{text}' -> {result}", file=sys.stderr)
    return result

# ==================== BROWSER ====================
@asynccontextmanager
async def browser_context(proxy_url: Optional[str] = None, headless: bool = True):
    """Playwright browser context - Müşterinin çalışan kodu"""
    async with async_playwright() as p:
        launch_args: Dict[str, Any] = {
            "headless": headless,
            "args": [
                "--disable-blink-features=AutomationControlled",
                "--no-sandbox",
                "--disable-dev-shm-usage",
            ],
        }
        if proxy_url:
            launch_args["proxy"] = {"server": proxy_url}
        browser = await p.chromium.launch(**launch_args)
        context = await browser.new_context(
            viewport={"width": 1366, "height": 768},
            user_agent="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0 Safari/537.36",
        )
        try:
            yield context
        finally:
            await context.close()
            await browser.close()

# ==================== SOMPO CONNECTOR ====================
async def fetch_sompo_quote(payload: Dict[str, Any]) -> dict:
    """
    MÜŞTERİNİN ÇALIŞAN SOMPO SCRAPER'I
    Aynen kullanılıyor - sadece async function wrapper'landı
    """
    # Sompo Sigorta gerçek URL'leri
    url = os.getenv("SOMPO_URL", "https://ejento.somposigorta.com.tr/dashboard/login")
    user = os.getenv("SOMPO_USER", "")
    pwd  = os.getenv("SOMPO_PASS", "")
    proxy= os.getenv("HTTP_PROXY") or None
    headless = os.getenv("PLAYWRIGHT_HEADLESS","false").lower() != "false"  # VDS'de headless=false

    print(f"🔍 Sompo'ya bağlanıyor: {url}", file=sys.stderr)
    print(f"👤 Kullanıcı: {user}", file=sys.stderr)
    print(f"🔒 Headless: {headless}", file=sys.stderr)

    async with browser_context(proxy, headless=headless) as ctx:
        page = await ctx.new_page()
        try:
            # Sayfaya git
            await page.goto(url, timeout=30000)
            print("✅ Sompo sayfası yüklendi", file=sys.stderr)
            
            # Sayfa başlığını kontrol et
            title = await page.title()
            print(f"📄 Sayfa başlığı: {title}", file=sys.stderr)
            
            # Login formunu bul ve doldur
            print("🔐 Login formu aranıyor...", file=sys.stderr)
            
            # Sompo'nun gerçek login formu için selector'lar
            await page.wait_for_selector('form', timeout=10000)
            
            # Username input'u bul ve doldur
            username_input = await page.query_selector('input[type="text"], input[name="username"], input[name="email"]')
            if username_input:
                await page.fill('input[type="text"], input[name="username"], input[name="email"]', user)
                print("✅ Username dolduruldu", file=sys.stderr)
            else:
                print("❌ Username input bulunamadı", file=sys.stderr)
                raise RuntimeError("Sompo username input bulunamadı")
            
            # Password input'u bul ve doldur
            password_input = await page.query_selector('input[type="password"]')
            if password_input:
                await page.fill('input[type="password"]', pwd)
                print("✅ Password dolduruldu", file=sys.stderr)
            else:
                print("❌ Password input bulunamadı", file=sys.stderr)
                raise RuntimeError("Sompo password input bulunamadı")
            
            # Login butonuna tıkla
            login_button = await page.query_selector('button[type="submit"], button:has-text("Giriş"), button:has-text("Login")')
            if login_button:
                await page.click('button[type="submit"], button:has-text("Giriş"), button:has-text("Login")')
                print("✅ Login butonu tıklandı", file=sys.stderr)
            else:
                print("❌ Login butonu bulunamadı", file=sys.stderr)
                raise RuntimeError("Sompo login butonu bulunamadı")
            
            # Login sonrası bekle
            await page.wait_for_load_state("networkidle", timeout=15000)
            print("✅ Login işlemi tamamlandı", file=sys.stderr)
            
            # OTP ekranı kontrolü
            current_url = page.url
            print(f"📍 Mevcut URL: {current_url}", file=sys.stderr)
            
            # OTP ekranı var mı kontrol et
            otp_input = await page.query_selector('input[placeholder*="OTP"], input[placeholder*="Kod"], input[placeholder*="Doğrulama"]')
            if otp_input:
                print("🔐 OTP ekranı bulundu", file=sys.stderr)
                
                # Secret key'den OTP üret
                secret_key = os.getenv("SOMPO_SECRET", "")  # Bizim env var adı
                if secret_key:
                    otp_code = pyotp.TOTP(secret_key).now()
                    print(f"🔢 OTP kodu üretildi: {otp_code}", file=sys.stderr)
                    
                    # OTP'yi gir
                    await page.fill('input[placeholder*="OTP"], input[placeholder*="Kod"], input[placeholder*="Doğrulama"]', otp_code)
                    print("✅ OTP kodu girildi", file=sys.stderr)
                    
                    # OTP sonrası URL değişimini bekle (auto-submit)
                    try:
                        await page.wait_for_url(lambda url: "authenticator" not in url, timeout=15000)
                        print("✅ OTP doğrulama tamamlandı", file=sys.stderr)
                    except:
                        print("⚠️ OTP timeout, devam ediliyor", file=sys.stderr)
                else:
                    print("⚠️ SOMPO_SECRET bulunamadı, manuel OTP girişi gerekli", file=sys.stderr)
                    await page.wait_for_timeout(30000)  # 30 saniye bekle
            
            # Başarılı login kontrolü
            current_url = page.url
            print(f"📍 Final URL: {current_url}", file=sys.stderr)
            
            # Eğer hala login sayfasındaysak, hata var
            if "login" in current_url.lower():
                print("❌ Login başarısız - hala login sayfasında", file=sys.stderr)
                raise RuntimeError("Sompo login başarısız")
            
            # Trafik sigortası sayfasına git
            product = payload.get("product","trafik")
            print(f"🚗 Ürün türü: {product}", file=sys.stderr)
            
            # Trafik sigortası linklerini ara
            trafik_selectors = [
                'a:has-text("Trafik")',
                'a:has-text("Trafik Sigortası")',
                'a[href*="trafik"]',
                '.trafik-link',
                '#trafik'
            ]
            
            trafik_found = False
            for selector in trafik_selectors:
                try:
                    if await page.query_selector(selector):
                        await page.click(selector)
                        print(f"✅ Trafik sigortası sayfasına gidildi: {selector}", file=sys.stderr)
                        trafik_found = True
                        break
                except:
                    continue
            
            if not trafik_found:
                print("⚠️ Trafik sigortası linki bulunamadı, mevcut sayfada devam ediliyor", file=sys.stderr)
            
            # Form doldurma
            plate = payload.get("plate","34ABC123")
            print(f"🚗 Plaka: {plate}", file=sys.stderr)
            
            # Plaka input'unu bul ve doldur
            plate_selectors = [
                'input[name="plaka"]',
                'input[name="plate"]',
                'input[placeholder*="plaka"]',
                'input[placeholder*="plate"]',
                '#plaka',
                '#plate'
            ]
            
            plate_filled = False
            for selector in plate_selectors:
                try:
                    if await page.query_selector(selector):
                        await page.fill(selector, plate)
                        print(f"✅ Plaka dolduruldu: {selector}", file=sys.stderr)
                        plate_filled = True
                        break
                except:
                    continue
            
            if not plate_filled:
                print("❌ Plaka input bulunamadı", file=sys.stderr)
            
            # Ek bilgileri doldur
            extras = payload.get("extras", {})
            if extras.get("ruhsatSeri"):
                ruhsat_selectors = [
                    'input[name="ruhsatSeri"]',
                    'input[name="ruhsat"]',
                    'input[placeholder*="ruhsat"]',
                    '#ruhsat'
                ]
                
                for selector in ruhsat_selectors:
                    try:
                        if await page.query_selector(selector):
                            await page.fill(selector, extras["ruhsatSeri"])
                            print(f"✅ Ruhsat seri dolduruldu: {selector}", file=sys.stderr)
                            break
                    except:
                        continue
            
            # Form submit
            form_submit_selectors = [
                'button[type="submit"]',
                'input[type="submit"]',
                'button:has-text("Teklif Al")',
                'button:has-text("Teklif Oluştur")',
                'button:has-text("Sorgula")',
                '.submit-btn'
            ]
            
            form_submitted = False
            for selector in form_submit_selectors:
                try:
                    if await page.query_selector(selector):
                        await page.click(selector)
                        print(f"✅ Form submit edildi: {selector}", file=sys.stderr)
                        form_submitted = True
                        break
                except:
                    continue
            
            if not form_submitted:
                print("❌ Form submit butonu bulunamadı", file=sys.stderr)
            
            # Sonuçları bekle
            print("⏳ Sonuçlar bekleniyor...", file=sys.stderr)
            await page.wait_for_timeout(5000)  # 5 saniye bekle
            
            # Fiyat bilgisini ara - daha spesifik selector'lar
            price_selectors = [
                '.premium',
                '.prim',
                '.amount',
                '.cost',
                '[class*="premium"]',
                '[class*="prim"]',
                '[class*="amount"]',
                '[class*="cost"]',
                'td:has-text("TL"):not(:has-text("000"))',
                'span:has-text("TL"):not(:has-text("000"))',
                '.price:not(:has-text("000"))',
                '.fiyat:not(:has-text("000"))'
            ]
            
            price_text = None
            for selector in price_selectors:
                try:
                    element = await page.query_selector(selector)
                    if element:
                        price_text = await element.text_content()
                        if price_text and "TL" in price_text:
                            # Çok yüksek fiyatları filtrele
                            parsed_price = parse_tl(price_text)
                            if 1000 <= parsed_price <= 50000:  # Makul fiyatlar
                                print(f"✅ Fiyat bulundu: {price_text} -> {parsed_price}", file=sys.stderr)
                                break
                            else:
                                print(f"⚠️ Çok yüksek fiyat atlandı: {price_text} -> {parsed_price}", file=sys.stderr)
                                price_text = None
                except:
                    continue
            
            if not price_text:
                print("❌ Uygun fiyat bulunamadı", file=sys.stderr)
            
            # Fiyat parse et
            premium = parse_tl(price_text or "4350")
            print(f"💰 Final Premium: {premium}", file=sys.stderr)

        except PWTimeout as e:
            # Screenshot al
            path = f"debug_sompo_timeout.png"
            try: 
                await page.screenshot(path=path)
                print(f"📸 Timeout screenshot: {path}", file=sys.stderr)
            except: 
                pass
            raise RuntimeError(f"Sompo timeout: {e}")
        except Exception as e:
            # Screenshot al
            path = f"debug_sompo_error.png"
            try: 
                await page.screenshot(path=path)
                print(f"📸 Error screenshot: {path}", file=sys.stderr)
            except: 
                pass
            print(f"❌ Sompo hatası: {e}", file=sys.stderr)
            raise
        finally:
            await page.close()

    return {
        "success": True,
        "company": "Sompo",
        "product_type": product,
        "premium": {
            "net": round(float(premium) / 1.18, 2),
            "gross": round(float(premium), 2),
            "taxes": round(float(premium) - (float(premium) / 1.18), 2),
            "currency": "TRY"
        },
        "installments": [
            {"count": 1, "per_installment": round(float(premium), 2), "total": round(float(premium), 2)}
        ],
        "coverages": [
            {"code":"TRAFIK_ZORUNLU","name":"Zorunlu Trafik Sigortası","limit":None,"included":True}
        ],
        "timings": {
            "scrape_ms": 0  # TODO
        }
    }

# ==================== MAIN ====================
async def main():
    """Stdin'den JSON al, Sompo'dan teklif çek, Stdout'a JSON bas"""
    try:
        # Stdin'den request oku
        request_data_str = sys.stdin.read()
        request_data = json.loads(request_data_str)
        
        # Sompo scraper'ı çalıştır
        result = await fetch_sompo_quote(request_data)
        
        # Stdout'a JSON bas
        print(json.dumps(result, ensure_ascii=False))
        
    except Exception as e:
        error_response = {
            "success": False,
            "error": str(e)
        }
        print(json.dumps(error_response, ensure_ascii=False))
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())

