use crate::http::{ApiError, Coverage, Installment, PremiumDetail, QuoteResponse, Timings};
use fantoccini::Client;

pub async fn parse_quote_from_page(
    client: &Client,
    request_id: String,
    scrape_start_ms: u64,
) -> Result<QuoteResponse, ApiError> {
    tracing::info!("📊 Fiyat bilgisi parse ediliyor...");
    
    // JavaScript ile fiyat parse et (Python benzeri)
    let premium = parse_sompo_price(client).await?;
    
    // Vergileri hesapla (örnek: %18 KDV varsayımı)
    let net = premium / 1.18;
    let taxes = premium - net;
    
    let premium_detail = PremiumDetail {
        net: (net * 100.0).round() / 100.0,  // Round to 2 decimal places
        gross: (premium * 100.0).round() / 100.0,
        taxes: (taxes * 100.0).round() / 100.0,
        currency: "TRY".to_string(),
    };
    
    // Taksit bilgileri (örnek - gerçek parselleme eklenebilir)
    let installments = vec![
        Installment {
            count: 1,
            per_installment: premium,
            total: premium,
        },
        Installment {
            count: 3,
            per_installment: ((premium / 3.0) * 100.0).round() / 100.0,  // Round to 2 decimal places
            total: premium,
        },
    ];
    
    // Temel teminatlar
    let coverages = vec![
        Coverage {
            code: "TRAFIK_ZORUNLU".to_string(),
            name: "Zorunlu Trafik Sigortası".to_string(),
            limit: None,
            included: true,
        },
    ];
    
    let scrape_elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64 - scrape_start_ms;
    
    let response = QuoteResponse {
        request_id,
        company: "Sompo".to_string(),
        product_type: "trafik".to_string(),
        premium: premium_detail,
        installments,
        coverages,
        warnings: vec![],
        raw: None,
        timings: Some(Timings {
            queued_ms: 0,
            scrape_ms: scrape_elapsed,
        }),
    };
    
    tracing::info!("✅ Quote parse edildi: {} TRY", premium);
    
    Ok(response)
}

// Helper: JavaScript ile fiyat parse et (Python benzeri)
async fn parse_sompo_price(client: &Client) -> Result<f64, ApiError> {
    tracing::info!("🔍 JavaScript ile fiyat aranıyor...");
    
    let js_parse = r#"
        (function findPrice() {
            // Python'daki price_selectors mantığı
            const selectors = [
                '.premium', '.prim', '.amount', '.price', '.fiyat', '.cost',
                '[class*="premium"]', '[class*="prim"]', '[class*="amount"]',
                '[class*="price"]', '[class*="fiyat"]', '[class*="cost"]'
            ];
            
            // Önce spesifik selector'larda ara
            for (const sel of selectors) {
                const elements = document.querySelectorAll(sel);
                for (const el of elements) {
                    const text = el.textContent?.trim() || '';
                    if (text.includes('TL') && !text.includes('000.000')) {
                        // Makul uzunlukta mı? (çok uzun içerikler muhtemelen yanlış element)
                        if (text.length < 50) {
                            return { found: true, text: text, selector: sel };
                        }
                    }
                }
            }
            
            // Fallback: Sayfada TL içeren tüm elementler (regex ile)
            const allElements = Array.from(document.querySelectorAll('*'));
            const tlRegex = /\d{1,3}(\.\d{3})*(,\d{2})?\s*TL/;
            
            for (const el of allElements) {
                // Sadece leaf node'ları kontrol et (çocuğu olmayan)
                if (el.children.length === 0) {
                    const text = el.textContent?.trim() || '';
                    if (tlRegex.test(text) && text.length < 50) {
                        return { found: true, text: text, selector: 'regex_fallback' };
                    }
                }
            }
            
            return { found: false };
        })()
    "#;
    
    match client.execute(js_parse, vec![]).await {
        Ok(result) => {
            tracing::info!("🔧 Fiyat parse sonucu: {:?}", result);
            
            if let Some(obj) = result.as_object() {
                if obj.get("found").and_then(|v| v.as_bool()).unwrap_or(false) {
                    if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                        let selector = obj.get("selector").and_then(|v| v.as_str()).unwrap_or("unknown");
                        tracing::info!("✅ Fiyat text bulundu ({}): {}", selector, text);
                        
                        // Parse TL price
                        let price = parse_tl_price(text)?;
                        
                        // Makul fiyat kontrolü (Python'daki gibi 1.000-50.000 TL)
                        if price >= 1000.0 && price <= 50000.0 {
                            tracing::info!("✅ Makul fiyat: {:.2} TL", price);
                            return Ok(price);
                        } else {
                            tracing::warn!("⚠️ Makul olmayan fiyat: {:.2} TL", price);
                            return Err(ApiError::ParseError(format!("Makul olmayan fiyat: {:.2}", price)));
                        }
                    }
                }
            }
            
            Err(ApiError::ParseError("Fiyat text'i bulunamadı".to_string()))
        }
        Err(e) => {
            tracing::warn!("⚠️ Fiyat parse hatası: {}", e);
            Err(ApiError::ParseError(format!("JavaScript hatası: {}", e)))
        }
    }
}

// Helper: TL fiyat text'ini parse et (örn: "1.234,56 TL" -> 1234.56)
fn parse_tl_price(text: &str) -> Result<f64, ApiError> {
    let cleaned = text
        .replace("TL", "")
        .replace("₺", "")
        .replace(" ", "")
        .replace(".", "")  // Binlik ayracını kaldır
        .replace(",", ".") // Ondalık ayıracı standartlaştır
        .trim()
        .to_string();
    
    cleaned.parse::<f64>()
        .map_err(|e| ApiError::ParseError(format!("Fiyat parse hatası: {} (text: '{}')", e, text)))
}

