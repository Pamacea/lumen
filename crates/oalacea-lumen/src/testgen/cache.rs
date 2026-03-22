//! Persistent cache for test generation analysis
//!
//! Avoids re-analyzing unchanged files by caching:
//! - File hash → Analysis result
//! - Project metadata → Framework detection
//! - Function signatures → Test templates

use blake3::Hash;
use lumenx_core::LumenResult;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Cache entry for file analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    /// Blake3 hash of the file content
    pub file_hash: String,
    /// Cached analysis result
    pub analysis: CachedAnalysis,
    /// When this entry was created
    pub created_at: SystemTime,
    /// When this entry was last accessed
    pub accessed_at: SystemTime,
    /// Number of times this entry was used
    pub access_count: u64,
}

/// Cached analysis data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum CachedAnalysis {
    /// List of functions extracted from the file
    Functions(Vec<CachedFunction>),
    /// Framework detection result
    Framework(String),
    /// Test generation result
    TestGeneration(String),
}

/// Cached function information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CachedFunction {
    pub name: String,
    pub line: usize,
    pub signature: String,
    pub is_async: bool,
    pub visibility: String,
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum cache size in MB (default: 100)
    pub max_size_mb: usize,
    /// Cache entry TTL in seconds (default: 1 day)
    pub ttl_seconds: u64,
    /// Whether to use memory cache in addition to disk cache
    pub use_memory_cache: bool,
    /// Cache directory path
    pub cache_dir: PathBuf,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_size_mb: 100,
            ttl_seconds: 86400, // 1 day
            use_memory_cache: true,
            cache_dir: PathBuf::from(".lumen/cache"),
        }
    }
}

/// Test generation cache
pub struct TestGenCache {
    config: CacheConfig,
    /// In-memory cache for faster access
    memory_cache: Arc<RwLock<lru::LruCache<String, CacheEntry>>>,
    /// Directory for disk cache
    cache_dir: PathBuf,
}

