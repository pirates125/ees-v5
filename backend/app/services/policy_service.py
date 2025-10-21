from datetime import datetime
from ..store import db

def create_policy(user_id, user_name, giris_kanali, urun, tutar, plaka=None, musteri_ad="-", pdf_file=None, yenileme=False):
    pid = int(datetime.now().timestamp())
    item = {
        "id": pid,
        "user_id": user_id,
        "user_name": user_name,
        "giris_kanali": giris_kanali,
        "urun": urun,
        "tutar": tutar,
        "plaka": plaka,
        "musteri_ad": musteri_ad,
        "tarih": datetime.now().strftime("%d %b %Y %H:%M"),
        "yenileme": yenileme,
        "pdf_file": pdf_file or "policy_demo.pdf"
    }
    db.policies.append(item)
    return pid