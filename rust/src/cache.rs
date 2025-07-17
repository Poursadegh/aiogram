use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use dashmap::DashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry<T> {
    pub value: T,
    pub created_at: Instant,
    pub accessed_at: Instant,
    pub access_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: usize,
    pub max_size: usize,
    pub hit_rate: f64,
}

pub struct Cache<T> {
    data: DashMap<String, CacheEntry<T>>,
    max_size: usize,
    ttl_seconds: Option<u64>,
    stats: Arc<Mutex<CacheStats>>,
}

impl<T> Cache<T> 
where 
    T: Clone + Send + Sync + 'static
{
    pub fn new(max_size: usize, ttl_seconds: Option<u64>) -> Self {
        Self {
            data: DashMap::new(),
            max_size,
            ttl_seconds,
            stats: Arc::new(Mutex::new(CacheStats {
                hits: 0,
                misses: 0,
                evictions: 0,
                size: 0,
                max_size,
                hit_rate: 0.0,
            })),
        }
    }
    
    pub fn get(&self, key: &str) -> Option<T> {
        if let Some(entry) = self.data.get(key) {
            // Check if entry has expired
            if let Some(ttl) = self.ttl_seconds {
                if entry.created_at.elapsed().as_secs() > ttl {
                    self.data.remove(key);
                    self.update_stats(false);
                    return None;
                }
            }
            
            // Update access statistics
            let mut entry = entry.clone();
            entry.accessed_at = Instant::now();
            entry.access_count += 1;
            self.data.insert(key.to_string(), entry);
            
            self.update_stats(true);
            Some(entry.value)
        } else {
            self.update_stats(false);
            None
        }
    }
    
    pub fn set(&self, key: &str, value: T) {
        // Check if we need to evict entries
        if self.data.len() >= self.max_size {
            self.evict_lru();
        }
        
        let entry = CacheEntry {
            value,
            created_at: Instant::now(),
            accessed_at: Instant::now(),
            access_count: 1,
        };
        
        self.data.insert(key.to_string(), entry);
        self.update_stats(false);
    }
    
    pub fn remove(&self, key: &str) -> Option<T> {
        if let Some((_, entry)) = self.data.remove(key) {
            Some(entry.value)
        } else {
            None
        }
    }
    
    pub fn clear(&self) {
        self.data.clear();
        self.update_stats(false);
    }
    
    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }
    
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    fn evict_lru(&self) {
        let mut entries: Vec<(String, CacheEntry<T>)> = self.data
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect();
        
        // Sort by access time and count (LRU)
        entries.sort_by(|a, b| {
            let a_score = a.1.access_count as f64 / a.1.accessed_at.elapsed().as_secs().max(1) as f64;
            let b_score = b.1.access_count as f64 / b.1.accessed_at.elapsed().as_secs().max(1) as f64;
            a_score.partial_cmp(&b_score).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Remove the least recently used entry
        if let Some((key, _)) = entries.first() {
            self.data.remove(key);
            if let Ok(mut stats) = self.stats.lock() {
                stats.evictions += 1;
            }
        }
    }
    
    fn update_stats(&self, hit: bool) {
        if let Ok(mut stats) = self.stats.lock() {
            if hit {
                stats.hits += 1;
            } else {
                stats.misses += 1;
            }
            
            let total = stats.hits + stats.misses;
            stats.hit_rate = if total > 0 {
                stats.hits as f64 / total as f64
            } else {
                0.0
            };
            
            stats.size = self.data.len();
        }
    }
    
    pub fn get_stats(&self) -> CacheStats {
        if let Ok(stats) = self.stats.lock() {
            stats.clone()
        } else {
            CacheStats {
                hits: 0,
                misses: 0,
                evictions: 0,
                size: 0,
                max_size: self.max_size,
                hit_rate: 0.0,
            }
        }
    }
    
    pub fn cleanup_expired(&self) -> usize {
        let mut removed_count = 0;
        
        if let Some(ttl) = self.ttl_seconds {
            let now = Instant::now();
            let expired_keys: Vec<String> = self.data
                .iter()
                .filter(|entry| entry.created_at.elapsed().as_secs() > ttl)
                .map(|entry| entry.key().clone())
                .collect();
            
            for key in expired_keys {
                if self.data.remove(&key).is_some() {
                    removed_count += 1;
                }
            }
        }
        
        removed_count
    }
}

// Global cache instances
lazy_static! {
    static ref TEXT_CACHE: Arc<Cache<String>> = Arc::new(Cache::new(1000, Some(3600)));
    static ref DATA_CACHE: Arc<Cache<Vec<f64>>> = Arc::new(Cache::new(500, Some(1800)));
    static ref RESULT_CACHE: Arc<Cache<String>> = Arc::new(Cache::new(2000, Some(7200)));
}

// Public cache functions
pub fn get_cached_text(key: &str) -> Option<String> {
    TEXT_CACHE.get(key)
}

pub fn set_cached_text(key: &str, value: String) {
    TEXT_CACHE.set(key, value);
}

pub fn get_cached_data(key: &str) -> Option<Vec<f64>> {
    DATA_CACHE.get(key)
}

pub fn set_cached_data(key: &str, value: Vec<f64>) {
    DATA_CACHE.set(key, value);
}

pub fn get_cached_result(key: &str) -> Option<String> {
    RESULT_CACHE.get(key)
}

pub fn set_cached_result(key: &str, value: String) {
    RESULT_CACHE.set(key, value);
}

pub fn clear_all_caches() {
    TEXT_CACHE.clear();
    DATA_CACHE.clear();
    RESULT_CACHE.clear();
}

pub fn get_cache_stats() -> HashMap<String, CacheStats> {
    let mut stats = HashMap::new();
    stats.insert("text_cache".to_string(), TEXT_CACHE.get_stats());
    stats.insert("data_cache".to_string(), DATA_CACHE.get_stats());
    stats.insert("result_cache".to_string(), RESULT_CACHE.get_stats());
    stats
}

pub fn cleanup_all_caches() -> HashMap<String, usize> {
    let mut cleanup_stats = HashMap::new();
    cleanup_stats.insert("text_cache".to_string(), TEXT_CACHE.cleanup_expired());
    cleanup_stats.insert("data_cache".to_string(), DATA_CACHE.cleanup_expired());
    cleanup_stats.insert("result_cache".to_string(), RESULT_CACHE.cleanup_expired());
    cleanup_stats
}

// Cache key generation utilities
pub fn generate_text_cache_key(text: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("text_{:x}", hasher.finalize())
}

pub fn generate_data_cache_key(data: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("data_{:x}", hasher.finalize())
}

pub fn generate_result_cache_key(operation: &str, input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(operation.as_bytes());
    hasher.update(input.as_bytes());
    format!("result_{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_basic_operations() {
        let cache = Cache::new(10, Some(60));
        
        cache.set("key1", "value1".to_string());
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        assert_eq!(cache.get("key2"), None);
        
        cache.remove("key1");
        assert_eq!(cache.get("key1"), None);
    }
    
    #[test]
    fn test_cache_eviction() {
        let cache = Cache::new(2, None);
        
        cache.set("key1", "value1".to_string());
        cache.set("key2", "value2".to_string());
        cache.set("key3", "value3".to_string());
        
        // Should have evicted one entry
        assert_eq!(cache.size(), 2);
    }
    
    #[test]
    fn test_cache_stats() {
        let cache = Cache::new(10, None);
        
        cache.set("key1", "value1".to_string());
        cache.get("key1");
        cache.get("nonexistent");
        
        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!(stats.hit_rate > 0.0);
    }
    
    #[test]
    fn test_global_caches() {
        set_cached_text("test_key", "test_value".to_string());
        assert_eq!(get_cached_text("test_key"), Some("test_value".to_string()));
        
        set_cached_data("test_data", vec![1.0, 2.0, 3.0]);
        assert_eq!(get_cached_data("test_data"), Some(vec![1.0, 2.0, 3.0]));
    }
    
    #[test]
    fn test_cache_key_generation() {
        let key1 = generate_text_cache_key("test text");
        let key2 = generate_text_cache_key("test text");
        let key3 = generate_text_cache_key("different text");
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
} 