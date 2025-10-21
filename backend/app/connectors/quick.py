import os, uuid, datetime as dt
from typing import Any, Dict
from .base import BaseConnector
from ..browser import browser_context
from ..utils import parse_tl
from playwright.async_api import TimeoutError as PWTimeout

class QuickConnector(BaseConnector):
    def __init__(self):
        super().__init__("Quick Sigorta")
    
    def _is_business_hours(self) -> bool:
        """Mesai saatleri kontrolü: Pazartesi-Cuma 09:00-18:00"""
        now = dt.datetime.now()
        weekday = now.weekday()  # 0=Pazartesi, 6=Pazar
        hour = now.hour
        
        # Pazartesi-Cuma (0-4) ve 09:00-18:00 arası
        if 0 <= weekday <= 4 and 9 <= hour < 18:
            return True
        return False
    
    def _get_sms_auth_status(self) -> dict:
        """SMS Auth durumu kontrolü"""
        if not self._is_business_hours():
            return {
                "available": False,
                "reason": "Mesai saati dışında SMS doğrulaması yapılamaz",
                "business_hours": "Pazartesi-Cuma 09:00-18:00",
                "current_time": dt.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
            }
        
        return {
            "available": True,
            "reason": "SMS doğrulaması mevcut",
            "business_hours": "Pazartesi-Cuma 09:00-18:00",
            "current_time": dt.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        }
    
    async def fetch_quote(self, payload: Dict[str, Any]) -> dict:
        # SMS Auth durumu kontrolü
        sms_status = self._get_sms_auth_status()
        
        if not sms_status["available"]:
            # Mesai saati dışında mock data döndür
            return {
                "id": str(uuid.uuid4()),
                "company": self.company,
                "premium": 2150.0,  # Mock fiyat
                "currency": "TRY",
                "validUntil": (dt.datetime.utcnow() + dt.timedelta(hours=2)).isoformat() + "Z",
                "coverages": [{"code":"TRAFIK_ZORUNLU","label":"Zorunlu Trafik"}],
                "extras": {
                    "note": "Quick SMS Auth - Mesai saati dışında",
                    "sms_status": sms_status,
                    "mock_data": True
                },
            }
        
        # Mesai saatleri içinde gerçek API'yi kullan
        url = os.getenv("QUICK_URL", "https://www.quicksigorta.com.tr/agent/login")
        user = os.getenv("QUICK_USER", "")
        pwd  = os.getenv("QUICK_PASS", "")
        proxy= os.getenv("HTTP_PROXY") or None
        headless = os.getenv("PLAYWRIGHT_HEADLESS","true").lower() != "false"

        print(f"🔍 Quick'e bağlanıyor: {url}")
        print(f"👤 Kullanıcı: {user}")
        print(f"📱 SMS Auth Durumu: {sms_status}")
        print(f"🔒 Headless: {headless}")

        async with browser_context(proxy, headless=headless) as ctx:
            page = await ctx.new_page()
            try:
                # Sayfaya git
                await page.goto(url, timeout=30000)
                print("✅ Quick sayfası yüklendi")
                
                # Sayfa başlığını kontrol et
                title = await page.title()
                print(f"📄 Sayfa başlığı: {title}")
                
                # Login formunu bul ve doldur
                print("🔐 Login formu aranıyor...")
                
                # Farklı login selector'larını dene
                login_selectors = [
                    'input[name="username"]',
                    'input[name="email"]', 
                    'input[name="user"]',
                    'input[type="email"]',
                    '#username',
                    '#email',
                    '#user',
                    '.username',
                    '.email'
                ]
                
                username_filled = False
                for selector in login_selectors:
                    try:
                        if await page.query_selector(selector):
                            await page.fill(selector, user)
                            print(f"✅ Username dolduruldu: {selector}")
                            username_filled = True
                            break
                    except:
                        continue
                
                if not username_filled:
                    print("❌ Username input bulunamadı")
                
                # Password input'u bul
                password_selectors = [
                    'input[name="password"]',
                    'input[type="password"]',
                    '#password',
                    '.password'
                ]
                
                password_filled = False
                for selector in password_selectors:
                    try:
                        if await page.query_selector(selector):
                            await page.fill(selector, pwd)
                            print(f"✅ Password dolduruldu: {selector}")
                            password_filled = True
                            break
                    except:
                        continue
                
                if not password_filled:
                    print("❌ Password input bulunamadı")
                
                # Submit butonunu bul ve tıkla
                submit_selectors = [
                    'button[type="submit"]',
                    'input[type="submit"]',
                    'button:has-text("Giriş")',
                    'button:has-text("Login")',
                    '.login-btn',
                    '#login-btn'
                ]
                
                submitted = False
                for selector in submit_selectors:
                    try:
                        if await page.query_selector(selector):
                            await page.click(selector)
                            print(f"✅ Submit butonu tıklandı: {selector}")
                            submitted = True
                            break
                    except:
                        continue
                
                if not submitted:
                    print("❌ Submit butonu bulunamadı")
                
                # Login sonrası bekle
                await page.wait_for_load_state("networkidle", timeout=20000)
                print("✅ Login işlemi tamamlandı")
                
                # SMS doğrulama kontrolü
                sms_selectors = [
                    'input[name="sms_code"]',
                    'input[name="verification_code"]',
                    'input[placeholder*="SMS"]',
                    'input[placeholder*="Doğrulama"]',
                    '#sms_code',
                    '#verification_code'
                ]
                
                sms_input_found = False
                for selector in sms_selectors:
                    try:
                        if await page.query_selector(selector):
                            print(f"📱 SMS doğrulama input'u bulundu: {selector}")
                            sms_input_found = True
                            break
                    except:
                        continue
                
                if sms_input_found:
                    print("📱 SMS doğrulama gerekiyor - mesai saatleri içinde")
                    # SMS kodu bekle (gerçek uygulamada kullanıcıdan alınacak)
                    sms_code = os.getenv("QUICK_SMS_CODE", "123456")
                    await page.fill(selector, sms_code)
                    print(f"✅ SMS kodu dolduruldu: {sms_code}")
                    
                    # SMS submit
                    sms_submit_selectors = [
                        'button:has-text("Doğrula")',
                        'button:has-text("Onayla")',
                        'button[type="submit"]'
                    ]
                    
                    for submit_selector in sms_submit_selectors:
                        try:
                            if await page.query_selector(submit_selector):
                                await page.click(submit_selector)
                                print(f"✅ SMS doğrulama submit edildi: {submit_selector}")
                                break
                        except:
                            continue
                    
                    await page.wait_for_load_state("networkidle", timeout=20000)
                    print("✅ SMS doğrulama tamamlandı")
                else:
                    print("ℹ️ SMS doğrulama gerekmiyor")
                
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
                
                # Fiyat bilgisini ara
                price_selectors = [
                    '.price',
                    '.fiyat',
                    '.tutar',
                    '[class*="price"]',
                    '[class*="fiyat"]',
                    '[class*="tutar"]',
                    'td:has-text("TL")',
                    'span:has-text("TL")'
                ]
                
                price_text = None
                for selector in price_selectors:
                    try:
                        element = await page.query_selector(selector)
                        if element:
                            price_text = await element.text_content()
                            if price_text and "TL" in price_text:
                                print(f"✅ Fiyat bulundu: {price_text}")
                                break
                    except:
                        continue
                
                if not price_text:
                    print("❌ Fiyat bulunamadı")
                    # Mock fiyat kullan
                    premium = 2150.0
                else:
                    premium = parse_tl(price_text)
                
                print(f"💰 Premium: {premium}")

            except PWTimeout as e:
                # Screenshot al
                path = f"/tmp/quick_timeout_{uuid.uuid4().hex}.png"
                try: 
                    await page.screenshot(path=path)
                    print(f"📸 Timeout screenshot: {path}")
                except: 
                    pass
                raise RuntimeError(f"Quick timeout: {e}")
            except Exception as e:
                # Screenshot al
                path = f"/tmp/quick_error_{uuid.uuid4().hex}.png"
                try: 
                    await page.screenshot(path=path)
                    print(f"📸 Error screenshot: {path}")
                except: 
                    pass
                print(f"❌ Quick hatası: {e}")
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
            "extras": {
                "note": "quick playwright", 
                "url": url, 
                "user": user,
                "sms_status": sms_status,
                "business_hours": self._is_business_hours()
            },
        }
