from flask import Flask, request, jsonify, send_from_directory
from flask_cors import CORS
import os
import asyncio
import sys
from .store import db
from .adapters import mock_quick, mock_sompo
from .services.policy_service import create_policy

# Scraper import
sys.path.append(os.path.join(os.path.dirname(__file__), '..'))
from scrapers.insurance_scraper import InsuranceScraper

app = Flask(__name__)
CORS(app)

API_OK = {"ok": True}

# ========== DASHBOARD & LISTELER ==========

@app.route("/api/dashboard", methods=["GET"])
def dashboard():
    teklif = len(db.offers)
    police = len(db.policies)
    yenileme = sum(1 for p in db.policies if p.get("yenileme"))
    
    # Gerçek istatistikleri hesapla
    total_revenue = 0
    total_commission = 0
    
    for policy in db.policies:
        if policy.get("tutar"):
            try:
                tutar = float(policy["tutar"].replace(".", "").replace(",", ".").replace(" TL", ""))
                total_revenue += tutar
                # Komisyon %10 varsayımı
                total_commission += tutar * 0.1
            except:
                pass
    
    avg_policy = total_revenue / police if police > 0 else 0
    
    # Mesai saati kontrolü
    from datetime import datetime
    now = datetime.now()
    is_business_hours = 0 <= now.weekday() <= 4 and 9 <= now.hour < 18
    
    return jsonify({
        "teklif_sayisi": teklif,
        "police_sayisi": police,
        "yenileme_sayisi": yenileme,
        "yenileme_degisim": "-11,11%",  # demo
        "totalRevenue": round(total_revenue, 2),
        "totalCommission": round(total_commission, 2),
        "avgPolicy": round(avg_policy, 2),
        "businessHours": is_business_hours,
        "currentTime": now.strftime("%Y-%m-%d %H:%M:%S"),
        "quickSmsAvailable": is_business_hours
    })

@app.route("/api/offers", methods=["GET"])
def offers():
    limit = int(request.args.get("limit", 12))
    return jsonify({"items": db.offers[:limit], "total": len(db.offers)})

# ========== MOCK TEKLİF ENDPOINT’LERİ ==========

