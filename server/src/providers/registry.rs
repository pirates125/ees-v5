use crate::config::Config;
use crate::http::{ProviderInfo, ProvidersResponse};
use crate::providers::anadolu::AnadoluProvider;
use crate::providers::axa::AxaProvider;
use crate::providers::base::InsuranceProvider;
use crate::providers::quick::QuickProvider;
use crate::providers::sompo::SompoProvider;
use std::sync::Arc;

pub struct ProviderRegistry {
    providers: Vec<Arc<dyn InsuranceProvider>>,
}

impl ProviderRegistry {
    pub fn new(config: Arc<Config>) -> Self {
        let mut providers: Vec<Arc<dyn InsuranceProvider>> = Vec::new();
        
        // Tüm provider'ları ekle
        providers.push(Arc::new(SompoProvider::new(config.clone())));
        providers.push(Arc::new(QuickProvider::new(config.clone())));
        providers.push(Arc::new(AxaProvider::new(config.clone())));
        providers.push(Arc::new(AnadoluProvider::new(config.clone())));
        
        Self { providers }
    }
    
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn InsuranceProvider>> {
        self.providers
            .iter()
            .find(|p| p.name().eq_ignore_ascii_case(name))
            .cloned()
    }
    
    pub fn get_active_providers(&self) -> Vec<Arc<dyn InsuranceProvider>> {
        self.providers
            .iter()
            .filter(|p| p.is_active())
            .cloned()
            .collect()
    }
    
    pub fn get_all_providers(&self) -> Vec<Arc<dyn InsuranceProvider>> {
        self.providers.clone()
    }
    
    pub fn get_providers_info(&self) -> ProvidersResponse {
        let providers: Vec<ProviderInfo> = self
            .providers
            .iter()
            .map(|p| ProviderInfo {
                name: p.name().to_string(),
                active: p.is_active(),
                reason: p.inactive_reason(),
                supported_products: p.supported_products(),
            })
            .collect();
        
        let active_count = providers.iter().filter(|p| p.active).count();
        
        ProvidersResponse {
            total: providers.len(),
            active_count,
            providers,
        }
    }
}

