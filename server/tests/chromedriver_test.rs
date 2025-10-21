use sigorta_server::browser::driver::create_webdriver_client;

#[tokio::test]
async fn test_chromedriver_connection() {
    // Test ChromeDriver bağlantısı
    let webdriver_url = "http://localhost:9515";
    
    println!("ChromeDriver'a bağlanılıyor: {}", webdriver_url);
    
    let client_result = create_webdriver_client(webdriver_url).await;
    
    match client_result {
        Ok(_client) => {
            println!("✅ ChromeDriver bağlantısı başarılı!");
            // assert!(true);
        }
        Err(e) => {
            println!("❌ ChromeDriver bağlantısı başarısız: {}", e);
            println!("Not: ChromeDriver'ın http://localhost:9515'te çalıştığından emin olun");
            println!("Başlatmak için: chromedriver --port=9515");
            // Test başarısız olacak
            panic!("ChromeDriver connection failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Manuel olarak çalıştırılmalı
async fn test_chromedriver_navigation() {
    use fantoccini::Locator;
    
    let webdriver_url = "http://localhost:9515";
    let mut client = create_webdriver_client(webdriver_url)
        .await
        .expect("ChromeDriver'a bağlanılamadı");
    
    // Google'a git
    client
        .goto("https://www.google.com")
        .await
        .expect("Navigation failed");
    
    println!("✅ Google'a başarıyla yönlendirildi");
    
    // Title kontrol et
    let title = client.title().await.expect("Title alınamadı");
    println!("Page title: {}", title);
    assert!(title.contains("Google"));
    
    // Cleanup
    client.close().await.ok();
}