@app.route("/api/trafik", methods=["POST"])
def trafik_sigortasi():
    data = request.json or {}
    
    # Gerçek API kullanılıp kullanılmayacağını kontrol et
    use_real_api = data.get("use_real_api", False)
    
    if use_real_api:
        # Gerçek API'leri kullan (Sompo + Quick)
        try:
            import asyncio
            from .connectors.sompo import SompoConnector
            from .connectors.quick import QuickConnector
            
            async def get_real_quotes():
                # Sompo ve Quick'i paralel çalıştır
                sompo_task = asyncio.create_task(
                    SompoConnector().fetch_quote({
                        "product": "trafik",
                        "plate": data.get("plaka", "34ABC123"),
                        "extras": {
                            "ruhsatSeri": data.get("tescilSeri", ""),
                            "ruhsatKod": data.get("tescilNo", "")
                        }
                    })
                )
                
                quick_task = asyncio.create_task(
                    QuickConnector().fetch_quote({
                        "product": "trafik",
                        "plate": data.get("plaka", "34ABC123"),
                        "extras": {
                            "ruhsatSeri": data.get("tescilSeri", ""),
                            "ruhsatKod": data.get("tescilNo", "")
                        }
                    })
                )
                
                # Her ikisini de bekle
                sompo_result = await sompo_task
                quick_result = await quick_task
                
                return sompo_result, quick_result
            
            # Async fonksiyonu çalıştır
            sompo_result, quick_result = asyncio.run(get_real_quotes())
            
            # Sonuçları teklif formatına çevir
            real_teklifler = []
            
            # Sompo teklifi
            sompo_teklif = {
                "durum": "✅ Mevcut",
                "fiyat": f"{sompo_result['premium']:,.2f} TL",
                "prim": f"{sompo_result['premium']:,.2f} TL",
                "sira": 1,
                "sirket": "Sompo Sigorta (Gerçek)"
            }
            real_teklifler.append(sompo_teklif)
            
            # Quick teklifi
            quick_status = quick_result.get("extras", {}).get("sms_status", {})
            if quick_status.get("available", False):
                quick_teklif = {
                    "durum": "✅ Mevcut",
                    "fiyat": f"{quick_result['premium']:,.2f} TL",
                    "prim": f"{quick_result['premium']:,.2f} TL",
                    "sira": 1,
                    "sirket": "Quick Sigorta (Gerçek)"
                }
            else:
                quick_teklif = {
                    "durum": "⚠️ Mesai Saati Dışı",
                    "fiyat": f"{quick_result['premium']:,.2f} TL",
                    "prim": f"{quick_result['premium']:,.2f} TL",
                    "sira": 2,
                    "sirket": "Quick Sigorta (Mock)"
                }
            real_teklifler.append(quick_teklif)
            
            # Diğer mock'ları da ekle
            q_fiyat, q_list = mock_quick.trafik_quote(data)
            s_fiyat, s_list = mock_sompo.trafik_quote(data)
            
            # Mock'lardan sadece Sompo ve Quick'i çıkar
            other_mocks = [t for t in q_list + s_list if "Sompo" not in t["sirket"] and "Quick" not in t["sirket"]]
            
            teklifler = real_teklifler + other_mocks
            en_uygun_fiyat = min(sompo_result['premium'], quick_result['premium'])
            
        except Exception as e:
            print(f"❌ Gerçek API hatası: {e}")
            # Hata durumunda mock'a geri dön
            q_fiyat, q_list = mock_quick.trafik_quote(data)
            s_fiyat, s_list = mock_sompo.trafik_quote(data)
            teklifler = sorted(q_list + s_list, key=lambda x: x["sira"])
            en_uygun_fiyat = q_fiyat
    else:
        # Mock data kullan
        q_fiyat, q_list = mock_quick.trafik_quote(data)
        s_fiyat, s_list = mock_sompo.trafik_quote(data)
        teklifler = sorted(q_list + s_list, key=lambda x: x["sira"])
        en_uygun_fiyat = q_fiyat

    # “kim hangi girişten kesti” için poliçe kaydı (mock: başarılı senaryo)
    policy_id = create_policy(
        user_id=data.get("user_id","emrah"),
        user_name=data.get("user_name","EMRAH ÖZTÜRK"),
        giris_kanali=data.get("giris_kanali","panel"),
        urun="trafik",
        tutar=q_fiyat,
        plaka=data.get("plaka"),
        musteri_ad=data.get("musteri_ad", "-"),
        pdf_file="policy_demo.pdf",
        yenileme=False
    )

    return jsonify({
        "ok": True,
        "message": f"{len(teklifler)} sigorta şirketinden trafik teklifi alındı",
        "en_uygun_teklif": f"{float(en_uygun_fiyat):,.2f} TL" if isinstance(en_uygun_fiyat, (int, float)) else str(en_uygun_fiyat),
        "teklifler": teklifler,
        "policy_id": policy_id,
        "real_api_used": use_real_api
    })

@app.route("/api/kasko", methods=["POST"])
def kasko_sigortasi():
    data = request.json or {}
    fiyatlar, teklifler = mock_quick.kasko_quote(data)
    en_uygun = fiyatlar["pesin"]

    policy_id = create_policy(
        user_id=data.get("user_id","emrah"),
        user_name=data.get("user_name","EMRAH ÖZTÜRK"),
        giris_kanali=data.get("giris_kanali","panel"),
        urun="kasko",
        tutar=en_uygun,
        plaka=data.get("plaka"),
        musteri_ad=data.get("musteri_ad", "-"),
        pdf_file="policy_demo.pdf",
        yenileme=False
    )

    return jsonify({
        "ok": True,
        "message": f"{len(teklifler)} sigorta şirketinden kasko teklifi alındı",
        "en_uygun_teklif": en_uygun,
        "teklifler": teklifler,
        "pesin_fiyat": fiyatlar["pesin"],
        "taksitli_fiyat": fiyatlar["taksitli"],
        "policy_id": policy_id
    })

