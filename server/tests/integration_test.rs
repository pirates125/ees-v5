use reqwest;

#[tokio::test]
async fn test_health_endpoint() {
    // Server'ın çalıştığını varsay
    let response = reqwest::get("http://localhost:8099/health")
        .await
        .expect("Failed to call health endpoint");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["ok"], true);
}

#[tokio::test]
async fn test_providers_endpoint() {
    let response = reqwest::get("http://localhost:8099/api/v1/providers")
        .await
        .expect("Failed to call providers endpoint");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body["total"].as_i64().unwrap() >= 1);
}

