import re

def parse_tl(text: str) -> float:
    # Accept formats like '4.350,00 TL', '4350 TL', '₺4.350', '300.000 TL'
    if not text:
        return 0.0
    
    t = text.replace("₺","").replace("TL","").strip()
    
    # Handle Turkish number format: 300.000 TL -> 300000
    # First remove thousands separators (dots), then convert comma to decimal point
    if "." in t and "," in t:
        # Format: 300.000,00 -> 300000.00
        t = t.replace(".", "").replace(",", ".")
    elif "." in t and "," not in t:
        # Format: 300.000 -> 300000 (thousands separator)
        t = t.replace(".", "")
    elif "," in t and "." not in t:
        # Format: 300,00 -> 300.00 (decimal separator)
        t = t.replace(",", ".")
    
    # Extract number
    m = re.findall(r"[0-9]+(?:\.[0-9]+)?", t)
    if not m: 
        print(f"⚠️ Fiyat parse edilemedi: '{text}' -> '{t}'")
        return 0.0
    
    result = float(m[0])
    print(f"💰 Fiyat parse edildi: '{text}' -> {result}")
    return result
