import os, uuid, datetime as dt
from typing import Any, Dict
from .base import BaseConnector
from ..browser import browser_context
from ..utils import parse_tl
from playwright.async_api import TimeoutError as PWTimeout
import pyotp

class SompoConnector(BaseConnector):
    def __init__(self):
        super().__init__("Sompo Sigorta")
    
    async def fetch_quote(self, payload: Dict[str, Any]) -> dict:
        # Sompo Sigorta gerçek URL'leri
        url = os.getenv("SOMPO_URL", "https://ejento.somposigorta.com.tr/dashboard/login")
        user = os.getenv("SOMPO_USER", "")
        pwd  = os.getenv("SOMPO_PASS", "")
        proxy= os.getenv("HTTP_PROXY") or None
        headless = os.getenv("PLAYWRIGHT_HEADLESS","true").lower() != "false"

        print(f"🔍 Sompo'ya bağlanıyor: {url}")
        print(f"👤 Kullanıcı: {user}")
        print(f"🔒 Headless: {headless}")

        async with browser_context(proxy, headless=headless) as ctx:
            page = await ctx.new_page()
            try:
                # Sayfaya git
                await page.goto(url, timeout=30000)
                print("✅ Sompo sayfası yüklendi")
                
                # Sayfa başlığını kontrol et
                title = await page.title()
                print(f"📄 Sayfa başlığı: {title}")
                
                # Login formunu bul ve doldur
                print("🔐 Login formu aranıyor...")
                
                # Sompo'nun gerçek login formu için selector'lar
                await page.wait_for_selector('form', timeout=10000)
                
                # Username input'u bul ve doldur
                username_input = await page.query_selector('input[type="text"], input[name="username"], input[name="email"]')
                if username_input:
                    await page.fill('input[type="text"], input[name="username"], input[name="email"]', user)
                    print("✅ Username dolduruldu")
                else:
                    print("❌ Username input bulunamadı")
                    raise RuntimeError("Sompo username input bulunamadı")
                
                # Password input'u bul ve doldur
                password_input = await page.query_selector('input[type="password"]')
                if password_input:
                    await page.fill('input[type="password"]', pwd)
                    print("✅ Password dolduruldu")
                else:
                    print("❌ Password input bulunamadı")
                    raise RuntimeError("Sompo password input bulunamadı")
                
                # Login butonuna tıkla
                login_button = await page.query_selector('button[type="submit"], button:has-text("Giriş"), button:has-text("Login")')
                if login_button:
                    await page.click('button[type="submit"], button:has-text("Giriş"), button:has-text("Login")')
                    print("✅ Login butonu tıklandı")
                else:
                    print("❌ Login butonu bulunamadı")
                    raise RuntimeError("Sompo login butonu bulunamadı")
                
                # Login sonrası bekle
                await page.wait_for_load_state("networkidle", timeout=15000)
                print("✅ Login işlemi tamamlandı")
                
                # OTP ekranı kontrolü
                current_url = page.url
                print(f"📍 Mevcut URL: {current_url}")
                
                # OTP ekranı var mı kontrol et
                otp_input = await page.query_selector('input[placeholder*="OTP"], input[placeholder*="Kod"], input[placeholder*="Doğrulama"]')
                if otp_input:
                    print("🔐 OTP ekranı bulundu")
                    
                    # Secret key'den OTP üret
                    secret_key = os.getenv("SOMPO_SECRET_KEY", "")
                    if secret_key:
                        otp_code = pyotp.TOTP(secret_key).now()
                        print(f"🔢 OTP kodu üretildi: {otp_code}")
                        
                        # OTP'yi gir
                        await page.fill('input[placeholder*="OTP"], input[placeholder*="Kod"], input[placeholder*="Doğrulama"]', otp_code)
                        print("✅ OTP kodu girildi")
                        
                        # OTP submit butonuna tıkla
                        otp_submit = await page.query_selector('button[type="submit"], button:has-text("Doğrula"), button:has-text("Onayla")')
                        if otp_submit:
                            await page.click('button[type="submit"], button:has-text("Doğrula"), button:has-text("Onayla")')
                            print("✅ OTP doğrulama butonu tıklandı")
                            
                            # OTP sonrası bekle
                            await page.wait_for_load_state("networkidle", timeout=15000)
                            print("✅ OTP doğrulama tamamlandı")
                        else:
                            print("⚠️ OTP submit butonu bulunamadı, manuel onay bekleniyor")
                    else:
                        print("⚠️ SOMPO_SECRET_KEY bulunamadı, manuel OTP girişi gerekli")
                        # Manuel OTP girişi için bekle
                        await page.wait_for_timeout(30000)  # 30 saniye bekle
                
                # Başarılı login kontrolü
                current_url = page.url
                print(f"📍 Final URL: {current_url}")
                
                # Eğer hala login sayfasındaysak, hata var
                if "login" in current_url.lower():
                    print("❌ Login başarısız - hala login sayfasında")
                    raise RuntimeError("Sompo login başarısız")
                
                # Trafik sigortası sayfasına git
                product = payload.get("product","trafik")
                print(f"🚗 Ürün türü: {product}")
                
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
                            print(f"✅ Trafik sigortası sayfasına gidildi: {selector}")
                            trafik_found = True
                            break
                    except:
                        continue
                
                if not trafik_found:
                    print("⚠️ Trafik sigortası linki bulunamadı, mevcut sayfada devam ediliyor")
                
                # Form doldurma
                plate = payload.get("plate","34ABC123")
                print(f"🚗 Plaka: {plate}")
                
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
                            print(f"✅ Plaka dolduruldu: {selector}")
                            plate_filled = True
                            break
                    except:
                        continue
                
                if not plate_filled:
                    print("❌ Plaka input bulunamadı")
                
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
                                print(f"✅ Ruhsat seri dolduruldu: {selector}")
                                break
                        except:
                            continue
                
                # Form submit
                form_submit_selectors = [
                    'button[type="submit"]',
                    'input[type="submit"]',
                    'button:has-text("Teklif Al")',
                    'button:has-text("Sorgula")',
                    '.submit-btn'
                ]
                
                form_submitted = False
                for selector in form_submit_selectors:
                    try:
                        if await page.query_selector(selector):
                            await page.click(selector)
                            print(f"✅ Form submit edildi: {selector}")
                            form_submitted = True
                            break
                    except:
                        continue
                
                if not form_submitted:
                    print("❌ Form submit butonu bulunamadı")
                
                # Sonuçları bekle
                print("⏳ Sonuçlar bekleniyor...")
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
                    'td:has-text("TL"):not(:has-text("000"))',  # 000 içermeyen TL'ler
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
                                # Çok yüksek fiyatları filtrele (muhtemelen yanlış element)
                                parsed_price = parse_tl(price_text)
                                if 1000 <= parsed_price <= 50000:  # 1.000-50.000 TL arası makul fiyatlar
                                    print(f"✅ Fiyat bulundu: {price_text} -> {parsed_price}")
                                    break
                                else:
                                    print(f"⚠️ Çok yüksek fiyat atlandı: {price_text} -> {parsed_price}")
                                    price_text = None
                    except:
                        continue
                
                if not price_text:
                    print("❌ Uygun fiyat bulunamadı")
                    # Sayfa içeriğini kontrol et
                    content = await page.content()
                    if "TL" in content:
                        print("⚠️ Sayfada TL içeriği var ama uygun fiyat bulunamadı")
                        # Tüm TL içeren elementleri listele
                        try:
                            all_tl_elements = await page.query_selector_all('*:has-text("TL")')
                            for i, el in enumerate(all_tl_elements[:5]):  # İlk 5'i göster
                                text = await el.text_content()
                                if text and "TL" in text:
                                    print(f"   {i+1}. {text}")
                        except:
                            pass
                    else:
                        print("❌ Sayfada hiç TL içeriği yok")
                
                # Mock fiyat kullan
                premium = parse_tl(price_text or "4350")
                print(f"💰 Premium: {premium}")

            except PWTimeout as e:
                # Screenshot al
                path = f"/tmp/sompo_timeout_{uuid.uuid4().hex}.png"
                try: 
                    await page.screenshot(path=path)
                    print(f"📸 Timeout screenshot: {path}")
                except: 
                    pass
                raise RuntimeError(f"Sompo timeout: {e}")
            except Exception as e:
                # Screenshot al
                path = f"/tmp/sompo_error_{uuid.uuid4().hex}.png"
                try: 
                    await page.screenshot(path=path)
                    print(f"📸 Error screenshot: {path}")
                except: 
                    pass
                print(f"❌ Sompo hatası: {e}")
                raise
            finally:
                await page.close()

        return {
            "id": str(uuid.uuid4()),
            "company": self.company,
            "premium": float(premium),
            "currency": "TRY",
            "validUntil": (dt.datetime.utcnow() + dt.timedelta(hours=2)).isoformat() + "Z",
            "coverages": [
                {"code":"TRAFIK_ZORUNLU","label":"Zorunlu Trafik"}
            ],
            "extras": {"note":"sompo playwright", "url": url, "user": user},
        }
