use crate::http::{ApiError, Coverage, Installment, PremiumDetail, QuoteResponse, Timings};
use crate::providers::quick::selectors::QuickSelectors;
use crate::utils::parse_tl_price;
use fantoccini::{Client, Locator};
use rust_decimal::Decimal;

pub async fn parse_quick_quote(
    client: &Client,
    request_id: String,
    scrape_start_ms: u64,
) -> Result<QuoteResponse, ApiError> {
    let mut price_value: Option<Decimal> = None;
    
    for selector in QuickSelectors::PRICE_ELEMENTS {
        if let Ok(elements) = client.find_all(Locator::Css(selector)).await {
            for elem in elements {
                if let Ok(text) = elem.text().await {
                    if text.contains("TL") {
                        if let Ok(parsed) = parse_tl_price(&text) {
                            let value = parsed.to_string().parse::<f64>().unwrap_or(0.0);
                            if value >= 1000.0 && value <= 50000.0 {
                                price_value = Some(parsed);
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
    
    let net = premium / Decimal::from_str_exact("1.18").unwrap();
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
            net: net.round_dp(2),
            gross: premium.round_dp(2),
            taxes: taxes.round_dp(2),
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

