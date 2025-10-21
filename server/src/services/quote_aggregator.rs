use crate::http::{ApiError, QuoteRequest, QuoteResponse};
use crate::providers::ProviderRegistry;
use std::sync::Arc;
use tokio::task::JoinSet;

pub struct QuoteAggregator {
    registry: Arc<ProviderRegistry>,
}

impl QuoteAggregator {
    pub fn new(registry: Arc<ProviderRegistry>) -> Self {
        Self { registry }
    }
    
    pub async fn fetch_all_quotes(
        &self,
        request: QuoteRequest,
    ) -> Result<Vec<QuoteResponse>, ApiError> {
        let active_providers = self.registry.get_active_providers();
        
        if active_providers.is_empty() {
            return Err(ApiError::ProviderInactive(
                "Hiç aktif provider yok".to_string()
            ));
        }
        
        tracing::info!(
            "🚀 {} aktif provider'dan teklif alınıyor...",
            active_providers.len()
        );
        
        // JoinSet ile paralel task'lar oluştur
        let mut join_set = JoinSet::new();
        
        for provider in active_providers {
            let req = request.clone();
            join_set.spawn(async move {
                let provider_name = provider.name().to_string();
                tracing::info!("⏳ {} - Teklif alınıyor...", provider_name);
                
                match provider.fetch_quote(req).await {
                    Ok(quote) => {
                        tracing::info!(
                            "✅ {} - Başarılı: {} TRY",
                            provider_name,
                            quote.premium.gross
                        );
                        Ok(quote)
                    }
                    Err(e) => {
                        tracing::error!("❌ {} - Hata: {}", provider_name, e);
                        Err((provider_name, e))
                    }
                }
            });
        }
        
        // Tüm sonuçları topla
        let mut quotes = Vec::new();
        let mut errors = Vec::new();
        
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok(quote)) => quotes.push(quote),
                Ok(Err((provider, error))) => errors.push((provider, error)),
                Err(e) => {
                    tracing::error!("❌ Task join hatası: {}", e);
                }
            }
        }
        
        tracing::info!(
            "📊 Sonuç: {} başarılı, {} hata",
            quotes.len(),
            errors.len()
        );
        
        if quotes.is_empty() {
            return Err(ApiError::Unknown(format!(
                "Hiç teklif alınamadı. Hatalar: {:?}",
                errors
            )));
        }
        
        Ok(quotes)
    }
}