@app.route("/api/seyahat-saglik", methods=["POST"])
def seyahat_saglik():
    data = request.json or {}
    fiyat, teklifler = mock_quick.seyahat_quote(data)

    policy_id = create_policy(
        user_id=data.get("user_id","emrah"),
        user_name=data.get("user_name","EMRAH ÖZTÜRK"),
        giris_kanali=data.get("giris_kanali","panel"),
        urun="seyahat",
        tutar=fiyat,
        plaka=None,
        musteri_ad=data.get("musteri_ad", "-"),
        pdf_file="policy_demo.pdf",
        yenileme=False
    )

    return jsonify({
        "ok": True,
        "message": f"{len(teklifler)} sigorta şirketinden seyahat sağlık teklifi alındı",
        "en_uygun_teklif": fiyat,
        "teklifler": teklifler,
        "policy_id": policy_id
    })

# ========== POLİÇE KAYDI & PDF ==========

@app.route("/api/policy", methods=["POST"])
def create_policy_manual():
    data = request.json or {}
    pid = create_policy(
        user_id=data.get("user_id","emrah"),
        user_name=data.get("user_name","EMRAH ÖZTÜRK"),
        giris_kanali=data.get("giris_kanali","panel"),
        urun=data.get("urun","trafik"),
        tutar=data.get("tutar","0 TL"),
        plaka=data.get("plaka"),
        musteri_ad=data.get("musteri_ad","-"),
        pdf_file=data.get("pdf_file","policy_demo.pdf"),
        yenileme=bool(data.get("yenileme", False))
    )
    return jsonify({"ok": True, "policy_id": pid})

@app.route("/api/policy/<int:pid>/pdf", methods=["GET"])
def get_policy_pdf(pid):
    # ilgili policy kaydındaki pdf_file’u bul
    file_name = "policy_demo.pdf"
    for p in db.policies:
        if p["id"] == pid and p.get("pdf_file"):
            file_name = p["pdf_file"]
            break
    folder = os.path.join(os.path.dirname(__file__), "pdf")
    return send_from_directory(folder, file_name, as_attachment=True)

# ========== SAĞLIK TAMAMLAYICI (placeholder) ==========

@app.route("/api/tamamlayici-saglik", methods=["POST"])
def tamamlayici_saglik():
    return jsonify({"ok": True, "message": "Yakında aktif olacak", "price": "Yakında"})

# ========== WEB SCRAPER ENDPOINTS ==========

@app.route("/api/scraper/trafik-quotes", methods=["POST"])
def get_trafik_quotes():
    """Web scraper ile trafik sigortası tekliflerini çek"""
    try:
        data = request.json
        vehicle_data = {
            "plaka": data.get("plaka", "34ABC123"),
            "tescil_seri": data.get("tescilSeri", "A"),
            "tescil_no": data.get("tescilNo", "123456"),
            "model_year": data.get("modelYear", 2020),
            "brand": data.get("brand", "Toyota"),
            "model": data.get("model", "Corolla")
        }
        
        # Async scraper çalıştır
        async def run_scraper():
            async with InsuranceScraper() as scraper:
                quotes = await scraper.scrape_trafik_quotes(vehicle_data)
                return quotes
        
        # Event loop oluştur ve çalıştır
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        quotes = loop.run_until_complete(run_scraper())
        loop.close()
        
        # Sonuçları formatla
        formatted_quotes = []
        for quote in quotes:
            formatted_quotes.append({
                "company": quote.company,
                "amount": quote.amount,
                "currency": quote.currency,
                "valid_until": quote.valid_until,
                "coverage_details": quote.coverage_details,
                "scraped_at": quote.scraped_at
            })
        
        return jsonify({
            "success": True,
            "quotes": formatted_quotes,
            "total_quotes": len(formatted_quotes),
            "scraped_at": asyncio.get_event_loop().time() if asyncio.get_event_loop().is_running() else None
        })
        
    except Exception as e:
        return jsonify({
            "success": False,
            "error": str(e),
            "message": "Teklifler çekilirken hata oluştu"
        }), 500

