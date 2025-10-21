use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CacheEntry<T> {
    pub value: T,
    pub expires_at: Instant,
}

/// Simple in-memory cache service
/// TODO: For production, consider using Redis
pub struct CacheService<T: Clone> {
    store: Arc<RwLock<HashMap<String, CacheEntry<T>>>>,
    default_ttl: Duration,
}

impl<T: Clone> CacheService<T> {
    pub fn new(default_ttl_secs: u64) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            default_ttl: Duration::from_secs(default_ttl_secs),
        }
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        let store = self.store.read().await;
        if let Some(entry) = store.get(key) {
            if Instant::now() < entry.expires_at {
                return Some(entry.value.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: String, value: T) {
        self.set_with_ttl(key, value, self.default_ttl).await;
    }

    pub async fn set_with_ttl(&self, key: String, value: T, ttl: Duration) {
        let mut store = self.store.write().await;
        store.insert(
            key,
            CacheEntry {
                value,
                expires_at: Instant::now() + ttl,
            },
        );
    }

    pub async fn invalidate(&self, key: &str) {
        let mut store = self.store.write().await;
        store.remove(key);
    }

    pub async fn clear(&self) {
        let mut store = self.store.write().await;
        store.clear();
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        let mut store = self.store.write().await;
        let now = Instant::now();
        store.retain(|_, entry| entry.expires_at > now);
    }
}

impl<T: Clone> Default for CacheService<T> {
    fn default() -> Self {
        Self::new(300) // 5 minutes default TTL
    }
}

