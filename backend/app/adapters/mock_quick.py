def trafik_quote(payload):
    # gelen formu kullanarak “gerçekmiş” gibi cevap
    fiyat = "2.150,00 TL"
    teklifler = [
        {"sirket": "Quick Sigorta", "prim": fiyat, "fiyat": fiyat, "durum": "✅ Mevcut", "sira": 1},
        {"sirket": "Anadolu Sigorta", "prim": "1.950,00 TL", "fiyat": "2.150,00 TL", "durum": "✅ Mevcut", "sira": 2},
        {"sirket": "Allianz Sigorta",  "prim": "2.100,00 TL", "fiyat": "2.300,00 TL", "durum": "✅ Mevcut", "sira": 3},
        {"sirket": "Axa Sigorta",      "prim": "1.900,00 TL", "fiyat": "2.100,00 TL", "durum": "⏳ Beklemede", "sira": 4},
    ]
    return fiyat, teklifler

def kasko_quote(payload):
    pesin = "8.750,00 TL"
    taksitli = "9.100,00 TL"
    teklifler = [
        {"sirket": "Quick Sigorta (Peşin)", "prim": pesin, "fiyat": pesin, "durum": "✅ Mevcut", "sira": 1, "odeme_tipi": "Peşin"},
        {"sirket": "Quick Sigorta (Taksitli)", "prim": taksitli, "fiyat": taksitli, "durum": "✅ Mevcut", "sira": 2, "odeme_tipi": "Taksitli"},
        {"sirket": "Allianz Sigorta", "prim": "8.700,00 TL", "fiyat": "9.300,00 TL", "durum": "✅ Mevcut", "sira": 3, "odeme_tipi": "Peşin"},
    ]
    return {"pesin": pesin, "taksitli": taksitli}, teklifler

def seyahat_quote(payload):
    fiyat = "430,00 TL"
    teklifler = [
        {"sirket": "Quick Sigorta", "prim": fiyat, "fiyat": fiyat, "durum": "✅ Mevcut", "sira": 1},
        {"sirket": "Axa Sigorta", "prim": "410,00 TL", "fiyat": "480,00 TL", "durum": "⏳ Beklemede", "sira": 2},
        {"sirket": "Allianz Sigorta", "prim": "470,00 TL", "fiyat": "520,00 TL", "durum": "✅ Mevcut", "sira": 3},
    ]
    return fiyat, teklifler