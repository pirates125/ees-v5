use serde_json::json;

const API_URL: &str = "http://localhost:8099";

#[tokio::test]
#[ignore] // Manuel olarak çalıştırılmalı - backend running gerekli
async fn test_admin_stats_endpoint() {
    // İlk önce login olup token alalım
    let login_response = reqwest::Client::new()
        .post(format!("{}/api/v1/auth/login", API_URL))
        .json(&json!({
            "email": "adminsigorta@eesigorta.com",
            "password": "eesigorta1"
        }))
        .send()
        .await
        .expect("Login request failed");
    
    assert!(login_response.status().is_success(), "Login failed");
    
    let login_data: serde_json::Value = login_response.json().await.expect("JSON parse failed");
    let token = login_data["token"].as_str().expect("Token not found");
    
    println!("✅ Login başarılı, token alındı");
    
    // Admin stats endpoint'ini test et
    let stats_response = reqwest::Client::new()
        .get(format!("{}/api/v1/admin/stats", API_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Stats request failed");
    
    assert!(stats_response.status().is_success(), "Stats request failed");
    
    let stats: serde_json::Value = stats_response.json().await.expect("JSON parse failed");
    
    println!("📊 Admin Stats:");
    println!("  Total Users: {}", stats["totalUsers"]);
    println!("  Total Quotes: {}", stats["totalQuotes"]);
    println!("  Total Policies: {}", stats["totalPolicies"]);
    println!("  Total Revenue: {}", stats["totalRevenue"]);
    println!("  Total Commission: {}", stats["totalCommission"]);
    
    // assert!(stats["totalUsers"].as_i64().unwrap() > 0);
}

#[tokio::test]
#[ignore] // Manuel olarak çalıştırılmalı
async fn test_admin_users_list() {
    // Login
    let login_response = reqwest::Client::new()
        .post(format!("{}/api/v1/auth/login", API_URL))
        .json(&json!({
            "email": "adminsigorta@eesigorta.com",
            "password": "eesigorta1"
        }))
        .send()
        .await
        .expect("Login failed");
    
    let login_data: serde_json::Value = login_response.json().await.expect("JSON parse failed");
    let token = login_data["token"].as_str().expect("Token not found");
    
    // Users list
    let users_response = reqwest::Client::new()
        .get(format!("{}/api/v1/admin/users?limit=10&offset=0", API_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Users request failed");
    
    assert!(users_response.status().is_success());
    
    let users: serde_json::Value = users_response.json().await.expect("JSON parse failed");
    
    println!("👥 Kullanıcı Listesi:");
    if let Some(users_array) = users["users"].as_array() {
        for user in users_array {
            println!("  - {} ({}) - {}", 
                user["name"], 
                user["email"], 
                user["role"]
            );
        }
        println!("Total: {} users", users_array.len());
    }
}

#[tokio::test]
#[ignore] // Manuel olarak çalıştırılmalı
async fn test_admin_activity_logs() {
    // Login
    let login_response = reqwest::Client::new()
        .post(format!("{}/api/v1/auth/login", API_URL))
        .json(&json!({
            "email": "adminsigorta@eesigorta.com",
            "password": "eesigorta1"
        }))
        .send()
        .await
        .expect("Login failed");
    
    let login_data: serde_json::Value = login_response.json().await.expect("JSON parse failed");
    let token = login_data["token"].as_str().expect("Token not found");
    
    // Activity logs
    let logs_response = reqwest::Client::new()
        .get(format!("{}/api/v1/admin/logs?limit=10&offset=0", API_URL))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Logs request failed");
    
    assert!(logs_response.status().is_success());
    
    let logs: serde_json::Value = logs_response.json().await.expect("JSON parse failed");
    
    println!("📋 Activity Logs:");
    if let Some(logs_array) = logs["logs"].as_array() {
        for log in logs_array {
            println!("  [{}] {} - {} ({})", 
                log["created_at"], 
                log["action"], 
                log["user_id"],
                log["metadata"]
            );
        }
        println!("Total: {} logs", logs_array.len());
    }
}

