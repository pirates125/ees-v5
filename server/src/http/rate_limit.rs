use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_requests_per_minute: usize,
}

impl RateLimiter {
    pub fn new(max_requests_per_minute: usize) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_requests_per_minute,
        }
    }

    pub async fn check_rate_limit(&self, ip: &str) -> bool {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);

        // Get or create entry for this IP
        let ip_requests = requests.entry(ip.to_string()).or_insert_with(Vec::new);

        // Remove old requests
        ip_requests.retain(|&req_time| req_time > one_minute_ago);

        // Check if limit exceeded
        if ip_requests.len() >= self.max_requests_per_minute {
            return false;
        }

        // Add current request
        ip_requests.push(now);
        true
    }

    // Cleanup old entries periodically
    pub async fn cleanup(&self) {
        let mut requests = self.requests.lock().await;
        let now = Instant::now();
        let one_minute_ago = now - Duration::from_secs(60);

        requests.retain(|_, times| {
            times.retain(|&time| time > one_minute_ago);
            !times.is_empty()
        });
    }
}

pub async fn rate_limit_middleware(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get client IP from header or connection
    let ip = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown");

    // For now, just log and continue
    // In production, you would check the rate limit here
    tracing::debug!("Request from IP: {}", ip);

    Ok(next.run(req).await)
}

