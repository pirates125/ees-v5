mod login;
mod parser;
mod quote;
mod selectors;

use crate::config::Config;
use crate::http::{ApiError, QuoteRequest, QuoteResponse};
use crate::providers::base::InsuranceProvider;
use async_trait::async_trait;
use std::sync::Arc;

pub struct QuickProvider {
    config: Arc<Config>,
}

impl QuickProvider {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl InsuranceProvider for QuickProvider {
    fn name(&self) -> &str {
        "Quick"
    }
    
    fn is_active(&self) -> bool {
        // TODO: Quick credentials için env variable ekle
        std::env::var("QUICK_USERNAME").is_ok()
            && std::env::var("QUICK_PASSWORD").is_ok()
    }
    
    fn inactive_reason(&self) -> Option<String> {
        if !self.is_active() {
            Some("Henüz kayıtlı değil".to_string())
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
                "Quick credentials yapılandırılmamış".to_string()
            ));
        }
        
        quote::fetch_quick_quote(self.config.clone(), request).await
    }
}

