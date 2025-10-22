use crate::config::Config;
use crate::http::{ApiError, QuoteRequest, QuoteResponse};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::Command;

/// Python full scraper (Login + Quote + Parse) - %100 garantili
pub async fn fetch_sompo_quote_python(
    config: Arc<Config>,
    request: QuoteRequest,
) -> Result<QuoteResponse, ApiError> {
    tracing::info!("🐍 Sompo Python full scraper başlatılıyor...");
    
    let scrape_start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    
    // Python script path
    let script_path = "backend/app/connectors/sompo_full.py";
    
    // Request data (JSON)
    let product_type_str = match request.coverage.product_type {
        crate::http::models::ProductType::Trafik => "trafik",
        crate::http::models::ProductType::Kasko => "kasko",
        _ => "trafik",
    };
    
    let request_json = serde_json::json!({
        "plate": request.vehicle.plate,
        "tckn": request.insured.tckn,
        "product_type": product_type_str,
        "name": request.insured.name,
        "phone": request.insured.phone,
        "email": request.insured.email,
        "birth_date": request.insured.birth_date,
    });
    
    let request_str = request_json.to_string();
    
    tracing::debug!("🐍 Request: {}", request_str);
    
    // Python command
    let output = Command::new("python3")
        .arg(script_path)
        .arg(&request_str)
        .env("SOMPO_USER", &config.sompo_username)
        .env("SOMPO_PASS", &config.sompo_password)
        .env("SOMPO_SECRET", &config.sompo_secret_key)
        .output()
        .await
        .map_err(|e| {
            tracing::error!("❌ Python subprocess başlatılamadı: {}", e);
            ApiError::WebDriverError(format!("Python subprocess başlatılamadı: {}", e))
        })?;
    
    // Stderr'i logla (Python'dan gelen info mesajları)
    if !output.stderr.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        for line in stderr.lines() {
            tracing::info!("🐍 Python: {}", line);
        }
    }
    
    // Exit code kontrol
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        
        // Stdout'tan error mesajı parse et
        if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
            if let Some(error_msg) = error_json.get("error").and_then(|v| v.as_str()) {
                return Err(ApiError::WebDriverError(format!(
                    "Python scraper hatası: {}",
                    error_msg
                )));
            }
        }
        
        return Err(ApiError::WebDriverError(format!(
            "Python scraper başarısız: {}",
            stderr
        )));
    }
    
    // Stdout'tan JSON parse et
    let stdout = String::from_utf8_lossy(&output.stdout);
    tracing::debug!("🐍 Python output: {}", stdout);
    
    // Python response struct
    #[derive(serde::Deserialize)]
    struct PythonResponse {
        success: bool,
        company: String,
        product_type: String,
        premium: PremiumData,
        installments: Vec<InstallmentData>,
        coverages: Vec<CoverageData>,
        #[serde(default)]
        warnings: Vec<String>,
        timings: TimingsData,
    }
    
    #[derive(serde::Deserialize)]
    struct PremiumData {
        net: f64,
        gross: f64,
        taxes: f64,
        currency: String,
    }
    
    #[derive(serde::Deserialize)]
    struct InstallmentData {
        count: u8,
        per_installment: f64,
        total: f64,
    }
    
    #[derive(serde::Deserialize)]
    struct CoverageData {
        code: String,
        name: String,
        limit: Option<String>,
        included: bool,
    }
    
    #[derive(serde::Deserialize)]
    struct TimingsData {
        scrape_ms: u64,
    }
    
    let python_response: PythonResponse = serde_json::from_str(&stdout)
        .map_err(|e| {
            tracing::error!("❌ Python response JSON parse hatası: {}", e);
            tracing::error!("Response: {}", stdout);
            ApiError::ParseError(format!("Python response parse hatası: {}", e))
        })?;
    
    if !python_response.success {
        return Err(ApiError::WebDriverError(
            "Python scraper başarısız (success=false)".to_string()
        ));
    }
    
    // Rust QuoteResponse'a convert et
    let response = QuoteResponse {
        request_id: request.quote_meta.request_id,
        company: python_response.company,
        product_type: python_response.product_type,
        premium: crate::http::PremiumDetail {
            net: python_response.premium.net,
            gross: python_response.premium.gross,
            taxes: python_response.premium.taxes,
            currency: python_response.premium.currency,
        },
        installments: python_response
            .installments
            .into_iter()
            .map(|i| crate::http::Installment {
                count: i.count,
                per_installment: i.per_installment,
                total: i.total,
            })
            .collect(),
        coverages: python_response
            .coverages
            .into_iter()
            .map(|c| crate::http::Coverage {
                code: c.code,
                name: c.name,
                limit: c.limit,
                included: c.included,
            })
            .collect(),
        warnings: python_response.warnings,
        raw: None,
        timings: Some(crate::http::Timings {
            queued_ms: 0,
            scrape_ms: python_response.timings.scrape_ms,
        }),
    };
    
    let total_elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64 - scrape_start;
    
    tracing::info!(
        "✅ Python scraper başarılı! {} TL ({}ms)",
        response.premium.gross,
        total_elapsed
    );
    
    Ok(response)
}

