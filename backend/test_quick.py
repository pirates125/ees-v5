#!/usr/bin/env python3
"""
Quick Sigorta Connector Test Script
SMS Auth ve mesai saati kontrolü test edilir
"""

import asyncio
import os
import sys
from datetime import datetime

# Backend modüllerini import et
sys.path.append('/Users/kaanoba/workspace/scrapper-project/EESigorta/backend')

from app.connectors.quick import QuickConnector

def print_header():
    print("=" * 50)
    print("🚀 QUICK WEB SCRAPER TEST")
    print("=" * 50)

def print_footer():
    print("=" * 50)

async def test_quick_connector():
    print_header()
    print("🔍 Quick Connector Test Başlıyor...")
    
    # Test verisi
    test_payload = {
        "product": "trafik",
        "plate": "34ABC123",
        "extras": {
            "ruhsatSeri": "A123456",
            "ruhsatKod": "123456789"
        }
    }
    
    try:
        # Quick connector'ını başlat
        connector = QuickConnector()
        
        print(f"📋 Test Verisi: {test_payload}")
        print("🌐 Quick'e bağlanıyor...")
        
        # Mesai saati kontrolü
        is_business_hours = connector._is_business_hours()
        print(f"🕒 Mesai Saati: {'✅ Evet' if is_business_hours else '❌ Hayır'}")
        
        # SMS Auth durumu
        sms_status = connector._get_sms_auth_status()
        print(f"📱 SMS Auth: {'✅ Mevcut' if sms_status['available'] else '❌ Mevcut Değil'}")
        print(f"📝 Sebep: {sms_status['reason']}")
        print(f"🕒 Mesai Saatleri: {sms_status['business_hours']}")
        print(f"⏰ Şu Anki Zaman: {sms_status['current_time']}")
        
        # Quote'u çek
        result = await connector.fetch_quote(test_payload)
        
        print("\n✅ BAŞARILI!")
        print(f"🏢 Şirket: {result['company']}")
        print(f"💰 Premium: {result['premium']} {result['currency']}")
        print(f"🆔 ID: {result['id']}")
        print(f"⏰ Geçerlilik: {result['validUntil']}")
        
        # SMS durumu detayları
        extras = result.get('extras', {})
        if 'sms_status' in extras:
            print(f"\n📱 SMS Durumu:")
            sms_info = extras['sms_status']
            print(f"   - Mevcut: {'✅' if sms_info['available'] else '❌'}")
            print(f"   - Sebep: {sms_info['reason']}")
            print(f"   - Mesai Saatleri: {sms_info['business_hours']}")
        
        if extras.get('mock_data'):
            print(f"\n⚠️ Mock Data Kullanıldı (Mesai Saati Dışı)")
        
        print("\n🎉 Test başarıyla tamamlandı!")
        
    except Exception as e:
        print(f"\n❌ Hata: {e}")
        print("\n💡 Çözüm önerileri:")
        print("   - QUICK_URL, QUICK_USER, QUICK_PASS env değişkenlerini kontrol edin")
        print("   - Proxy ayarlarını kontrol edin")
        print("   - Playwright browser'ının yüklü olduğundan emin olun")
        print("   - Mesai saatleri içinde test edin (Pazartesi-Cuma 09:00-18:00)")
        
        print("\n💥 Test başarısız! Sorunları çözelim.")
    
    print_footer()

if __name__ == "__main__":
    asyncio.run(test_quick_connector())
