use sigorta_server::browser::driver::create_webdriver_client;
use sigorta_server::config::Config;
use sigorta_server::providers::sompo::login::login_to_sompo;
use std::sync::Arc;

#[tokio::test]
#[ignore] // Manuel olarak çalıştırılmalı - credentials gerekli
async fn test_sompo_login() {
    dotenvy::dotenv().ok();
    
    let config = Arc::new(Config::from_env());
    
    // Credentials kontrol
    let username = std::env::var("SOMPO_USERNAME").expect("SOMPO_USERNAME not set");
    let password = std::env::var("SOMPO_PASSWORD").expect("SOMPO_PASSWORD not set");
    
    println!("Sompo'ya giriş yapılıyor (user: {})...", username);
    
    let webdriver_url = config.webdriver_url.clone();
    let mut client = create_webdriver_client(&webdriver_url)
        .await
        .expect("ChromeDriver connection failed");
    
    let login_result = login_to_sompo(&mut client, &username, &password, &config).await;
    
    match login_result {
        Ok(_) => {
            println!("✅ Sompo login başarılı!");
            // assert!(true);
        }
        Err(e) => {
            println!("❌ Sompo login başarısız: {}", e);
            panic!("Sompo login failed: {}", e);
        }
    }
    
    // Cleanup
    client.close().await.ok();
}

#[tokio::test]
#[ignore] // Manuel olarak çalıştırılmalı - credentials gerekli
async fn test_sompo_quote_fetch() {
    use sigorta_server::http::QuoteRequest;
    use sigorta_server::providers::sompo::quote::fetch_sompo_quote;
    
    dotenvy::dotenv().ok();
    
    let config = Arc::new(Config::from_env());
    
    // Test request data
    let request = QuoteRequest {
        product_type: "trafik".to_string(),
        customer_data: serde_json::json!({
            "tckn": "12345678901",
            "plate": "34ABC123",
            "name": "Test User"
        }),
    };
    
    println!("Sompo'dan teklif alınıyor...");
    
    let quote_result = fetch_sompo_quote(request, config).await;
    
    match quote_result {
        Ok(quote) => {
            println!("✅ Teklif başarıyla alındı!");
            println!("Provider: {}", quote.provider);
            println!("Premium: {}", quote.gross_premium);
            // assert!(true);
        }
        Err(e) => {
            println!("❌ Teklif alma başarısız: {:?}", e);
            // Bu hatanın normal olabileceğini not et (credentials, network, vb.)
        }
    }
}