impl TestGenCache {
    /// Create a new cache instance
    pub fn new(config: CacheConfig) -> LumenResult<Self> {
        // Create cache directory if it doesn't exist
        let cache_dir = config.cache_dir.clone();
        std::fs::create_dir_all(&cache_dir)?;

        // Initialize LRU cache (1000 entries in memory)
        let memory_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(1000).unwrap(),
        )));

        Ok(Self {
            config,
            memory_cache,
            cache_dir,
        })
    }

    /// Create cache with default configuration
    pub fn with_project_root(project_root: &Path) -> LumenResult<Self> {
        let cache_dir = project_root.join(".lumen").join("cache");
        let config = CacheConfig {
            cache_dir,
            ..Default::default()
        };
        Self::new(config)
    }

    /// Get cached analysis for a file
    pub fn get(&self, file_path: &Path) -> LumenResult<Option<CachedAnalysis>> {
        // First check memory cache
        let cache_key = self.cache_key(file_path)?;

        {
            let mut mem_cache = self.memory_cache.write()
                .map_err(|e| lumenx_core::LumenError::ConfigError(format!("Memory cache lock failed: {}", e)))?;

            if let Some(entry) = mem_cache.get_mut(&cache_key) {
                // Update access stats
                entry.accessed_at = SystemTime::now();
                entry.access_count += 1;
                return Ok(Some(entry.analysis.clone()));
            }
        }

        // Check disk cache
        let disk_path = self.cache_path(&cache_key);
        if disk_path.exists() {
            let data = std::fs::read(&disk_path)?;
            let entry: CacheEntry = bincode::deserialize(&data)
                .map_err(|e| lumenx_core::LumenError::ConfigError(format!("Deserialization failed: {}", e)))?;

            // Check if entry is still valid
            if let Ok(elapsed) = entry.created_at.elapsed() {
                if elapsed.as_secs() < self.config.ttl_seconds {
                    // Store in memory cache for faster access next time
                    let mut mem_cache = self.memory_cache.write()
                        .map_err(|e| lumenx_core::LumenError::ConfigError(format!("Memory cache lock failed: {}", e)))?;
                    mem_cache.put(cache_key.clone(), entry.clone());

                    return Ok(Some(entry.analysis));
                }
            }
        }

        Ok(None)
    }

    /// Store analysis result in cache
    pub fn put(&self, file_path: &Path, file_hash: Hash, analysis: CachedAnalysis) -> LumenResult<()> {
        let cache_key = self.cache_key(file_path)?;

        let entry = CacheEntry {
            file_hash: file_hash.to_hex().to_string(),
            analysis,
            created_at: SystemTime::now(),
            accessed_at: SystemTime::now(),
            access_count: 0,
        };

        // Store in memory cache
        {
            let mut mem_cache = self.memory_cache.write()
                .map_err(|e| lumenx_core::LumenError::ConfigError(format!("Memory cache lock failed: {}", e)))?;
            mem_cache.put(cache_key.clone(), entry.clone());
        }

        // Store on disk
        let disk_path = self.cache_path(&cache_key);
        let data = bincode::serialize(&entry)
            .map_err(|e| lumenx_core::LumenError::ConfigError(format!("Serialization failed: {}", e)))?;

        std::fs::write(&disk_path, data)?;

        Ok(())
    }

    /// Check if a file has changed since last analysis
    pub fn is_file_changed(&self, file_path: &Path, current_hash: Hash) -> LumenResult<bool> {
        if let Some(entry) = self.get(file_path)? {
            match entry {
                CachedAnalysis::Functions(_) => {
                    // Check if the cached hash matches
                    if let Ok(Some(cached_entry)) = self.get_entry(file_path) {
                        return Ok(cached_entry.file_hash != current_hash.to_hex().to_string());
                    }
                }
                _ => {}
            }
        }
        Ok(true) // No cache = file has changed
    }

    /// Get the full cache entry (including metadata)
    pub fn get_entry(&self, file_path: &Path) -> LumenResult<Option<CacheEntry>> {
        let cache_key = self.cache_key(file_path)?;
        let disk_path = self.cache_path(&cache_key);

        if disk_path.exists() {
            let data = std::fs::read(&disk_path)?;
            let entry: CacheEntry = bincode::deserialize(&data)
                .map_err(|e| lumenx_core::LumenError::ConfigError(format!("Deserialization failed: {}", e)))?;
            return Ok(Some(entry));
        }

        Ok(None)
    }

    /// Invalidate cache for a specific file
    pub fn invalidate(&self, file_path: &Path) -> LumenResult<()> {
        let cache_key = self.cache_key(file_path)?;

        // Remove from memory cache
        {
            let mut mem_cache = self.memory_cache.write()
                .map_err(|e| lumenx_core::LumenError::ConfigError(format!("Memory cache lock failed: {}", e)))?;
            mem_cache.pop(&cache_key);
        }

        // Remove from disk
        let disk_path = self.cache_path(&cache_key);
        if disk_path.exists() {
            std::fs::remove_file(&disk_path)?;
        }

        Ok(())
    }

    /// Clear all cache entries
    pub fn clear(&self) -> LumenResult<()> {
        // Clear memory cache
        {
            let mut mem_cache = self.memory_cache.write()
                .map_err(|e| lumenx_core::LumenError::ConfigError(format!("Memory cache lock failed: {}", e)))?;
            mem_cache.clear();
        }

        // Clear disk cache
        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                std::fs::remove_file(&path)?;
            }
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> LumenResult<CacheStats> {
        let mut total_entries = 0;
        let mut total_size = 0;
        let mut oldest_entry = None;
        let mut newest_entry = None;

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                total_entries += 1;
                total_size += entry.metadata()?.len();

                if let Ok(data) = std::fs::read(&path) {
                    if let Ok(cache_entry) = bincode::deserialize::<CacheEntry>(&data) {
                        if oldest_entry.is_none() || cache_entry.created_at < oldest_entry.unwrap() {
                            oldest_entry = Some(cache_entry.created_at);
                        }
                        if newest_entry.is_none() || cache_entry.created_at > newest_entry.unwrap() {
                            newest_entry = Some(cache_entry.created_at);
                        }
                    }
                }
            }
        }

        let memory_cache_size = {
            let mem_cache = self.memory_cache.read()
                .map_err(|e| lumenx_core::LumenError::ConfigError(format!("Memory cache lock failed: {}", e)))?;
            mem_cache.len()
        };

        Ok(CacheStats {
            total_entries,
            memory_entries: memory_cache_size,
            total_size_bytes: total_size,
            total_size_mb: total_size as f64 / (1024.0 * 1024.0),
            oldest_entry,
            newest_entry,
        })
    }

    /// Prune old cache entries to free up space
    pub fn prune(&self) -> LumenResult<usize> {
        let mut pruned = 0;
        let now = SystemTime::now();

        for entry in std::fs::read_dir(&self.cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Ok(data) = std::fs::read(&path) {
                    if let Ok(cache_entry) = bincode::deserialize::<CacheEntry>(&data) {
                        if let Ok(elapsed) = cache_entry.created_at.elapsed() {
                            if elapsed.as_secs() > self.config.ttl_seconds {
                                std::fs::remove_file(&path)?;
                                pruned += 1;
                            }
                        }
                    }
                }
            }
        }

        Ok(pruned)
    }

    /// Generate cache key for a file path
    fn cache_key(&self, file_path: &Path) -> LumenResult<String> {
        // Use relative path from cache dir as key
        let abs_path = std::fs::canonicalize(file_path)
            .unwrap_or_else(|_| file_path.to_path_buf());
        Ok(abs_path.to_string_lossy().to_string())
    }

    /// Get disk cache path for a cache key
    fn cache_path(&self, cache_key: &str) -> PathBuf {
        // Hash the key to get a valid filename
        let hash = blake3::hash(cache_key.as_bytes());
        self.cache_dir.join(format!("{}.bin", hash.to_hex().as_str()[..16].to_string()))
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub memory_entries: usize,
    pub total_size_bytes: u64,
    pub total_size_mb: f64,
    pub oldest_entry: Option<SystemTime>,
    pub newest_entry: Option<SystemTime>,
}

