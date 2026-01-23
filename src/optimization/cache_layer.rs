//! High-Performance Caching Layer
//!
//! In-memory and Redis caching for hot paths
//! Target: 90%+ cache hit rate, <1ms cache access

use std::sync::Arc;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use dashmap::DashMap;

/// Cache entry with TTL and access tracking
#[derive(Debug, Clone)]
struct CacheEntry<T> {
    value: T,
    created_at: Instant,
    last_accessed: Instant,
    access_count: u64,
    ttl: Duration,
}

/// High-performance in-memory cache
pub struct InMemoryCache<K, V> 
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    store: Arc<DashMap<K, CacheEntry<V>>>,
    max_size: usize,
    default_ttl: Duration,
}

impl<K, V> InMemoryCache<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(max_size: usize, ttl_secs: u64) -> Self {
        let cache = Self {
            store: Arc::new(DashMap::new()),
            max_size,
            default_ttl: Duration::from_secs(ttl_secs),
        };
        
        // Start TTL cleanup task
        let store = cache.store.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(60)).await;
                
                // Remove expired entries
                let now = Instant::now();
                store.retain(|_, entry| {
                    now.duration_since(entry.created_at) < entry.ttl
                });
            }
        });
        
        cache
    }
    
    /// Get value from cache
    pub fn get(&self, key: &K) -> Option<V> {
        let mut entry = self.store.get_mut(key)?;
        
        // Check TTL
        let now = Instant::now();
        if now.duration_since(entry.created_at) > entry.ttl {
            drop(entry);
            self.store.remove(key);
            return None;
        }
        
        // Update access stats
        entry.last_accessed = now;
        entry.access_count += 1;
        
        Some(entry.value.clone())
    }
    
    /// Set value in cache
    pub fn set(&self, key: K, value: V) {
        self.set_with_ttl(key, value, self.default_ttl);
    }
    
    /// Set value with custom TTL
    pub fn set_with_ttl(&self, key: K, value: V, ttl: Duration) {
        // Evict if at capacity
        if self.store.len() >= self.max_size {
            self.evict_lru();
        }
        
        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 0,
            ttl,
        };
        
        self.store.insert(key, entry);
    }
    
    /// Evict least recently used entry
    fn evict_lru(&self) {
        if let Some(lru_key) = self.find_lru_key() {
            self.store.remove(&lru_key);
        }
    }
    
    /// Find LRU key
    fn find_lru_key(&self) -> Option<K> {
        let mut oldest_key = None;
        let mut oldest_time = Instant::now();
        
        for entry in self.store.iter() {
            if entry.last_accessed < oldest_time {
                oldest_time = entry.last_accessed;
                oldest_key = Some(entry.key().clone());
            }
        }
        
        oldest_key
    }
    
    /// Clear cache
    pub fn clear(&self) {
        self.store.clear();
    }
    
    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let mut total_accesses = 0u64;
        let mut total_entries = 0usize;
        let mut expired_count = 0usize;
        
        let now = Instant::now();
        
        for entry in self.store.iter() {
            total_entries += 1;
            total_accesses += entry.access_count;
            
            if now.duration_since(entry.created_at) > entry.ttl {
                expired_count += 1;
            }
        }
        
        CacheStats {
            total_entries,
            total_accesses,
            expired_count,
            capacity: self.max_size,
            utilization: (total_entries as f64 / self.max_size as f64) * 100.0,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_accesses: u64,
    pub expired_count: usize,
    pub capacity: usize,
    pub utilization: f64,
}

/// Multi-tier cache with L1 (in-memory) and L2 (Redis)
pub struct MultiTierCache {
    l1_cache: Arc<InMemoryCache<String, Vec<u8>>>,
    l2_enabled: bool,
    // Redis client would go here
}

impl MultiTierCache {
    pub fn new(l1_size: usize, l1_ttl_secs: u64, enable_redis: bool) -> Self {
        Self {
            l1_cache: Arc::new(InMemoryCache::new(l1_size, l1_ttl_secs)),
            l2_enabled: enable_redis,
        }
    }
    
    /// Get with fallback to L2
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        // Try L1 first
        if let Some(value) = self.l1_cache.get(&key.to_string()) {
            return Some(value);
        }
        
        // Try L2 (Redis) if enabled
        if self.l2_enabled {
            // Would query Redis here
            // If found, populate L1
        }
        
