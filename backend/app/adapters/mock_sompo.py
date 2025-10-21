def trafik_quote(payload):
    # Sompo mock; istersek Quick ile aynı kalabilir
    fiyat = "2.250,00 TL"
    teklifler = [
        {"sirket": "Sompo Sigorta", "prim": fiyat, "fiyat": fiyat, "durum": "✅ Mevcut", "sira": 1},
        {"sirket": "Generali Sigorta", "prim": "2.000,00 TL", "fiyat": "2.300,00 TL", "durum": "⏳ Beklemede", "sira": 2},
    ]
    return fiyat, teklifler