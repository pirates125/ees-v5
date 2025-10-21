# -*- coding: utf-8 -*-
"""
Sompo login + OTP (Google Auth) + session dump (cookies + localStorage).
Koy: backend/app/connectors/sompo_login.py
Çalıştır:  python -m app.connectors.sompo_login
"""

import json, time, sys
import pyotp
import undetected_chromedriver as uc
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

# ======= AYARLAR =======
LOGIN_URL = "https://ejento.somposigorta.com.tr/dashboard/login"

USERNAME = "BULUT1"
PASSWORD = "EEsigorta.2828"
SECRET_KEY = "DD3JCJB7E7H25MB6BZ5IKXLKLJBZDQAO"  # Google Auth secret (base32)

COOKIE_FILE_PATH = "sompo_cookies.json"
LOCAL_STORAGE_FILE_PATH = "sompo_local_storage.json"
OTP_DUMP_HTML = "otp_dom_dump.html"
WINDOW_SIZE = "1200,800"
# =======================


def save_cookies(driver):
    cookies = driver.get_cookies()
    with open(COOKIE_FILE_PATH, "w", encoding="utf-8") as f:
        json.dump(cookies, f, ensure_ascii=False, indent=2)
    print(f"[OK] Cookies kaydedildi -> {COOKIE_FILE_PATH}  ({len(cookies)} adet)")


def save_local_storage(driver):
    data = driver.execute_script(
        """
        var items = {};
        for (var i=0;i<localStorage.length;i++){
            items[localStorage.key(i)] = localStorage.getItem(localStorage.key(i));
        }
        return items;
        """
    )
    with open(LOCAL_STORAGE_FILE_PATH, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)
    print(f"[OK] LocalStorage kaydedildi -> {LOCAL_STORAGE_FILE_PATH}  ({len(data)} anahtar)")


def click_login_button(driver):
    # Giriş butonunu birkaç olası seçiciyle dene
    candidates = [
        "//button[contains(., 'Giriş')]",
        "//button[contains(., 'GİRİŞ')]",
        "//button[@type='submit']",
        "//input[@type='submit']",
        "//button[contains(@class,'login')]",
    ]
    for xp in candidates:
        try:
            btn = WebDriverWait(driver, 4).until(EC.element_to_be_clickable((By.XPATH, xp)))
            btn.click()
            print(f"[INFO] Giriş butonuna tıklandı (xpath: {xp})")
            return True
        except:
            pass
    print("[WARN] Giriş butonu otomatik bulunamadı; gerekiyorsa elle tıklayın.")
    return False


def find_otp_input(driver, timeout=15):
    """OTP input’unu normal DOM, iframe ve shadow-root’ta arar."""
    basic_selectors = [
        (By.ID, "otp"),
        (By.NAME, "otp"),
        (By.CSS_SELECTOR, "input[autocomplete='one-time-code']"),
        (By.XPATH, "//input[contains(@placeholder,'OTP') or contains(@placeholder,'Kod') or contains(@placeholder,'Doğrulama')]"),
        (By.XPATH, "//input[@type='tel' or @inputmode='numeric']"),
    ]
    # 1) Normal DOM
    for by, sel in basic_selectors:
        try:
            el = WebDriverWait(driver, 3).until(EC.presence_of_element_located((by, sel)))
            print(f"[OK] OTP input bulundu (normal DOM) -> {by} {sel}")
            return el
        except:
            pass

    # 2) iframe içinde
    iframes = driver.find_elements(By.TAG_NAME, "iframe")
    print(f"[INFO] {len(iframes)} iframe tespit edildi, içerisini tarıyorum...")
    for i, frame in enumerate(iframes):
        try:
            driver.switch_to.frame(frame)
            for by, sel in basic_selectors:
                try:
                    el = WebDriverWait(driver, 2).until(EC.presence_of_element_located((by, sel)))
                    print(f"[OK] OTP input bulundu (iframe #{i}) -> {by} {sel}")
                    return el
                except:
                    pass
        finally:
            driver.switch_to.default_content()

    # 3) Shadow-root taraması (JS)
    try:
        el = driver.execute_script(
            """
            function findOtp(root){
              const sels = [
                "input#otp","input[name=otp]",
                "input[autocomplete='one-time-code']",
                "input[placeholder*='OTP']","input[placeholder*='Kod']","input[placeholder*='Doğrulama']",
                "input[inputmode='numeric']","input[type='tel']"
              ];
              for (const s of sels){
                const f = root.querySelector(s);
                if (f) return f;
              }
              const all = root.querySelectorAll('*');
              for (const n of all){
                if (n.shadowRoot){
                  const r = findOtp(n.shadowRoot);
                  if (r) return r;
                }
              }
              return null;
            }
            return findOtp(document);
            """
        )
        if el:
            print("[OK] OTP input bulundu (shadow-root)")
            return el
    except:
        pass

    # Bulunamadı -> DOM dump al
    try:
        with open(OTP_DUMP_HTML, "w", encoding="utf-8") as f:
            f.write(driver.page_source)
        print(f"[WARN] OTP input bulunamadı. DOM dump alındı -> {OTP_DUMP_HTML}")
    except Exception as e:
        print(f"[WARN] DOM dump yazılamadı: {e}")

    return None


