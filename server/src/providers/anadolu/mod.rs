use crate::config::Config;
use crate::http::{ApiError, QuoteRequest, QuoteResponse};
use crate::providers::base::InsuranceProvider;
use async_trait::async_trait;
use std::sync::Arc;

pub struct AnadoluProvider {
    config: Arc<Config>,
}

impl AnadoluProvider {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl InsuranceProvider for AnadoluProvider {
    fn name(&self) -> &str {
        "Anadolu"
    }
    
    fn is_active(&self) -> bool {
        std::env::var("ANADOLU_USERNAME").is_ok()
    }
    
    fn inactive_reason(&self) -> Option<String> {
        if !self.is_active() {
            Some("Henüz kayıtlı değil".to_string())
        } else {
            None
        }
    }
    
    fn supported_products(&self) -> Vec<String> {
        vec!["trafik".to_string(), "kasko".to_string(), "konut".to_string()]
    }
    
    async fn fetch_quote(&self, _request: QuoteRequest) -> Result<QuoteResponse, ApiError> {
        if !self.is_active() {
            return Err(ApiError::ProviderInactive(
                "Anadolu credentials yapılandırılmamış".to_string()
            ));
        }
        Err(ApiError::ProviderInactive("Anadolu entegrasyonu devam ediyor".to_string()))
    }
}

