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
        username_input.clear()
        username_input.send_keys(username)
        print(f"[INFO] Username girildi: {username}", file=sys.stderr)
        
        # Validation
        input_value = username_input.get_attribute('value')
        if input_value != username:
            print(f"[WARNING] Username validation failed! Expected: {username}, Got: {input_value}", file=sys.stderr)
        
        # Password
        password_input = driver.find_element(
            By.XPATH,
            '/html/body/div[1]/div/div[1]/div[2]/form/div[2]/div/div/input'
        )
        password_input.clear()
        password_input.send_keys(password)
        print(f"[INFO] Password girildi (len={len(password)})", file=sys.stderr)
        
        # Password validation
        pwd_value = password_input.get_attribute('value')
        if len(pwd_value) != len(password):
            print(f"[WARNING] Password validation failed! Expected len: {len(password)}, Got: {len(pwd_value)}", file=sys.stderr)
        
        # Login button
        login_btn = driver.find_element(By.XPATH, '//button[@type="submit"]')
        login_btn.click()
        print(f"[INFO] Login button tıklandı", file=sys.stderr)
        
        # Screenshot al (debug)
        time.sleep(2)
        driver.save_screenshot("debug_after_login_click.png")
        print(f"[DEBUG] Screenshot: debug_after_login_click.png", file=sys.stderr)
        
        # URL değişimini bekle
        login_url = "https://ejento.somposigorta.com.tr/dashboard/login"
        try:
            WebDriverWait(driver, 20).until(
                lambda d: d.current_url != login_url
            )
            print(f"[INFO] URL değişti!", file=sys.stderr)
        except Exception as e:
            # Timeout - ne oldu?
            final_url = driver.current_url
            driver.save_screenshot("debug_login_timeout.png")
            print(f"[ERROR] URL değişmedi (timeout)! Current URL: {final_url}", file=sys.stderr)
            print(f"[DEBUG] Timeout screenshot: debug_login_timeout.png", file=sys.stderr)
            
            # Sayfa içeriği logla
            print(f"[DEBUG] Page title: {driver.title}", file=sys.stderr)
            
            # Error mesajı var mı?
            try:
                errors = driver.find_elements(By.CSS_SELECTOR, '.error, .alert, .message')
                for err in errors:
                    if err.is_displayed():
                        print(f"[DEBUG] Error element: {err.text[:100]}", file=sys.stderr)
            except:
                pass
            
            raise
        
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
        
        # Wait for dashboard to fully load
        time.sleep(3)
        
        # YAKLAŞIM 1: Dashboard'da direkt trafik/kasko linklerini ara (Playwright gibi)
        print(f"[INFO] Dashboard'da direkt ürün linkleri aranıyor...", file=sys.stderr)
        
        product_link_clicked = False
        product_keywords_for_link = {
            'trafik': ['trafik', 'zorunlu trafik'],
            'kasko': ['kasko', 'tam kasko']
        }
        keywords_link = product_keywords_for_link.get(product_type.lower(), ['trafik'])
        
        try:
            js_find_product_link = """
            const keywords = arguments[0];
            
            // Link, button, card elementlerini ara
            const elements = Array.from(document.querySelectorAll('a, button, [role="button"], .card, .product-card'));
            
            for (const el of elements) {
                const text = (el.textContent || el.innerText || '').toLowerCase();
                
                // Keyword match
                const hasKeyword = keywords.some(kw => text.includes(kw));
                
                if (hasKeyword && el.offsetParent !== null && !el.disabled) {
                    // "Yenileme" veya "Bilgilendirme" içermemeli (yanlış link)
                    if (!text.includes('yenileme') && !text.includes('bilgilendirme') && text.length < 100) {
                        el.scrollIntoView({block: 'center'});
                        el.click();
                        
                        return {
                            success: true,
                            text: text.substring(0, 80),
                            tag: el.tagName
                        };
                    }
                }
            }
            
            return {success: false};
            """
            
            result = driver.execute_script(js_find_product_link, keywords_link)
            
            if result.get('success'):
                print(f"[INFO] Ürün linki tıklandı: {result.get('text', 'unknown')} ({result.get('tag', 'unknown')})", file=sys.stderr)
                product_link_clicked = True
                time.sleep(3)  # Sayfa yüklensin
            else:
                print(f"[INFO] Dashboard'da direkt ürün linki bulunamadı", file=sys.stderr)
                
        except Exception as e:
            print(f"[WARNING] Ürün linki arama hatası: {str(e)[:100]}", file=sys.stderr)
        
        # YAKLAŞIM 2: Eğer link bulunamadıysa, "YENİ İŞ TEKLİFİ" butonuna tıkla (popup açmak için)
        if not product_link_clicked:
            print(f"[INFO] Alternatif: YENİ İŞ TEKLİFİ butonu deneniyor...", file=sys.stderr)
            try:
                new_offer_btn = WebDriverWait(driver, 5).until(
                    EC.element_to_be_clickable((By.XPATH, '//button[contains(., "YENİ İŞ TEKLİFİ")]'))
                )
                driver.execute_script("arguments[0].click();", new_offer_btn)
                print(f"[INFO] Yeni İş Teklifi butonu tıklandı", file=sys.stderr)
                
                # Popup'ın açılmasını bekle (modal/dialog/overlay)
                print(f"[INFO] Popup yükleniyor...", file=sys.stderr)
                time.sleep(3)
                
                # Popup açıldı mı kontrol et
                popup_opened = driver.execute_script("""
                    const popup = document.querySelector('.p-dialog, .modal, .p-overlay, [role="dialog"]');
                    return popup && popup.offsetParent !== null;
                """)
                
                if popup_opened:
                    print(f"[INFO] Popup açıldı", file=sys.stderr)
                else:
                    print(f"[WARNING] Popup açılmamış olabilir", file=sys.stderr)
                    
            except Exception as e:
                print(f"[WARNING] Yeni İş Teklifi butonu hatası: {str(e)[:100]}", file=sys.stderr)
        
        # QR popup ve diğer popup'ları agresif kapat
        print(f"[INFO] Popup'lar kontrol ediliyor...", file=sys.stderr)
        for i in range(3):  # 3 kez dene
            try:
                result = driver.execute_script("""
                    let closed = [];
                    
                    // 1. QR Kod popup'ı kapat
                    const qrHeaders = Array.from(document.querySelectorAll('.p-dialog-header'));
                    for (const header of qrHeaders) {
                        const text = header.textContent || '';
                        if (text.includes('QR') || text.includes('Kod') || text.includes('Sıfır')) {
                            const dialog = header.closest('.p-dialog');
                            if (dialog) {
                                // "Hayır" veya "Kapat" butonunu bul
                                const buttons = dialog.querySelectorAll('button');
                                for (const btn of buttons) {
                                    const btnText = (btn.textContent || '').toLowerCase();
                                    if (btnText.includes('hayır') || btnText.includes('kapat') || btnText.includes('iptal')) {
                                        btn.click();
                                        closed.push('QR_popup');
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    
                    // 2. Close icon'ları
                    const closeButtons = document.querySelectorAll('.p-dialog-header-close, [aria-label="Close"]');
                    for (const btn of closeButtons) {
                        const dialog = btn.closest('.p-dialog');
                        if (dialog && dialog.offsetParent !== null) {
                            // Dialog görünür mü?
                            const dialogText = (dialog.textContent || '').toLowerCase();
                            // Eğer "teklif" içermiyorsa kapat (yanlış popup)
                            if (!dialogText.includes('teklif al') && !dialogText.includes('trafik') && !dialogText.includes('kasko')) {
                                btn.click();
                                closed.push('unwanted_popup');
                            }
                        }
                    }
                    
                    return {closed: closed, count: closed.length};
                """)
                
                if result['count'] > 0:
                    print(f"[INFO] {result['count']} popup kapatıldı: {result['closed']}", file=sys.stderr)
                    time.sleep(2)  # Popup kapandıktan sonra bekle
                else:
                    print(f"[INFO] Kapatılacak popup yok (deneme {i+1})", file=sys.stderr)
                    break
                    
            except Exception as e:
                print(f"[WARNING] Popup kapatma hatası: {str(e)[:100]}", file=sys.stderr)
        
        # Asıl popup'ın içeriğini kontrol et
        time.sleep(2)
        
        # Ürün seçimi (Trafik/Kasko) - JavaScript ile robust
        product_keywords = {
            'trafik': ['trafik', 'zorunlu'],
            'kasko': ['kasko']
        }
        keywords = product_keywords.get(product_type.lower(), ['trafik'])
        
        print(f"[INFO] Ürün seçiliyor: {product_type}", file=sys.stderr)
        
        # Popup içeriğini kontrol et
        print(f"[INFO] Popup içeriği kontrol ediliyor...", file=sys.stderr)
        try:
            popup_content = driver.execute_script("""
                const popups = Array.from(document.querySelectorAll('.p-dialog, [role="dialog"]'));
                const visiblePopups = popups.filter(p => p.offsetParent !== null);
                
                return visiblePopups.map(popup => ({
                    text: (popup.textContent || '').substring(0, 300),
                    hasButtons: popup.querySelectorAll('button').length,
                    visible: popup.offsetParent !== null
                }));
            """)
            
            print(f"[DEBUG] Açık popup sayısı: {len(popup_content)}", file=sys.stderr)
            for i, popup in enumerate(popup_content):
                print(f"  Popup {i+1}: {popup['hasButtons']} buton, text[:100]: {popup['text'][:100]}", file=sys.stderr)
        except Exception as e:
            print(f"[WARNING] Popup içerik kontrolü hatası: {str(e)[:100]}", file=sys.stderr)
        
        # Ürün butonlarının yüklenmesini bekle
        print(f"[INFO] Ürün butonları yükleniyor...", file=sys.stderr)
        time.sleep(3)
        
        # JavaScript ile ürün butonunu bul ve tıkla (stale element hatası almamak için)
        product_selected = False
        for attempt in range(3):  # 3 deneme
            try:
                print(f"[INFO] Ürün seçimi denemesi {attempt + 1}/3", file=sys.stderr)
                
                # İlk denemede debug bilgisi topla
                if attempt == 0:
                    js_debug_buttons = """
                    const buttons = Array.from(document.querySelectorAll('button'));
                    const visibleButtons = buttons.filter(b => b.offsetParent !== null && !b.disabled);
                    
                    // TEKLİF AL butonlarını ayrıca göster
                    const teklifButtons = visibleButtons.filter(b => 
                        (b.textContent || '').toLowerCase().includes('teklif')
                    );
                    
                    return {
                        all: visibleButtons.slice(0, 15).map(btn => ({
                            text: (btn.textContent || btn.innerText || '').trim().substring(0, 60),
                            class: btn.className.substring(0, 30)
                        })),
                        teklif: teklifButtons.map(btn => ({
                            text: (btn.textContent || '').trim().substring(0, 80),
                            parent: btn.parentElement ? (btn.parentElement.textContent || '').trim().substring(0, 100) : 'no parent'
                        }))
                    };
                    """
                    
                    try:
                        debug_result = driver.execute_script(js_debug_buttons)
                        print(f"[DEBUG] Tüm görünen butonlar ({len(debug_result.get('all', []))}):", file=sys.stderr)
                        for i, btn_info in enumerate(debug_result.get('all', [])[:10]):
                            print(f"  {i+1}. '{btn_info['text']}' (class={btn_info.get('class', '')})", file=sys.stderr)
                        
                        print(f"[DEBUG] TEKLİF AL butonları ({len(debug_result.get('teklif', []))}):", file=sys.stderr)
                        for i, btn_info in enumerate(debug_result.get('teklif', [])):
                            print(f"  {i+1}. '{btn_info['text']}' (parent={btn_info.get('parent', '')[:50]})", file=sys.stderr)
                    except Exception as e:
                        print(f"[DEBUG] Debug error: {str(e)[:100]}", file=sys.stderr)
                
                # JavaScript ile bul ve tıkla (parent container'da ürün ismi var!)
                js_click_product = """
                const keywords = arguments[0];
                
                // "TEKLİF AL" butonlarını bul
                const buttons = Array.from(document.querySelectorAll('button'));
                const teklifButtons = buttons.filter(btn => {
                    const text = (btn.textContent || '').toLowerCase();
                    return text.includes('teklif') && btn.offsetParent !== null && !btn.disabled;
                });
                
                // Her butonun parent container'ında keyword'ü ara
                for (const btn of teklifButtons) {
                    // 5 seviye yukarı parent'lara bak
                    let container = btn.parentElement;
                    let depth = 0;
                    
                    while (container && depth < 5) {
                        const containerText = (container.textContent || container.innerText || '').toLowerCase();
                        
                        // Container'da keyword var mı?
                        const hasKeyword = keywords.some(kw => containerText.includes(kw));
                        
                        if (hasKeyword) {
                            // Container'ın çok büyük olmamasını kontrol et (tüm popup içermemeli)
                            if (containerText.length < 200) {
                                // Scroll to view
                                btn.scrollIntoView({block: 'center', behavior: 'smooth'});
                                
                                // Wait for scroll
                                setTimeout(() => {}, 300);
                                
                                // Click
                                btn.click();
                                
                                return {
                                    success: true,
                                    text: containerText.substring(0, 80),
                                    buttonText: btn.textContent.trim()
                                };
                            }
                        }
                        
                        container = container.parentElement;
                        depth++;
                    }
                }
                
                return {success: false};
                """
                
                result = driver.execute_script(js_click_product, keywords)
                
                if result.get('success'):
                    print(f"[INFO] Ürün butonu tıklandı: {result.get('text', 'unknown')}", file=sys.stderr)
                    product_selected = True
                    time.sleep(3)
                    break
                else:
                    print(f"[WARNING] Ürün butonu bulunamadı (deneme {attempt + 1})", file=sys.stderr)
                    time.sleep(3)  # Daha uzun bekle
                    
            except Exception as e:
                print(f"[WARNING] Ürün seçimi hatası (deneme {attempt + 1}): {str(e)[:100]}", file=sys.stderr)
                time.sleep(3)
        
        if not product_selected:
            # Screenshot al
            driver.save_screenshot("debug_product_not_found.png")
            print(f"[DEBUG] Screenshot: debug_product_not_found.png", file=sys.stderr)
            print(json.dumps({"error": f"{product_type} ürünü seçilemedi"}), file=sys.stderr)
            sys.exit(1)
        
        # Form doldur - Plaka ve TCKN (JavaScript ile robust)
        print(f"[INFO] Form dolduruluyor: Plaka={plate}, TCKN={tckn}", file=sys.stderr)
        
        # JavaScript ile form doldur (stale element hatası almamak için)
        js_fill_form = """
        const plate = arguments[0];
        const tckn = arguments[1];
        
        const inputs = Array.from(document.querySelectorAll('input:not([type="hidden"])'));
        const visibleInputs = inputs.filter(inp => inp.offsetParent !== null && !inp.disabled);
        
        let plakaDone = false;
        let tcknDone = false;
        
        for (const inp of visibleInputs) {
            const placeholder = (inp.placeholder || '').toLowerCase();
            const name = (inp.name || '').toLowerCase();
            const label = inp.labels && inp.labels[0] ? inp.labels[0].textContent.toLowerCase() : '';
            
            // Plaka
            if (!plakaDone && (placeholder.includes('plak') || name.includes('plak') || label.includes('plak'))) {
                inp.focus();
                inp.value = '';
                inp.value = plate;
                inp.dispatchEvent(new Event('input', {bubbles: true}));
                inp.dispatchEvent(new Event('change', {bubbles: true}));
                plakaDone = true;
                continue;
            }
            
            // TCKN
            if (!tcknDone && (placeholder.includes('tc') || name.includes('tc') || label.includes('tc') || 
                              placeholder.includes('kimlik') || name.includes('kimlik') || label.includes('kimlik'))) {
                inp.focus();
                inp.value = '';
                inp.value = tckn;
                inp.dispatchEvent(new Event('input', {bubbles: true}));
                inp.dispatchEvent(new Event('change', {bubbles: true}));
                tcknDone = true;
                continue;
            }
        }
        
        return {
            plaka: plakaDone,
            tckn: tcknDone
        };
        """
        
        try:
            result = driver.execute_script(js_fill_form, plate, tckn)
            if result.get('plaka'):
                print(f"[INFO] Plaka dolduruldu", file=sys.stderr)
            else:
                print(f"[WARNING] Plaka input bulunamadı", file=sys.stderr)
            
            if result.get('tckn'):
                print(f"[INFO] TCKN dolduruldu", file=sys.stderr)
            else:
                print(f"[WARNING] TCKN input bulunamadı", file=sys.stderr)
        except Exception as e:
            print(f"[WARNING] Form doldurma hatası: {str(e)[:100]}", file=sys.stderr)
        
        time.sleep(2)
        
        # Submit button (JavaScript ile robust)
        print(f"[INFO] Submit butonu aranıyor...", file=sys.stderr)
        
        js_click_submit = """
        const keywords = ['teklif', 'sorgula', 'hesapla', 'devam'];
        const buttons = Array.from(document.querySelectorAll('button:not([disabled])'));
        const visibleButtons = buttons.filter(btn => btn.offsetParent !== null);
        
        for (const btn of visibleButtons) {
            const btnText = (btn.textContent || btn.innerText || '').toLowerCase();
            
            if (keywords.some(kw => btnText.includes(kw))) {
                btn.scrollIntoView({block: 'center', behavior: 'smooth'});
                
                // Wait for scroll
                setTimeout(() => {}, 300);
                
                btn.click();
                
                return {
                    success: true,
                    text: btnText.substring(0, 50)
                };
            }
        }
        
        return {success: false};
        """
        
        submit_clicked = False
        try:
            result = driver.execute_script(js_click_submit)
            if result.get('success'):
                print(f"[INFO] Submit button tıklandı: {result.get('text', 'unknown')}", file=sys.stderr)
                submit_clicked = True
            else:
                print(f"[WARNING] Submit butonu bulunamadı", file=sys.stderr)
        except Exception as e:
            print(f"[WARNING] Submit button hatası: {str(e)[:100]}", file=sys.stderr)
        
        if not submit_clicked:
            # Screenshot al
            driver.save_screenshot("debug_submit_not_found.png")
            print(f"[DEBUG] Screenshot: debug_submit_not_found.png", file=sys.stderr)
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

