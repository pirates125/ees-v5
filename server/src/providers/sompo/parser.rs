use crate::http::{ApiError, Coverage, Installment, PremiumDetail, QuoteResponse, Timings};
use crate::providers::sompo::selectors::SompoSelectors;
use crate::utils::parse_tl_price;
use fantoccini::{Client, Locator};
use rust_decimal::Decimal;

pub async fn parse_quote_from_page(
    client: &Client,
    request_id: String,
    scrape_start_ms: u64,
) -> Result<QuoteResponse, ApiError> {
    tracing::info!("📊 Fiyat bilgisi parse ediliyor...");
    
    // Fiyat elementlerini ara
    let mut price_value: Option<Decimal> = None;
    
    for selector in SompoSelectors::PRICE_ELEMENTS {
        if let Ok(elements) = client.find_all(Locator::Css(selector)).await {
            for elem in elements {
                if let Ok(text) = elem.text().await {
                    if text.contains("TL") && !text.contains("000.000") {
                        // Çok yüksek fiyatları filtrele
                        if let Ok(parsed) = parse_tl_price(&text) {
                            let value = parsed.to_string().parse::<f64>().unwrap_or(0.0);
                            if value >= 1000.0 && value <= 50000.0 {
                                tracing::info!("✅ Fiyat bulundu: {} -> {:?}", text, parsed);
                                price_value = Some(parsed);
                                break;
                            } else {
                                tracing::debug!("⚠️ Aralık dışı fiyat atlandı: {} -> {}", text, value);
                            }
                        }
                    }
                }
            }
            if price_value.is_some() {
                break;
            }
        }
    }
    
    let premium = price_value.ok_or_else(|| {
        ApiError::ParseError("Fiyat bilgisi bulunamadı".to_string())
    })?;
    
    // Vergileri hesapla (örnek: %18 KDV varsayımı)
    let net = premium / Decimal::from_str_exact("1.18").unwrap();
    let taxes = premium - net;
    
    let premium_detail = PremiumDetail {
        net: net.round_dp(2),
        gross: premium.round_dp(2),
        taxes: taxes.round_dp(2),
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
            per_installment: (premium / Decimal::from(3)).round_dp(2),
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

