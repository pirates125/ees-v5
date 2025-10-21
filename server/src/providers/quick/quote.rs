use crate::browser::{create_webdriver_client, SessionManager};
use crate::config::Config;
use crate::http::{ApiError, QuoteRequest, QuoteResponse};
use crate::providers::quick::login::login_to_quick;
use crate::providers::quick::parser::parse_quick_quote;
use crate::providers::quick::selectors::QuickSelectors;
use fantoccini::Locator;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn fetch_quick_quote(
    config: Arc<Config>,
    request: QuoteRequest,
) -> Result<QuoteResponse, ApiError> {
    let scrape_start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    let client = create_webdriver_client(&config).await
        .map_err(|e| ApiError::WebDriverError(format!("WebDriver: {}", e)))?;
    
    let session_manager = SessionManager::new(&config.session_dir);
    
    if let Err(e) = login_to_quick(&client, config.clone(), &session_manager).await {
        let _ = client.close().await;
        return Err(e);
    }
    
    // Form doldur
    let plate = &request.vehicle.plate;
    for selector in QuickSelectors::PLATE_INPUTS {
        if let Ok(elem) = client.find(Locator::Css(selector)).await {
            if let Ok(_) = elem.send_keys(plate).await {
                break;
            }
        }
    }
    
    // Submit
    for selector in QuickSelectors::FORM_SUBMIT_BUTTONS {
        if let Ok(elem) = client.find(Locator::Css(selector)).await {
            if let Ok(_) = elem.click().await {
                break;
            }
        }
    }
    
    tokio::time::sleep(tokio::time::Duration::from_millis(8000)).await;
    
    let result = parse_quick_quote(&client, request.quote_meta.request_id, scrape_start).await;
    
    let _ = client.close().await;
    
    result
}

