use crate::http::{ApiError, Coverage, Installment, PremiumDetail, QuoteResponse, Timings};
use crate::providers::quick::selectors::QuickSelectors;
use fantoccini::{Client, Locator};

pub async fn parse_quick_quote(
    client: &Client,
    request_id: String,
    scrape_start_ms: u64,
) -> Result<QuoteResponse, ApiError> {
    let mut price_value: Option<f64> = None;
    
    for selector in QuickSelectors::PRICE_ELEMENTS {
        if let Ok(elements) = client.find_all(Locator::Css(selector)).await {
            for elem in elements {
                if let Ok(text) = elem.text().await {
                    if text.contains("TL") {
                        // Parse TL price (e.g., "1.234,56 TL" -> 1234.56)
                        let cleaned = text.replace("TL", "").replace(".", "").replace(",", ".").trim().to_string();
                        if let Ok(value) = cleaned.parse::<f64>() {
                            if value >= 1000.0 && value <= 50000.0 {
                                price_value = Some(value);
                                break;
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
        ApiError::ParseError("Quick fiyat bulunamadı".to_string())
    })?;
    
    let net = premium / 1.18;
    let taxes = premium - net;
    
    let scrape_elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64 - scrape_start_ms;
    
    Ok(QuoteResponse {
        request_id,
        company: "Quick".to_string(),
        product_type: "trafik".to_string(),
        premium: PremiumDetail {
            net: (net * 100.0).round() / 100.0,  // Round to 2 decimal places
            gross: (premium * 100.0).round() / 100.0,
            taxes: (taxes * 100.0).round() / 100.0,
            currency: "TRY".to_string(),
        },
        installments: vec![
            Installment {
                count: 1,
                per_installment: premium,
                total: premium,
            },
        ],
        coverages: vec![
            Coverage {
                code: "TRAFIK_ZORUNLU".to_string(),
                name: "Zorunlu Trafik Sigortası".to_string(),
                limit: None,
                included: true,
            },
        ],
        warnings: vec![],
        raw: None,
        timings: Some(Timings {
            queued_ms: 0,
            scrape_ms: scrape_elapsed,
        }),
    })
}

