use crate::http::{ApiError, QuoteRequest, QuoteResponse};
use async_trait::async_trait;

#[async_trait]
pub trait InsuranceProvider: Send + Sync {
    /// Provider adını döndürür
    fn name(&self) -> &str;
    
    /// Provider'ın aktif olup olmadığını kontrol eder
    fn is_active(&self) -> bool;
    
    /// Aktif olmama nedenini döndürür (varsa)
    fn inactive_reason(&self) -> Option<String> {
        None
    }
    
    /// Desteklenen ürün tiplerini döndürür
    fn supported_products(&self) -> Vec<String> {
        vec!["trafik".to_string(), "kasko".to_string()]
    }
    
    /// Teklif al
    async fn fetch_quote(&self, request: QuoteRequest) -> Result<QuoteResponse, ApiError>;
}