/// Compute file hash using BLAKE3
pub fn compute_file_hash(path: &Path) -> LumenResult<Hash> {
    let mut hasher = blake3::Hasher::new();
    let content = std::fs::read(path)?;
    hasher.update(&content);
    Ok(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_put_get() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let cache = TestGenCache::new(config).unwrap();

        let test_file = temp_dir.path().join("test.rs");

        // Put cache entry
        let hash = blake3::hash(b"test content");
        let analysis = CachedAnalysis::Framework("NextJs".to_string());
        cache.put(&test_file, hash, analysis).unwrap();

        // Get cache entry
        let result = cache.get(&test_file).unwrap();
        assert!(result.is_some());

        if let Some(CachedAnalysis::Framework(fw)) = result {
            assert_eq!(fw, "NextJs");
        } else {
            panic!("Expected framework analysis");
        }
    }

    #[test]
    fn test_cache_invalidation() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let cache = TestGenCache::new(config).unwrap();

        let test_file = temp_dir.path().join("test.rs");
        let hash = blake3::hash(b"test content");
        let analysis = CachedAnalysis::Framework("NextJs".to_string());

        cache.put(&test_file, hash, analysis).unwrap();
        assert!(cache.get(&test_file).unwrap().is_some());

        cache.invalidate(&test_file).unwrap();
        assert!(cache.get(&test_file).unwrap().is_none());
    }

    #[test]
    fn test_file_hash_computation() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        std::fs::write(&test_file, b"test content").unwrap();

        let hash1 = compute_file_hash(&test_file).unwrap();
        let hash2 = compute_file_hash(&test_file).unwrap();

        assert_eq!(hash1, hash2);

        // Modify file
        std::fs::write(&test_file, b"modified content").unwrap();
        let hash3 = compute_file_hash(&test_file).unwrap();

        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let cache = TestGenCache::new(config).unwrap();

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 0);

        // Add some entries
        for i in 0..5 {
            let test_file = temp_dir.path().join(format!("test{}.rs", i));
            let hash = blake3::hash(format!("content{}", i).as_bytes());
            let analysis = CachedAnalysis::Framework(format!("Framework{}", i));
            cache.put(&test_file, hash, analysis).unwrap();
        }

        let stats = cache.stats().unwrap();
        assert_eq!(stats.total_entries, 5);
    }

    #[test]
    fn test_is_file_changed() {
        let temp_dir = TempDir::new().unwrap();
        let config = CacheConfig {
            cache_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let cache = TestGenCache::new(config).unwrap();

        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, b"original content").unwrap();

        let hash = compute_file_hash(&test_file).unwrap();
        let analysis = CachedAnalysis::Framework("NextJs".to_string());
        cache.put(&test_file, hash, analysis).unwrap();

        // File hasn't changed
        let current_hash = compute_file_hash(&test_file).unwrap();
        assert!(!cache.is_file_changed(&test_file, current_hash).unwrap());

        // File changed
        std::fs::write(&test_file, b"modified content").unwrap();
        let new_hash = compute_file_hash(&test_file).unwrap();
        assert!(cache.is_file_changed(&test_file, new_hash).unwrap());
    }
}