@app.route("/api/scraper/kasko-quotes", methods=["POST"])
def get_kasko_quotes():
    """Web scraper ile kasko sigortası tekliflerini çek"""
    try:
        data = request.json
        vehicle_data = {
            "plaka": data.get("plaka", "34ABC123"),
            "tescil_seri": data.get("tescilSeri", "A"),
            "tescil_no": data.get("tescilNo", "123456"),
            "model_year": data.get("modelYear", 2020),
            "brand": data.get("brand", "Toyota"),
            "model": data.get("model", "Corolla")
        }
        
        # Async scraper çalıştır
        async def run_scraper():
            async with InsuranceScraper() as scraper:
                quotes = await scraper.scrape_kasko_quotes(vehicle_data)
                return quotes
        
        # Event loop oluştur ve çalıştır
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        quotes = loop.run_until_complete(run_scraper())
        loop.close()
        
        # Sonuçları formatla
        formatted_quotes = []
        for quote in quotes:
            formatted_quotes.append({
                "company": quote.company,
                "amount": quote.amount,
                "currency": quote.currency,
                "valid_until": quote.valid_until,
                "coverage_details": quote.coverage_details,
                "scraped_at": quote.scraped_at
            })
        
        return jsonify({
            "success": True,
            "quotes": formatted_quotes,
            "total_quotes": len(formatted_quotes),
            "scraped_at": asyncio.get_event_loop().time() if asyncio.get_event_loop().is_running() else None
        })
        
    except Exception as e:
        return jsonify({
            "success": False,
            "error": str(e),
            "message": "Teklifler çekilirken hata oluştu"
        }), 500

@app.route("/api/scraper/konut-quotes", methods=["POST"])
def get_konut_quotes():
    """Web scraper ile konut sigortası tekliflerini çek"""
    try:
        data = request.json
        property_data = {
            "address": data.get("address", "İstanbul"),
            "building_type": data.get("buildingType", "apartment"),
            "construction_year": data.get("constructionYear", 2010),
            "area": data.get("area", 100),
            "floor": data.get("floor", 1)
        }
        
        # Async scraper çalıştır
        async def run_scraper():
            async with InsuranceScraper() as scraper:
                quotes = await scraper._scrape_general_quotes(property_data, "konut")
                return quotes
        
        # Event loop oluştur ve çalıştır
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        quotes = loop.run_until_complete(run_scraper())
        loop.close()
        
        # Sonuçları formatla
        formatted_quotes = []
        for quote in quotes:
            formatted_quotes.append({
                "company": quote.company,
                "amount": quote.amount,
                "currency": quote.currency,
                "valid_until": quote.valid_until,
                "coverage_details": quote.coverage_details,
                "scraped_at": quote.scraped_at
            })
        
        return jsonify({
            "success": True,
            "quotes": formatted_quotes,
            "total_quotes": len(formatted_quotes),
            "scraped_at": asyncio.get_event_loop().time() if asyncio.get_event_loop().is_running() else None
        })
        
    except Exception as e:
        return jsonify({
            "success": False,
            "error": str(e),
            "message": "Konut teklifleri çekilirken hata oluştu"
        }), 500

@app.route("/api/scraper/saglik-quotes", methods=["POST"])
def get_saglik_quotes():
    """Web scraper ile sağlık sigortası tekliflerini çek"""
    try:
        data = request.json
        health_data = {
            "age": data.get("age", 30),
            "gender": data.get("gender", "male"),
            "smoking": data.get("smoking", False),
            "chronic_disease": data.get("chronicDisease", False),
            "coverage_type": data.get("coverageType", "comprehensive")
        }
        
        # Async scraper çalıştır
        async def run_scraper():
            async with InsuranceScraper() as scraper:
                quotes = await scraper._scrape_general_quotes(health_data, "saglik")
                return quotes
        
        # Event loop oluştur ve çalıştır
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        quotes = loop.run_until_complete(run_scraper())
        loop.close()
        
        # Sonuçları formatla
        formatted_quotes = []
        for quote in quotes:
            formatted_quotes.append({
                "company": quote.company,
                "amount": quote.amount,
                "currency": quote.currency,
                "valid_until": quote.valid_until,
                "coverage_details": quote.coverage_details,
                "scraped_at": quote.scraped_at
            })
        
        return jsonify({
            "success": True,
            "quotes": formatted_quotes,
            "total_quotes": len(formatted_quotes),
            "scraped_at": asyncio.get_event_loop().time() if asyncio.get_event_loop().is_running() else None
        })
        
    except Exception as e:
        return jsonify({
            "success": False,
            "error": str(e),
            "message": "Sağlık teklifleri çekilirken hata oluştu"
        }), 500

