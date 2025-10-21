from datetime import datetime

class Store:
    def __init__(self):
        # sağ panel “Tekliflerim”
        self.offers = []
        # kesilen poliçeler (raporlama)
        self.policies = []
        # basit kullanıcı oturumu/kim giriş yaptı (frontend’den header ile gelebilir)
        self.sessions = {}  # {api_key: {"user":"EMRAH ÖZTÜRK", "channel":"web"}}

db = Store()

# --- demo verileri ---
now = datetime.now().strftime("%d %b %Y %H:%M")
db.offers = [
    {"id": 9001, "urun": "trafik", "plaka": "06TG932", "musteri_ad": "Muhammet Eski",
     "tarih": now, "urun_kod": "T", "kalan_text": "2 gün kaldı", "kalan_tip": "warn"},
    {"id": 9002, "urun": "kasko", "plaka": "35BCF225", "musteri_ad": "Gözde Kefal",
     "tarih": now, "urun_kod": "K", "kalan_text": "1 gün kaldı", "kalan_tip": ""},
    {"id": 9003, "urun": "seyahat", "teklif_no": "TR-AV-2025-001",
     "musteri_ad": "Caner Genç", "tarih": now, "urun_kod": "B",
     "kalan_text": "3 gün kaldı", "kalan_tip": "err"},
]

# örnek poliçe (PDF indirme testi)
db.policies = [
    {
        "id": 1001, "user_id": "emrah", "user_name": "EMRAH ÖZTÜRK",
        "giris_kanali": "panel", "urun": "trafik", "tutar": "2.150,00 TL",
        "plaka": "28ADN676", "musteri_ad": "Bayram Pektaş",
        "tarih": now, "yenileme": False, "pdf_file": "policy_demo.pdf"
    }
]