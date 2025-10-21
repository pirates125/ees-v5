"""
Sigorta Şirketleri Web Scraper
Bu modül Türkiye'deki major sigorta şirketlerinden teklif bilgilerini çeker
"""

import asyncio
import aiohttp
import json
import time
from datetime import datetime
from typing import Dict, List, Optional
from dataclasses import dataclass
import logging

# Logging ayarları
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

@dataclass
class InsuranceQuote:
    """Sigorta teklifi veri modeli"""
    company: str
    policy_type: str  # trafik, kasko, konut, saglik
    amount: float
    currency: str = "TL"
    valid_until: Optional[str] = None
    coverage_details: Dict = None
    scraped_at: str = None
    
    def __post_init__(self):
        if self.scraped_at is None:
            self.scraped_at = datetime.now().isoformat()

class InsuranceScraper:
    """Ana scraper sınıfı"""
    
    def __init__(self):
        self.session = None
        self.headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36',
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8',
            'Accept-Language': 'tr-TR,tr;q=0.9,en;q=0.8',
            'Accept-Encoding': 'gzip, deflate',
            'Connection': 'keep-alive',
            'Upgrade-Insecure-Requests': '1',
        }
        
        # Sigorta şirketleri konfigürasyonu
        self.companies = {
            "anadolu_sigorta": {
                "name": "Anadolu Sigorta",
                "base_url": "https://www.anadolusigorta.com.tr",
                "api_endpoint": "/api/quotes/trafik",
                "active": True
            },
            "axa_sigorta": {
                "name": "Axa Sigorta", 
                "base_url": "https://www.axa.com.tr",
                "api_endpoint": "/api/quotes/trafik",
                "active": True
            },
            "generali_sigorta": {
                "name": "Generali Sigorta",
                "base_url": "https://www.generali.com.tr", 
                "api_endpoint": "/api/quotes/trafik",
                "active": True
            },
            "allianz_sigorta": {
                "name": "Allianz Sigorta",
                "base_url": "https://www.allianz.com.tr",
                "api_endpoint": "/api/quotes/trafik", 
                "active": True
            },
            "halk_sigorta": {
                "name": "Halk Sigorta",
                "base_url": "https://www.halksigorta.com.tr",
                "api_endpoint": "/api/quotes/trafik",
                "active": True
            },
            "sompo_sigorta": {
                "name": "Sompo Sigorta",
                "base_url": "https://www.somposigorta.com.tr",
                "api_endpoint": "/api/quotes/trafik",
                "active": True
            },
            "quick_sigorta": {
                "name": "Quick Sigorta",
                "base_url": "https://www.quicksigorta.com.tr",
                "api_endpoint": "/api/quotes/trafik",
                "active": True
            },
            "atlas_sigorta": {
                "name": "Atlas Sigorta",
                "base_url": "https://www.atlassigorta.com.tr",
                "api_endpoint": "/api/quotes/trafik",
                "active": True
            },
            "koru_sigorta": {
                "name": "Koru Sigorta",
                "base_url": "https://www.korusigorta.com.tr",
                "api_endpoint": "/api/quotes/trafik",
                "active": True
            },
            "referans_sigorta": {
                "name": "Referans Sigorta",
                "base_url": "https://www.referanssigorta.com.tr",
                "api_endpoint": "/api/quotes/trafik",
                "active": True
            },
            "doga_sigorta": {
                "name": "Doğa Sigorta",
                "base_url": "https://www.dogasigorta.com.tr",
                "api_endpoint": "/api/quotes/trafik",
                "active": True
            },
            "seker_sigorta": {
                "name": "Şeker Sigorta",
                "base_url": "https://www.sekersigorta.com.tr",
                "api_endpoint": "/api/quotes/trafik",
                "active": True
            }
        }

    async def __aenter__(self):
        """Async context manager girişi"""
        connector = aiohttp.TCPConnector(limit=100, limit_per_host=10)
        timeout = aiohttp.ClientTimeout(total=30, connect=10)
        self.session = aiohttp.ClientSession(
            connector=connector,
            timeout=timeout,
            headers=self.headers
        )
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async context manager çıkışı"""
        if self.session:
            await self.session.close()

    async def scrape_trafik_quotes(self, vehicle_data: Dict) -> List[InsuranceQuote]:
        """
        Trafik sigortası tekliflerini çek
        
        Args:
            vehicle_data: Araç bilgileri
                {
                    "plaka": "34ABC123",
                    "tescil_seri": "A", 
                    "tescil_no": "123456",
                    "model_year": 2020,
                    "brand": "Toyota",
                    "model": "Corolla"
                }
        
        Returns:
            List[InsuranceQuote]: Çekilen teklifler
        """
        quotes = []
        
        # Paralel olarak tüm şirketlerden teklif çek
        tasks = []
        for company_id, company_info in self.companies.items():
            if company_info["active"]:
                task = self._scrape_company_quote(company_id, company_info, vehicle_data, "trafik")
                tasks.append(task)
        
        if tasks:
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
            for result in results:
                if isinstance(result, InsuranceQuote):
                    quotes.append(result)
                elif isinstance(result, Exception):
                    logger.error(f"Scraping hatası: {result}")
        
        return quotes

    async def scrape_kasko_quotes(self, vehicle_data: Dict) -> List[InsuranceQuote]:
        """Kasko sigortası tekliflerini çek"""
        quotes = []
        
        tasks = []
        for company_id, company_info in self.companies.items():
            if company_info["active"]:
                task = self._scrape_company_quote(company_id, company_info, vehicle_data, "kasko")
                tasks.append(task)
        
        if tasks:
            results = await asyncio.gather(*tasks, return_exceptions=True)
            
            for result in results:
                if isinstance(result, InsuranceQuote):
                    quotes.append(result)
                elif isinstance(result, Exception):
                    logger.error(f"Scraping hatası: {result}")
        
        return quotes

    async def _scrape_company_quote(self, company_id: str, company_info: Dict, 
                                  vehicle_data: Dict, policy_type: str) -> Optional[InsuranceQuote]:
        """Tek bir şirketten teklif çek"""
        try:
            # Mock API çağrısı - gerçek implementasyonda şirket API'lerine istek atılacak
            quote_data = await self._mock_api_call(company_info, vehicle_data, policy_type)
            
            if quote_data:
                return InsuranceQuote(
                    company=company_info["name"],
                    policy_type=policy_type,
                    amount=quote_data["amount"],
                    valid_until=quote_data.get("valid_until"),
                    coverage_details=quote_data.get("coverage_details", {})
                )
                
        except Exception as e:
            logger.error(f"{company_info['name']} scraping hatası: {e}")
            
        return None

    async def _mock_api_call(self, company_info: Dict, vehicle_data: Dict, policy_type: str) -> Optional[Dict]:
        """Mock API çağrısı - gerçek implementasyonda şirket API'lerine istek atılacak"""
        
        # Simüle edilmiş gecikme
        await asyncio.sleep(0.5)
        
        # Mock fiyat hesaplama
        base_prices = {
            "trafik": {
                "anadolu_sigorta": 2150,
                "axa_sigorta": 2200, 
                "generali_sigorta": 2100,
                "allianz_sigorta": 2250,
                "halk_sigorta": 2080
            },
            "kasko": {
                "anadolu_sigorta": 8500,
                "axa_sigorta": 8800,
                "generali_sigorta": 8200, 
                "allianz_sigorta": 9000,
                "halk_sigorta": 8100
            }
        }
        
        company_key = company_info["name"].lower().replace(" ", "_")
        
        if policy_type in base_prices and company_key in base_prices[policy_type]:
            base_price = base_prices[policy_type][company_key]
            
            # Araç yaşına göre fiyat ayarlama
            model_year = vehicle_data.get("model_year", 2020)
            current_year = datetime.now().year
            vehicle_age = current_year - model_year
            
            # Yaş faktörü (eski araçlar daha pahalı)
            age_factor = 1 + (vehicle_age * 0.05)
            
            final_price = base_price * age_factor
            
            return {
                "amount": round(final_price, 2),
                "valid_until": (datetime.now().timestamp() + 86400) * 1000,  # 24 saat
                "coverage_details": {
                    "minimum_coverage": True if policy_type == "trafik" else False,
                    "comprehensive": True if policy_type == "kasko" else False,
                    "roadside_assistance": True,
                    "legal_protection": True
                }
            }
        
        return None

    async def scrape_all_quotes(self, vehicle_data: Dict, policy_types: List[str] = None) -> Dict[str, List[InsuranceQuote]]:
        """
        Belirtilen tüm poliçe türleri için teklifleri çek
        
        Args:
            vehicle_data: Araç bilgileri
            policy_types: Çekilecek poliçe türleri ['trafik', 'kasko', 'konut', 'saglik']
        
        Returns:
            Dict[str, List[InsuranceQuote]]: Poliçe türüne göre teklifler
        """
        if policy_types is None:
            policy_types = ["trafik", "kasko"]
        
        results = {}
        
        for policy_type in policy_types:
            logger.info(f"{policy_type.upper()} teklifleri çekiliyor...")
            
            if policy_type == "trafik":
                quotes = await self.scrape_trafik_quotes(vehicle_data)
            elif policy_type == "kasko":
                quotes = await self.scrape_kasko_quotes(vehicle_data)
            else:
                # Diğer poliçe türleri için genel scraper
                quotes = await self._scrape_general_quotes(vehicle_data, policy_type)
            
            results[policy_type] = quotes
            logger.info(f"{policy_type.upper()}: {len(quotes)} teklif bulundu")
        
        return results

    async def _scrape_general_quotes(self, vehicle_data: Dict, policy_type: str) -> List[InsuranceQuote]:
        """Genel poliçe türleri için scraper"""
        quotes = []
        
        # Mock fiyatlar - daha fazla şirket için
        mock_prices = {
            "konut": [1500, 1800, 2000, 2200, 2500, 2800, 3000, 3200, 3500, 3800, 4000, 4200, 4500, 4800, 5000],
            "saglik": [800, 950, 1100, 1250, 1400, 1600, 1800, 2000, 2200, 2400, 2600, 2800, 3000, 3200, 3400]
        }
        
        if policy_type in mock_prices:
            prices = mock_prices[policy_type]
            
            for i, (company_id, company_info) in enumerate(self.companies.items()):
                if company_info["active"]:
                    quotes.append(InsuranceQuote(
                        company=company_info["name"],
                        policy_type=policy_type,
                        amount=prices[i % len(prices)],
                        coverage_details={
                            "basic_coverage": True,
                            "premium_coverage": i >= 2
                        }
                    ))
        
        return quotes

# Async kullanım örneği
async def main():
    """Test fonksiyonu"""
    
    vehicle_data = {
        "plaka": "34ABC123",
        "tescil_seri": "A",
        "tescil_no": "123456", 
        "model_year": 2020,
        "brand": "Toyota",
        "model": "Corolla"
    }
    
    async with InsuranceScraper() as scraper:
        # Tüm teklifleri çek
        all_quotes = await scraper.scrape_all_quotes(vehicle_data, ["trafik", "kasko"])
        
        # Sonuçları yazdır
        for policy_type, quotes in all_quotes.items():
            print(f"\n{policy_type.upper()} TEKLİFLERİ:")
            for quote in quotes:
                print(f"- {quote.company}: {quote.amount} {quote.currency}")
        
        return all_quotes

if __name__ == "__main__":
    # Test çalıştır
    asyncio.run(main())
