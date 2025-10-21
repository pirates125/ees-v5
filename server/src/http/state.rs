use crate::config::Config;
use crate::db::DbPool;
use crate::providers::ProviderRegistry;
use crate::services::QuoteAggregator;
use std::sync::Arc;
use std::time::SystemTime;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub registry: Arc<ProviderRegistry>,
    pub aggregator: Arc<QuoteAggregator>,
    pub db_pool: DbPool,
    pub jwt_secret: String,
    pub start_time: SystemTime,
}