        None
    }
    
    /// Set in both tiers
    pub async fn set(&self, key: String, value: Vec<u8>) {
        // Set in L1
        self.l1_cache.set(key.clone(), value.clone());
        
        // Set in L2 if enabled
        if self.l2_enabled {
            // Would set in Redis here
        }
    }
}

/// Specialized cache for vector embeddings
pub struct EmbeddingCache {
    cache: Arc<InMemoryCache<String, ndarray::Array1<f32>>>,
}

impl EmbeddingCache {
    pub fn new(max_embeddings: usize) -> Self {
        Self {
            cache: Arc::new(InMemoryCache::new(max_embeddings, 300)),  // 5 min TTL
        }
    }
    
    /// Cache embedding with text key
    pub fn cache_embedding(&self, text: &str, embedding: ndarray::Array1<f32>) {
        let key = format!("{:x}", md5::compute(text));
        self.cache.set(key, embedding);
    }
    
    /// Get cached embedding
    pub fn get_embedding(&self, text: &str) -> Option<ndarray::Array1<f32>> {
        let key = format!("{:x}", md5::compute(text));
        self.cache.get(&key)
    }
    
    /// Batch get embeddings
    pub fn get_batch(&self, texts: &[String]) -> Vec<Option<ndarray::Array1<f32>>> {
        texts.iter()
            .map(|text| self.get_embedding(text))
            .collect()
    }
}

/// Query result cache
pub struct QueryCache {
    cache: Arc<InMemoryCache<String, serde_json::Value>>,
}

impl QueryCache {
    pub fn new(max_queries: usize, ttl_secs: u64) -> Self {
        Self {
            cache: Arc::new(InMemoryCache::new(max_queries, ttl_secs)),
        }
    }
    
    /// Cache query result
    pub fn cache_query(&self, sql: &str, params: &[&str], result: serde_json::Value) {
        let key = format!("{}:{}", sql, params.join(","));
        self.cache.set(key, result);
    }
    
    /// Get cached query result
    pub fn get_query(&self, sql: &str, params: &[&str]) -> Option<serde_json::Value> {
        let key = format!("{}:{}", sql, params.join(","));
        self.cache.get(&key)
    }
}

/// Sacred geometry cache for flux positions
pub struct FluxPositionCache {
    cache: Arc<DashMap<u64, u8>>,  // seed -> position
}

impl FluxPositionCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }
    
    /// Cache flux position
    pub fn cache_position(&self, seed: u64, position: u8) {
        self.cache.insert(seed, position);
    }
    
    /// Get cached position
    pub fn get_position(&self, seed: u64) -> Option<u8> {
        self.cache.get(&seed).map(|entry| *entry.value())
    }
    
    /// Precompute common positions
    pub fn precompute_common(&self) {
        // Cache sacred positions
        for i in 0..1000 {
            let seed = i * 3;
            self.cache_position(seed, 3);
            self.cache_position(seed * 2, 6);
            self.cache_position(seed * 3, 9);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_in_memory_cache() {
        let cache: InMemoryCache<String, String> = InMemoryCache::new(100, 60);
        
        cache.set("key1".to_string(), "value1".to_string());
        assert_eq!(cache.get(&"key1".to_string()), Some("value1".to_string()));
        
        // Test TTL
        cache.set_with_ttl(
            "key2".to_string(),
            "value2".to_string(),
            Duration::from_millis(100)
        );
        
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert_eq!(cache.get(&"key2".to_string()), None);
    }
    
    #[test]
    fn test_lru_eviction() {
        let cache: InMemoryCache<i32, i32> = InMemoryCache::new(3, 60);
        
        cache.set(1, 1);
        cache.set(2, 2);
        cache.set(3, 3);
        
        // Access 1 and 2 to make them more recent
        cache.get(&1);
        cache.get(&2);
        
        // Add 4th element, should evict 3
        cache.set(4, 4);
        
        assert!(cache.get(&1).is_some());
        assert!(cache.get(&2).is_some());
        assert!(cache.get(&4).is_some());
        assert!(cache.get(&3).is_none());  // Evicted
    }
    
    #[test]
    fn test_flux_position_cache() {
        let cache = FluxPositionCache::new();
        cache.precompute_common();
        
        assert_eq!(cache.get_position(0), Some(3));
        assert_eq!(cache.get_position(3), Some(3));
        assert_eq!(cache.get_position(6), Some(6));
        assert_eq!(cache.get_position(9), Some(9));
    }
}
