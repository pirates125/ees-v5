# -*- coding: utf-8 -*-
import json
import undetected_chromedriver as uc
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
import time

LOGIN_URL = "https://ejento.somposigorta.com.tr/dashboard/login"

def login_and_get_session(username, password):
    options = uc.ChromeOptions()
    options.add_argument("--no-sandbox")
    options.add_argument("--disable-dev-shm-usage")
    options.add_argument("--window-size=1200,800")
    driver = uc.Chrome(options=options)

    try:
        driver.get(LOGIN_URL)

        WebDriverWait(driver, 15).until(EC.presence_of_element_located((By.TAG_NAME, "form")))

        driver.find_element(By.XPATH, '/html/body/div[1]/div/div[1]/div[2]/form/div[1]/div/input').send_keys(username)
        driver.find_element(By.XPATH, '/html/body/div[1]/div/div[1]/div[2]/form/div[2]/div/div/input').send_keys(password)

        print("Kullanıcı adı ve şifre girildi. Manuel doğrulama bekleniyor...")
        input("Manuel kod girişini tamamlayın ve ENTER’a basın...")

        WebDriverWait(driver, 20).until(EC.url_changes(LOGIN_URL))
        print("✅ Giriş başarılı")

        cookies = driver.get_cookies()
        local_storage = driver.execute_script(
            "var items = {}; for (var i=0; i<localStorage.length; i++){items[localStorage.key(i)] = localStorage.getItem(localStorage.key(i));} return items;"
        )

        return {
            "cookies": cookies,
            "local_storage": local_storage
        }

    finally:
        driver.quit()

def get_quote(tc, plaka, model):
    """Login sonrası teklif alma işlemi burada yapılacak."""
    # TODO: Buraya teklif formunun URL’si ve XPATH’leri girilecek
    # Şu anda elimizde olmadığı için dolduramadım, ama burada yapılacak:
    # 1. teklif sayfasına git
    # 2. inputlara tc, plaka vs. yaz
    # 3. teklif butonuna bas
    # 4. fiyat bilgisini yakala ve döndür
    return {"ok": True, "message": "Mock teklif", "fiyat": "1200 TL"}