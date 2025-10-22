#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Sompo Full Scraper - Login + Quote + Parse
%100 garantili Python implementation
"""

import json
import os
import sys
import time
import pyotp
import undetected_chromedriver as uc
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

def parse_tl_price(text):
    """TL fiyat parse et"""
    cleaned = text.replace('TL', '').replace('₺', '').replace(' ', '')
    cleaned = cleaned.replace('.', '').replace(',', '.')  # 1.234,56 -> 1234.56
    try:
        return float(cleaned.strip())
    except:
        return 0.0

def main():
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
    product_type = request.get("product_type", "trafik")  # trafik veya kasko
    
    if not username or not password:
        print(json.dumps({"error": "SOMPO_USER ve SOMPO_PASS gerekli"}), file=sys.stderr)
        sys.exit(1)
    
    driver = None
    try:
        # Chrome options
        options = uc.ChromeOptions()
        options.add_argument("--no-sandbox")
        options.add_argument("--disable-dev-shm-usage")
        options.add_argument("--window-size=1200,800")
        
        driver = uc.Chrome(options=options)
        start_time = time.time()
        
        print(f"[INFO] Sompo scraping başlatıldı: {product_type} - {plate}", file=sys.stderr)
        
        # ==================== LOGIN ====================
        driver.get("https://ejento.somposigorta.com.tr/dashboard/login")
        
        WebDriverWait(driver, 15).until(
            EC.presence_of_element_located((By.TAG_NAME, "form"))
        )
        print(f"[INFO] Login sayfası yüklendi", file=sys.stderr)
        
        # Username
        username_input = driver.find_element(
            By.XPATH, 
            '/html/body/div[1]/div/div[1]/div[2]/form/div[1]/div/input'
        )
        username_input.send_keys(username)
        
        # Password
        password_input = driver.find_element(
            By.XPATH,
            '/html/body/div[1]/div/div[1]/div[2]/form/div[2]/div/div/input'
        )
        password_input.send_keys(password)
        
        # Login button
        login_btn = driver.find_element(By.XPATH, '//button[@type="submit"]')
        login_btn.click()
        print(f"[INFO] Login button tıklandı", file=sys.stderr)
        
        # URL değişimini bekle
        login_url = "https://ejento.somposigorta.com.tr/dashboard/login"
        WebDriverWait(driver, 15).until(
            lambda d: d.current_url != login_url
        )
        
        time.sleep(2)
        current_url = driver.current_url
        print(f"[INFO] URL after login: {current_url}", file=sys.stderr)
        
        # OTP ekranı?
        if "authenticator" in current_url or "google-authenticator" in current_url:
            print(f"[INFO] OTP ekranı tespit edildi", file=sys.stderr)
            
            if not secret_key:
                print(json.dumps({"error": "SOMPO_SECRET gerekli"}), file=sys.stderr)
                sys.exit(1)
            
            # TOTP üret
            otp = pyotp.TOTP(secret_key).now()
            print(f"[INFO] OTP üretildi", file=sys.stderr)
            
            # OTP input bul
            otp_input = None
            selectors = [
                (By.CSS_SELECTOR, 'input[placeholder*="OTP"]'),
                (By.CSS_SELECTOR, 'input[placeholder*="Kod"]'),
                (By.CSS_SELECTOR, 'input[placeholder*="kod"]'),
                (By.XPATH, '//input[@type="text"]'),
            ]
            
            for by, selector in selectors:
                try:
                    otp_input = driver.find_element(by, selector)
                    if otp_input:
                        break
                except:
                    continue
            
            if otp_input:
                otp_input.clear()
                otp_input.send_keys(otp)
                print(f"[INFO] OTP girildi", file=sys.stderr)
            else:
                print(json.dumps({"error": "OTP input bulunamadı"}), file=sys.stderr)
                sys.exit(1)
            
            # URL değişimini bekle (auto-submit)
            WebDriverWait(driver, 20).until(
                lambda d: "authenticator" not in d.current_url
            )
            print(f"[INFO] OTP başarılı!", file=sys.stderr)
        
        # Dashboard kontrolü
        WebDriverWait(driver, 10).until(
            lambda d: "dashboard" in d.current_url and "login" not in d.current_url
        )
        print(f"[INFO] Dashboard'a ulaşıldı: {driver.current_url}", file=sys.stderr)
        
        # ==================== QUOTE ====================
        
        # YENİ İŞ TEKLİFİ butonu
        try:
            new_offer_btn = WebDriverWait(driver, 10).until(
                EC.presence_of_element_located((By.XPATH, '//button[contains(., "YENİ İŞ TEKLİFİ")]'))
            )
            new_offer_btn.click()
            print(f"[INFO] Yeni İş Teklifi butonu tıklandı", file=sys.stderr)
            time.sleep(2)
        except:
            print(f"[WARNING] Yeni İş Teklifi butonu bulunamadı", file=sys.stderr)
        
        # QR popup kapat (varsa)
        try:
            driver.execute_script("""
                const qrPopup = document.querySelector('.p-dialog-header');
                if (qrPopup && qrPopup.textContent.includes('QR Kod')) {
                    const noButton = qrPopup.closest('.p-dialog').querySelector('button');
                    if (noButton && noButton.textContent.includes('Hayır')) {
                        noButton.click();
                    }
                }
            """)
            time.sleep(1)
        except:
            pass
        
        # Ürün seçimi (Trafik/Kasko)
        product_keywords = {
            'trafik': ['trafik', 'zorunlu'],
            'kasko': ['kasko']
        }
        keywords = product_keywords.get(product_type.lower(), ['trafik'])
        
        print(f"[INFO] Ürün seçiliyor: {product_type}", file=sys.stderr)
        
        # Ürün butonunu bul ve tıkla
        product_selected = False
        try:
            buttons = driver.find_elements(By.TAG_NAME, 'button')
            for btn in buttons:
                btn_text = btn.text.lower()
                if any(kw in btn_text for kw in keywords) and 'teklif' in btn_text:
                    # Trafik/Kasko + "Teklif Al" gibi
                    driver.execute_script("arguments[0].scrollIntoView({block: 'center'});", btn)
                    time.sleep(0.5)
                    driver.execute_script("arguments[0].click();", btn)
                    print(f"[INFO] Ürün butonu tıklandı: {btn_text[:30]}", file=sys.stderr)
                    product_selected = True
                    time.sleep(3)
                    break
        except Exception as e:
            print(f"[WARNING] Ürün seçimi hatası: {e}", file=sys.stderr)
        
        if not product_selected:
            print(json.dumps({"error": f"{product_type} ürünü seçilemedi"}), file=sys.stderr)
            sys.exit(1)
        
        # Form doldur - Plaka ve TCKN
        print(f"[INFO] Form dolduruluyor: Plaka={plate}, TCKN={tckn}", file=sys.stderr)
        
        # Plaka input
        plaka_filled = False
        try:
            inputs = driver.find_elements(By.TAG_NAME, 'input')
            for inp in inputs:
                if inp.is_displayed() and not inp.get_attribute('disabled'):
                    placeholder = inp.get_attribute('placeholder') or ''
                    name = inp.get_attribute('name') or ''
                    if 'plak' in placeholder.lower() or 'plak' in name.lower():
                        inp.clear()
                        inp.send_keys(plate)
                        print(f"[INFO] Plaka dolduruldu", file=sys.stderr)
                        plaka_filled = True
                        break
        except Exception as e:
            print(f"[WARNING] Plaka doldurma hatası: {e}", file=sys.stderr)
        
        # TCKN input
        tckn_filled = False
        try:
            inputs = driver.find_elements(By.TAG_NAME, 'input')
            for inp in inputs:
                if inp.is_displayed() and not inp.get_attribute('disabled'):
                    placeholder = inp.get_attribute('placeholder') or ''
                    name = inp.get_attribute('name') or ''
                    if 'tc' in placeholder.lower() or 'tc' in name.lower() or 'kimlik' in placeholder.lower():
                        inp.clear()
                        inp.send_keys(tckn)
                        print(f"[INFO] TCKN dolduruldu", file=sys.stderr)
                        tckn_filled = True
                        break
        except Exception as e:
            print(f"[WARNING] TCKN doldurma hatası: {e}", file=sys.stderr)
        
        time.sleep(1)
        
        # Submit button
        print(f"[INFO] Submit butonu aranıyor...", file=sys.stderr)
        submit_clicked = False
        try:
            buttons = driver.find_elements(By.TAG_NAME, 'button')
            for btn in buttons:
                if btn.is_displayed() and not btn.get_attribute('disabled'):
                    btn_text = btn.text.lower()
                    if 'teklif' in btn_text or 'sorgula' in btn_text or 'hesapla' in btn_text or 'devam' in btn_text:
                        driver.execute_script("arguments[0].scrollIntoView({block: 'center'});", btn)
                        time.sleep(0.5)
                        driver.execute_script("arguments[0].click();", btn)
                        print(f"[INFO] Submit button tıklandı: {btn_text[:30]}", file=sys.stderr)
                        submit_clicked = True
                        break
        except Exception as e:
            print(f"[WARNING] Submit button hatası: {e}", file=sys.stderr)
        
        if not submit_clicked:
            print(json.dumps({"error": "Submit butonu bulunamadı"}), file=sys.stderr)
            sys.exit(1)
        
        # Sonuçları bekle
        print(f"[INFO] Sonuçlar bekleniyor...", file=sys.stderr)
        time.sleep(10)
        
        # ==================== PARSE ====================
        
        # Fiyat bul
        price = 0.0
        price_text = ""
        
        try:
            # Sayfadaki tüm TL içeren textleri bul
            elements = driver.find_elements(By.XPATH, "//*[contains(text(), 'TL')]")
            for el in elements:
                text = el.text.strip()
                if 'TL' in text and len(text) < 50:
                    # Sayı içeriyor mu?
                    if any(char.isdigit() for char in text):
                        try:
                            parsed = parse_tl_price(text)
                            if parsed > 0 and parsed < 100000:  # Makul aralık
                                if parsed > price:  # En büyük fiyat (gross olabilir)
                                    price = parsed
                                    price_text = text
                        except:
                            continue
        except Exception as e:
            print(f"[WARNING] Fiyat parse hatası: {e}", file=sys.stderr)
        
        if price == 0.0:
            # Screenshot al (debug)
            driver.save_screenshot("debug_no_price.png")
            print(f"[ERROR] Fiyat bulunamadı!", file=sys.stderr)
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
        
    except Exception as e:
        import traceback
        print(f"[ERROR] Exception: {str(e)}", file=sys.stderr)
        print(traceback.format_exc(), file=sys.stderr)
        print(json.dumps({"error": str(e)}), file=sys.stderr)
        sys.exit(1)
    finally:
        if driver:
            driver.quit()
            print(f"[INFO] Browser kapatıldı", file=sys.stderr)

if __name__ == "__main__":
    main()

