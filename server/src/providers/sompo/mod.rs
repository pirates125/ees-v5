mod login;
mod parser;
mod quote;
mod quote_cdp;  // CDP implementation
mod selectors;

use crate::config::Config;
use crate::http::{ApiError, QuoteRequest, QuoteResponse};
use crate::providers::base::InsuranceProvider;
use async_trait::async_trait;
use std::sync::Arc;

pub struct SompoProvider {
    config: Arc<Config>,
}

impl SompoProvider {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl InsuranceProvider for SompoProvider {
    fn name(&self) -> &str {
        "Sompo"
    }
    
    fn is_active(&self) -> bool {
        // Credentials kontrolü
        !self.config.sompo_username.is_empty() && !self.config.sompo_password.is_empty()
    }
    
    fn inactive_reason(&self) -> Option<String> {
        if !self.is_active() {
            Some("Credentials yapılandırılmamış".to_string())
        } else {
            None
        }
    }
    
    fn supported_products(&self) -> Vec<String> {
        vec!["trafik".to_string(), "kasko".to_string()]
    }
    
    async fn fetch_quote(&self, request: QuoteRequest) -> Result<QuoteResponse, ApiError> {
        if !self.is_active() {
            return Err(ApiError::ProviderInactive(
                "Sompo credentials yapılandırılmamış".to_string()
            ));
        }
        
        // CDP implementasyonunu kullan (Playwright-style)
        tracing::info!("🚀 Sompo CDP modu kullanılıyor");
        quote_cdp::fetch_sompo_quote_cdp(self.config.clone(), request).await
    }
}