def main():
    # Chrome
    options = uc.ChromeOptions()
    options.add_argument("--no-sandbox")
    options.add_argument("--disable-dev-shm-usage")
    options.add_argument(f"--window-size={WINDOW_SIZE}")

    driver = uc.Chrome(options=options)
    try:
        driver.get(LOGIN_URL)
        WebDriverWait(driver, 15).until(EC.presence_of_element_located((By.TAG_NAME, "form")))
        print("Sayfa açıldı:", driver.title)

        # Kullanıcı & şifre (mevcut xpath’ler sende çalıştıysa aynen kalsın)
        driver.find_element(By.XPATH, '/html/body/div[1]/div/div[1]/div[2]/form/div[1]/div/input').send_keys(USERNAME)
        driver.find_element(By.XPATH, '/html/body/div[1]/div/div[1]/div[2]/form/div[2]/div/div/input').send_keys(PASSWORD)
        print("Kullanıcı adı ve şifre girildi.")

        # Önce giriş butonuna bas (OTP alanı genelde sonra gelir)
        clicked = click_login_button(driver)
        if not clicked:
            # elle basılacaksa ufak bekleme
            time.sleep(2)

        # OTP ekranının gelmesini beklerken ufak bir “any_of” beklemesi yap
        try:
            WebDriverWait(driver, 8).until(
                EC.any_of(
                    EC.presence_of_element_located((By.XPATH, "//*[contains(.,'Doğrulama') or contains(.,'OTP') or contains(.,'Onay')]")),
                    EC.url_contains("otp"),
                    EC.url_contains("verification"),
                )
            )
        except:
            pass

        # OTP input’unu bul
        otp_input = find_otp_input(driver, timeout=15)
        if otp_input:
            # Google Auth TOTP üret ve gir
            otp = pyotp.TOTP(SECRET_KEY).now()
            print("Üretilen OTP:", otp)
            try:
                otp_input.clear()
            except:
                pass
            otp_input.send_keys(otp)
        else:
            print("[INFO] OTP input bulunamadı; elle kodu girip onaylayın.")
            input("OTP’yi elle girip girişe tıkladıktan sonra ENTER’a basın...")

        # Girişin tamamlanmasını doğrula
        WebDriverWait(driver, 20).until(EC.url_changes(LOGIN_URL))
        print("✅ Giriş başarılı!")

        # Oturum verilerini kaydet
        save_cookies(driver)
        save_local_storage(driver)

    except Exception as e:
        print(f"[HATA] {e}", file=sys.stderr)
        raise
    finally:
        driver.quit()
        print("Tarayıcı kapatıldı.")


if __name__ == "__main__":
    main()