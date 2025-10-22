#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Sompo Session Generator - Python subprocess için
Login yapıp session'ı JSON olarak döndürür
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

def main():
    # Environment variables
    username = os.getenv("SOMPO_USER", "")
    password = os.getenv("SOMPO_PASS", "")
    secret_key = os.getenv("SOMPO_SECRET", "")
    
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
        driver.get("https://ejento.somposigorta.com.tr/dashboard/login")
        
        # Wait for form
        WebDriverWait(driver, 15).until(
            EC.presence_of_element_located((By.TAG_NAME, "form"))
        )
        
        print(f"[INFO] Login sayfası yüklendi", file=sys.stderr)
        
        # Username (XPath - zaten çalışıyor)
        username_input = driver.find_element(
            By.XPATH, 
            '/html/body/div[1]/div/div[1]/div[2]/form/div[1]/div/input'
        )
        username_input.send_keys(username)
        
        # Password (XPath - zaten çalışıyor)
        password_input = driver.find_element(
            By.XPATH,
            '/html/body/div[1]/div/div[1]/div[2]/form/div[2]/div/div/input'
        )
        password_input.send_keys(password)
        
        print(f"[INFO] Credentials girildi", file=sys.stderr)
        
        # Login button
        login_btn = driver.find_element(By.XPATH, '//button[@type="submit"]')
        login_btn.click()
        
        print(f"[INFO] Login button tıklandı", file=sys.stderr)
        
        # Screenshot al (debug)
        driver.save_screenshot("debug_after_login.png")
        print(f"[DEBUG] Screenshot kaydedildi: debug_after_login.png", file=sys.stderr)
        
        # Wait for URL change (login sayfasından çık)
        time.sleep(3)
        
        # URL bekle
        try:
            WebDriverWait(driver, 10).until(
                lambda d: d.current_url != "https://ejento.somposigorta.com.tr/dashboard/login"
            )
        except:
            print(f"[WARNING] URL değişmedi - login başarısız olabilir", file=sys.stderr)
        
        current_url = driver.current_url
        print(f"[INFO] Current URL after login: {current_url}", file=sys.stderr)
        
        # OTP ekranı var mı?
        if "authenticator" in current_url or "otp" in current_url.lower() or "google-authenticator" in current_url:
            print(f"[INFO] OTP ekranı tespit edildi", file=sys.stderr)
            
            if not secret_key:
                print(json.dumps({"error": "SOMPO_SECRET gerekli (OTP için)"}), file=sys.stderr)
                sys.exit(1)
            
            # TOTP üret
            otp = pyotp.TOTP(secret_key).now()
            print(f"[INFO] OTP üretildi: {otp}", file=sys.stderr)
            
            # OTP input bul - birden fazla selector dene
            otp_input = None
            selectors = [
                (By.CSS_SELECTOR, 'input[placeholder*="OTP"]'),
                (By.CSS_SELECTOR, 'input[placeholder*="Kod"]'),
                (By.CSS_SELECTOR, 'input[placeholder*="kod"]'),
                (By.CSS_SELECTOR, 'input[name*="otp"]'),
                (By.XPATH, '//input[@type="text"]'),
            ]
            
            for by, selector in selectors:
                try:
                    otp_input = driver.find_element(by, selector)
                    if otp_input:
                        print(f"[INFO] OTP input bulundu: {selector}", file=sys.stderr)
                        break
                except:
                    continue
            
            if otp_input:
                otp_input.clear()
                otp_input.send_keys(otp)
                print(f"[INFO] OTP girildi", file=sys.stderr)
                
                # Screenshot al (OTP ekranı)
                driver.save_screenshot("debug_otp_entered.png")
                print(f"[DEBUG] OTP screenshot: debug_otp_entered.png", file=sys.stderr)
            else:
                print(f"[ERROR] OTP input bulunamadı!", file=sys.stderr)
                driver.save_screenshot("debug_otp_not_found.png")
                print(f"[DEBUG] Error screenshot: debug_otp_not_found.png", file=sys.stderr)
                sys.exit(1)
            
            # URL değişimini bekle (auto-submit)
            WebDriverWait(driver, 20).until(
                lambda d: "authenticator" not in d.current_url and "otp" not in d.current_url.lower()
            )
            
            print(f"[INFO] OTP başarılı!", file=sys.stderr)
        
        # Dashboard'a ulaştığımızı doğrula (login sayfası DEĞİL!)
        try:
            WebDriverWait(driver, 10).until(
                lambda d: "dashboard" in d.current_url and "login" not in d.current_url
            )
            print(f"[INFO] Dashboard'a ulaşıldı: {driver.current_url}", file=sys.stderr)
        except:
            # Başarısız olursa debug bilgisi
            final_url = driver.current_url
            driver.save_screenshot("debug_login_failed.png")
            print(f"[ERROR] Dashboard'a ulaşılamadı! Final URL: {final_url}", file=sys.stderr)
            print(f"[DEBUG] Error screenshot: debug_login_failed.png", file=sys.stderr)
            
            # Bot detection kontrolü
            if "/bot" in final_url:
                print(json.dumps({"error": "Bot detection - CAPTCHA gerekli"}), file=sys.stderr)
            elif "login" in final_url:
                print(json.dumps({"error": "Login başarısız - credentials yanlış olabilir"}), file=sys.stderr)
            else:
                print(json.dumps({"error": f"Beklenmeyen sayfa: {final_url}"}), file=sys.stderr)
            sys.exit(1)
        
        # Session dump
        cookies = driver.get_cookies()
        
        # localStorage
        local_storage = driver.execute_script("""
            var storage = {};
            for (var i = 0; i < localStorage.length; i++) {
                var key = localStorage.key(i);
                storage[key] = localStorage.getItem(key);
            }
            return storage;
        """)
        
        # Validation: Gerçek session'da localStorage veya cookie olmalı
        if len(cookies) < 3 and len(local_storage) == 0:
            print(f"[WARNING] Şüpheli session: Çok az cookie ({len(cookies)}) ve localStorage boş!", file=sys.stderr)
            driver.save_screenshot("debug_suspicious_session.png")
            print(f"[DEBUG] Suspicious session screenshot: debug_suspicious_session.png", file=sys.stderr)
        
        session_data = {
            "cookies": cookies,
            "local_storage": local_storage,
            "timestamp": int(time.time()),
            "url": driver.current_url
        }
        
        # JSON output (stdout)
        print(json.dumps(session_data, ensure_ascii=False))
        
        print(f"[INFO] Session kaydedildi - {len(cookies)} cookies, {len(local_storage)} localStorage items", file=sys.stderr)
        
    except Exception as e:
        print(json.dumps({"error": str(e)}), file=sys.stderr)
        sys.exit(1)
    finally:
        if driver:
            driver.quit()
            print(f"[INFO] Browser kapatıldı", file=sys.stderr)

if __name__ == "__main__":
    main()

