#!/usr/bin/env python3
"""
Sompo Connector Test Script
Bu script Sompo web scraper'ını test eder
"""

import asyncio
import os
import sys
from pathlib import Path

# Backend modüllerini import etmek için path ekle
sys.path.append(str(Path(__file__).parent / "app"))

from app.connectors.sompo import SompoConnector

async def test_sompo():
    """Sompo connector'ını test et"""
    print("🔍 Sompo Connector Test Başlıyor...")
    
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
        # Sompo connector'ını başlat
        connector = SompoConnector()
        
        print(f"📋 Test Verisi: {test_payload}")
        print("🌐 Sompo'ya bağlanıyor...")
        
        # Quote'u çek
        result = await connector.fetch_quote(test_payload)
        
        print("✅ Başarılı!")
        print(f"🏢 Şirket: {result['company']}")
        print(f"💰 Prim: {result['premium']} {result['currency']}")
        print(f"⏰ Geçerlilik: {result['validUntil']}")
        print(f"🛡️ Teminatlar: {len(result['coverages'])} adet")
        
        return result
        
    except Exception as e:
        print(f"❌ Hata: {e}")
        print("💡 Çözüm önerileri:")
        print("   - SOMPO_URL, SOMPO_USER, SOMPO_PASS env değişkenlerini kontrol edin")
        print("   - Proxy ayarlarını kontrol edin")
        print("   - Playwright browser'ının yüklü olduğundan emin olun")
        return None

async def main():
    """Ana test fonksiyonu"""
    print("=" * 50)
    print("🚀 SOMPO WEB SCRAPER TEST")
    print("=" * 50)
    
    # Environment variables kontrolü
    required_envs = ["SOMPO_URL", "SOMPO_USER", "SOMPO_PASS"]
    missing_envs = [env for env in required_envs if not os.getenv(env)]
    
    if missing_envs:
        print(f"⚠️  Eksik environment variables: {missing_envs}")
        print("📝 .env dosyası oluşturun:")
        for env in missing_envs:
            print(f"   {env}=your_value_here")
        return
    
    # Test'i çalıştır
    result = await test_sompo()
    
    if result:
        print("\n🎉 Test başarılı! Sompo connector çalışıyor.")
        print("📊 Sonraki adım: Frontend entegrasyonu")
    else:
        print("\n💥 Test başarısız! Sorunları çözelim.")
    
    print("=" * 50)

if __name__ == "__main__":
    asyncio.run(main())