@app.route("/api/scraper/all-quotes", methods=["POST"])
def get_all_quotes():
    """Web scraper ile tüm poliçe türlerinden teklifleri çek"""
    try:
        data = request.json
        vehicle_data = {
            "plaka": data.get("plaka", "34ABC123"),
            "tescil_seri": data.get("tescilSeri", "A"),
            "tescil_no": data.get("tescilNo", "123456"),
            "model_year": data.get("modelYear", 2020),
            "brand": data.get("brand", "Toyota"),
            "model": data.get("model", "Corolla")
        }
        
        policy_types = data.get("policy_types", ["trafik", "kasko", "konut", "saglik"])
        
        # Async scraper çalıştır
        async def run_scraper():
            async with InsuranceScraper() as scraper:
                all_quotes = await scraper.scrape_all_quotes(vehicle_data, policy_types)
                return all_quotes
        
        # Event loop oluştur ve çalıştır
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        all_quotes = loop.run_until_complete(run_scraper())
        loop.close()
        
        # Sonuçları formatla
        formatted_results = {}
        total_quotes = 0
        
        for policy_type, quotes in all_quotes.items():
            formatted_quotes = []
            for quote in quotes:
                formatted_quotes.append({
                    "company": quote.company,
                    "amount": quote.amount,
                    "currency": quote.currency,
                    "valid_until": quote.valid_until,
                    "coverage_details": quote.coverage_details,
                    "scraped_at": quote.scraped_at
                })
            
            formatted_results[policy_type] = {
                "quotes": formatted_quotes,
                "count": len(formatted_quotes)
            }
            total_quotes += len(formatted_quotes)
        
        return jsonify({
            "success": True,
            "results": formatted_results,
            "total_quotes": total_quotes,
            "policy_types": policy_types,
            "scraped_at": asyncio.get_event_loop().time() if asyncio.get_event_loop().is_running() else None
        })
        
    except Exception as e:
        return jsonify({
            "success": False,
            "error": str(e),
            "message": "Teklifler çekilirken hata oluştu"
        }), 500

# ========== ADMIN & RAPORLAMA ==========

@app.route("/api/admin/user-operations", methods=["GET"])
def get_user_operations():
    """Kullanıcı işlemlerini getir - kim hangi girişten hangi poliçeyi kesmiş"""
    user_id = request.args.get("user_id")
    limit = int(request.args.get("limit", 50))
    
    operations = []
    for policy in db.policies:
        if user_id and policy["user_id"] != user_id:
            continue
            
        operations.append({
            "id": policy["id"],
            "user_id": policy["user_id"],
            "user_name": policy["user_name"],
            "giris_kanali": policy["giris_kanali"],
            "urun": policy["urun"],
            "tutar": policy["tutar"],
            "plaka": policy.get("plaka"),
            "musteri_ad": policy["musteri_ad"],
            "tarih": policy["tarih"],
            "yenileme": policy["yenileme"],
            "pdf_file": policy.get("pdf_file")
        })
    
    # En yeni işlemler önce
    operations.sort(key=lambda x: x["id"], reverse=True)
    
    return jsonify({
        "operations": operations[:limit],
        "total": len(operations),
        "summary": {
            "toplam_police": len(operations),
            "toplam_tutar": sum(
                float(op["tutar"].replace(".", "").replace(",", ".").replace(" TL", "")) 
                for op in operations 
                if op["tutar"] and op["tutar"] != "0 TL"
            ),
            "kullanici_sayisi": len(set(op["user_id"] for op in operations))
        }
    })

@app.route("/api/admin/users", methods=["GET"])
def get_users():
    """Tüm kullanıcıları listele"""
    users = {}
    for policy in db.policies:
        user_id = policy["user_id"]
        if user_id not in users:
            users[user_id] = {
                "user_id": user_id,
                "user_name": policy["user_name"],
                "police_sayisi": 0,
                "toplam_tutar": 0,
                "son_islem": policy["tarih"]
            }
        
        users[user_id]["police_sayisi"] += 1
        if policy["tutar"]:
            try:
                tutar = float(policy["tutar"].replace(".", "").replace(",", ".").replace(" TL", ""))
                users[user_id]["toplam_tutar"] += tutar
            except:
                pass
    
    return jsonify({"users": list(users.values())})