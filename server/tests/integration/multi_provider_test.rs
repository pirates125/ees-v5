use sigorta_server::config::Config;
use sigorta_server::http::QuoteRequest;
use sigorta_server::providers::registry::ProviderRegistry;
use std::sync::Arc;
use std::time::Instant;

#[tokio::test]
#[ignore] // Manuel olarak çalıştırılmalı - credentials gerekli
async fn test_parallel_quote_fetching() {
    dotenvy::dotenv().ok();
    
    let config = Arc::new(Config::from_env());
    let registry = ProviderRegistry::new(config.clone());
    
    // Test request data
    let request = QuoteRequest {
        product_type: "trafik".to_string(),
        customer_data: serde_json::json!({
            "tckn": "12345678901",
            "plate": "34ABC123",
            "name": "Test User"
        }),
    };
    
    println!("4 provider'dan paralel teklif alınıyor...");
    let start = Instant::now();
    
    let mut handles = vec![];
    
    for provider in registry.get_all_providers() {
        let provider_name = provider.name().to_string();
        let request_clone = request.clone();
        let provider_clone = provider.clone();
        
        let handle = tokio::spawn(async move {
            println!("  → {} provider'dan teklif alınıyor...", provider_name);
            let result = provider_clone.fetch_quote(request_clone).await;
            (provider_name, result)
        });
        
        handles.push(handle);
    }
    
    // Tüm sonuçları topla
    let mut results = vec![];
    for handle in handles {
        match handle.await {
            Ok((provider, result)) => {
                results.push((provider, result));
            }
            Err(e) => {
                println!("❌ Task error: {}", e);
            }
        }
    }
    
    let duration = start.elapsed();
    
    println!("\n📊 Sonuçlar ({:.2}s):", duration.as_secs_f64());
    println!("=" .repeat(60));
    
    let mut success_count = 0;
    let mut failed_count = 0;
    
    for (provider, result) in results {
        match result {
            Ok(quote) => {
                success_count += 1;
                println!("  ✅ {}: {} TL", provider, quote.gross_premium);
            }
            Err(e) => {
                failed_count += 1;
                println!("  ❌ {}: {:?}", provider, e);
            }
        }
    }
    
    println!("=" .repeat(60));
    println!("Başarılı: {}, Başarısız: {}", success_count, failed_count);
    
    // En az bir teklif başarılı olmalı
    // assert!(success_count > 0, "Hiç teklif alınamadı!");
}

#[tokio::test]
#[ignore] // Manuel olarak çalıştırılmalı
async fn test_timeout_handling() {
    use tokio::time::{timeout, Duration};
    
    dotenvy::dotenv().ok();
    
    let config = Arc::new(Config::from_env());
    let registry = ProviderRegistry::new(config.clone());
    
    let request = QuoteRequest {
        product_type: "trafik".to_string(),
        customer_data: serde_json::json!({
            "tckn": "12345678901",
            "plate": "34ABC123",
            "name": "Test User"
        }),
    };
    
    println!("Timeout handling testi (30 saniye limit)...");
    
    for provider in registry.get_all_providers() {
        let provider_name = provider.name().to_string();
        println!("  Testing {}", provider_name);
        
        let result = timeout(
            Duration::from_secs(30),
            provider.fetch_quote(request.clone())
        ).await;
        
        match result {
            Ok(Ok(quote)) => {
                println!("    ✅ Success: {} TL", quote.gross_premium);
            }
            Ok(Err(e)) => {
                println!("    ⚠️  Provider error: {:?}", e);
            }
            Err(_) => {
                println!("    ⏱️  Timeout (>30s)");
            }
        }
    }
}

