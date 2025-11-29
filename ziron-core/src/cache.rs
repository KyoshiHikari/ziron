//! Caching system for module data

use crate::module::ModuleData;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, RwLock};

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub size: usize,
}

/// Cached item with timestamp
struct CachedItem {
    data: ModuleData,
    timestamp: Instant,
}

/// Cache implementation
#[derive(Clone)]
pub struct Cache {
    inner: Arc<RwLock<CacheInner>>,
    ttl: Duration,
    max_size: usize,
}

struct CacheInner {
    data: HashMap<String, CachedItem>,
    stats: CacheStats,
}

impl Cache {
    /// Create a new cache with TTL and max size
    pub fn new(ttl: Duration, max_size: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(CacheInner {
                data: HashMap::new(),
                stats: CacheStats::default(),
            })),
            ttl,
            max_size,
        }
    }

    /// Get cached data for a module
    pub fn get(&self, key: &str) -> Option<ModuleData> {
        let mut inner = self.inner.write().unwrap();
        
        // Check if key exists and is valid
        let item_valid = inner.data.get(key)
            .map(|item| item.timestamp.elapsed() < self.ttl)
            .unwrap_or(false);
        
        if item_valid {
            inner.stats.hits += 1;
            let item = inner.data.get(key).unwrap();
            let mut data = item.data.clone();
            data.cached = true;
            Some(data)
        } else {
            // Remove expired item if it exists
            if inner.data.contains_key(key) {
                inner.data.remove(key);
            }
            inner.stats.misses += 1;
            None
        }
    }

    /// Store data in cache
    pub fn set(&self, key: String, data: ModuleData) {
        let mut inner = self.inner.write().unwrap();
        
        // Evict old items if cache is full (simple FIFO for now)
        if inner.data.len() >= self.max_size && !inner.data.contains_key(&key) {
            // Remove oldest item (simple: remove first entry)
            if let Some(oldest_key) = inner.data.keys().next().cloned() {
                inner.data.remove(&oldest_key);
            }
        }
        
        inner.data.insert(key, CachedItem {
            data,
            timestamp: Instant::now(),
        });
        inner.stats.size = inner.data.len();
    }

    /// Invalidate cache for a specific module or all modules
    pub fn invalidate(&self, module: Option<&str>) {
        let mut inner = self.inner.write().unwrap();
        
        if let Some(module) = module {
            // Remove specific module
            if inner.data.remove(module).is_some() {
                inner.stats.size = inner.data.len();
            }
        } else {
            // Clear all cache
            inner.data.clear();
            inner.stats.size = 0;
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let inner = self.inner.read().unwrap();
        CacheStats {
            hits: inner.stats.hits,
            misses: inner.stats.misses,
            size: inner.stats.size,
        }
    }

    /// Clear all cache
    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.data.clear();
        inner.stats.size = 0;
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new(Duration::from_millis(50), 1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module::ModuleData;
    use serde_json::json;

    #[test]
    fn test_cache_set_get() {
        let cache = Cache::new(Duration::from_secs(1), 100);
        let key = "test_module".to_string();
        let data = ModuleData {
            module: "test_module".to_string(),
            data: json!({"text": "test"}),
            cached: false,
        };

        cache.set(key.clone(), data.clone());
        let cached = cache.get(&key);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().module, "test_module");
    }

    #[test]
    fn test_cache_expiration() {
        let cache = Cache::new(Duration::from_millis(10), 100);
        let key = "test_module".to_string();
        let data = ModuleData {
            module: "test_module".to_string(),
            data: json!({"text": "test"}),
            cached: false,
        };

        cache.set(key.clone(), data);
        std::thread::sleep(Duration::from_millis(20));
        let cached = cache.get(&key);
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = Cache::new(Duration::from_secs(1), 100);
        let key = "test_module".to_string();
        let data = ModuleData {
            module: "test_module".to_string(),
            data: json!({"text": "test"}),
            cached: false,
        };

        cache.set(key.clone(), data);
        cache.invalidate(Some("test_module"));
        let cached = cache.get(&key);
        assert!(cached.is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = Cache::new(Duration::from_secs(1), 100);
        let key = "test_module".to_string();
        let data = ModuleData {
            module: "test_module".to_string(),
            data: json!({"text": "test"}),
            cached: false,
        };

        cache.set(key.clone(), data);
        let _ = cache.get(&key); // Hit
        let _ = cache.get("nonexistent"); // Miss
        
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.size, 1);
    }
}

