//! Parallel file analysis with caching support
//!
//! This module provides high-performance analysis using:
//! - Rayon for parallel processing
//! - File-level caching to avoid re-analysis
//! - Incremental updates based on file changes

use crate::cache::{TestGenCache, compute_file_hash, CachedAnalysis, CachedFunction};
use crate::code_parser;
use crate::{FunctionInfo, TestFramework};
use lumenx_core::{Language, LumenResult, ProjectInfo};
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::{debug, info, instrument};

/// Configuration for parallel analysis
#[derive(Debug, Clone)]
pub struct ParallelAnalysisConfig {
    /// Number of threads to use (None = use CPU count)
    pub num_threads: Option<usize>,
    /// Whether to use cache
    pub use_cache: bool,
    /// Batch size for processing
    pub batch_size: usize,
}

impl Default for ParallelAnalysisConfig {
    fn default() -> Self {
        Self {
            num_threads: None, // Use rayon's default
            use_cache: true,
            batch_size: 50,
        }
    }
}

/// Result of parallel analysis
#[derive(Debug)]
pub struct ParallelAnalysisResult {
    /// All functions found (from cache + newly analyzed)
    pub functions: Vec<FunctionInfo>,
    /// Number of files analyzed from cache
    pub cache_hits: usize,
    /// Number of files newly analyzed
    pub cache_misses: usize,
    /// Time taken for analysis
    pub analysis_time_ms: u64,
}

/// Analyzed file with metadata
#[derive(Debug)]
struct AnalyzedFile {
    path: PathBuf,
    functions: Vec<FunctionInfo>,
    from_cache: bool,
}

/// Parallel analyzer with caching
pub struct ParallelAnalyzer {
    project_root: PathBuf,
    framework: TestFramework,
    cache: Option<TestGenCache>,
    config: ParallelAnalysisConfig,
}

impl ParallelAnalyzer {
    /// Create a new parallel analyzer
    pub fn new(
        project_root: PathBuf,
        framework: TestFramework,
        config: ParallelAnalysisConfig,
    ) -> LumenResult<Self> {
        let cache = if config.use_cache {
            Some(TestGenCache::with_project_root(&project_root)?)
        } else {
            None
        };

        Ok(Self {
            project_root,
            framework,
            cache,
            config,
        })
    }

    /// Create with default configuration
    pub fn with_defaults(project_root: PathBuf, framework: TestFramework) -> LumenResult<Self> {
        Self::new(project_root, framework, ParallelAnalysisConfig::default())
    }

    /// Analyze all source files in parallel
    #[instrument(skip(self, info))]
    pub fn analyze_project(&self, info: &ProjectInfo) -> LumenResult<ParallelAnalysisResult> {
        let start = Instant::now();

        info!("Starting parallel analysis for: {}", info.name);

        // Collect source files
        let source_files = self.collect_source_files(info)?;
        info!("Found {} source files to analyze", source_files.len());

        // Set up thread pool if specified
        if let Some(num_threads) = self.config.num_threads {
            info!("Using {} threads for analysis", num_threads);
            rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build_global()
                .map_err(|e| lumenx_core::LumenError::ConfigError(format!("Failed to create thread pool: {}", e)))?;
        }

        // Track cache statistics
        let cache_hits = Arc::new(Mutex::new(0usize));
        let cache_misses = Arc::new(Mutex::new(0usize));

        // Process files in batches with parallel execution
        let analyzed_files: Vec<AnalyzedFile> = source_files
            .par_chunks(self.config.batch_size)
            .flat_map(|batch| {
                batch.iter().filter_map(|file_path| {
                    match self.analyze_file_cached(file_path, &cache_hits, &cache_misses) {
                        Ok(Some(result)) => Some(result),
                        Ok(None) => None,
                        Err(e) => {
                            debug!("Failed to analyze file {:?}: {}", file_path, e);
                            None
                        }
                    }
                }).collect::<Vec<_>>()
            })
            .collect();

        // Collect all functions
        let functions: Vec<FunctionInfo> = analyzed_files
            .into_iter()
            .flat_map(|af| af.functions)
            .collect();

        let elapsed = start.elapsed();
        let analysis_time_ms = elapsed.as_millis() as u64;

        let cache_hits_count = *cache_hits.lock().map_err(|e| {
            lumenx_core::LumenError::InternalError(format!("Cache hits lock poisoned: {}", e))
        })?;
        let cache_misses_count = *cache_misses.lock().map_err(|e| {
            lumenx_core::LumenError::InternalError(format!("Cache misses lock poisoned: {}", e))
        })?;

        info!(
            "Analysis complete: {} functions found in {}ms",
            functions.len(),
            analysis_time_ms
        );
        info!(
            "Cache stats: {} hits, {} misses",
            cache_hits_count,
            cache_misses_count
        );

        Ok(ParallelAnalysisResult {
            functions,
            cache_hits: cache_hits_count,
            cache_misses: cache_misses_count,
            analysis_time_ms,
        })
    }

    /// Analyze a single file with caching
    fn analyze_file_cached(
        &self,
        file_path: &Path,
        cache_hits: &Arc<Mutex<usize>>,
        cache_misses: &Arc<Mutex<usize>>,
    ) -> LumenResult<Option<AnalyzedFile>> {
        // Try cache first if enabled
        if let Some(ref cache) = self.cache {
            let file_hash = compute_file_hash(file_path)?;

            // Check if we have cached analysis
            if let Some(cached) = cache.get(file_path)? {
                *cache_hits.lock().map_err(|e| {
                    lumenx_core::LumenError::InternalError(format!("Cache hits lock poisoned: {}", e))
                })? += 1;

                // Convert cached analysis to FunctionInfo
                if let CachedAnalysis::Functions(cached_functions) = cached {
                    let functions: Vec<FunctionInfo> = cached_functions
                        .into_iter()
                        .map(|cf| self.convert_cached_function(cf, file_path))
                        .collect();

                    return Ok(Some(AnalyzedFile {
                        path: file_path.to_path_buf(),
                        functions,
                        from_cache: true,
                    }));
                }
            }

            // Not in cache or file changed - analyze and cache
            *cache_misses.lock().map_err(|e| {
                lumenx_core::LumenError::InternalError(format!("Cache misses lock poisoned: {}", e))
            })? += 1;

            let functions = code_parser::parse_file(file_path, self.framework)?;

            if !functions.is_empty() {
                // Cache the result
                let cached_functions: Vec<CachedFunction> = functions
                    .iter()
                    .map(|f| CachedFunction {
                        name: f.name.clone(),
                        line: f.line,
                        signature: f.signature.clone(),
                        is_async: f.is_async,
                        visibility: format!("{:?}", f.visibility),
                    })
                    .collect();

                cache.put(
                    file_path,
                    file_hash,
                    CachedAnalysis::Functions(cached_functions),
                )?;
            }

            Ok(Some(AnalyzedFile {
                path: file_path.to_path_buf(),
                functions,
                from_cache: false,
            }))
        } else {
            // No cache - just analyze
            *cache_misses.lock().unwrap() += 1;
            let functions = code_parser::parse_file(file_path, self.framework)?;

            Ok(if functions.is_empty() {
                None
            } else {
                Some(AnalyzedFile {
                    path: file_path.to_path_buf(),
                    functions,
                    from_cache: false,
                })
            })
        }
    }

    /// Convert cached function back to FunctionInfo
    fn convert_cached_function(&self, cf: CachedFunction, file_path: &Path) -> FunctionInfo {
        FunctionInfo {
            name: cf.name,
            file_path: file_path.to_path_buf(),
            line: cf.line,
            signature: cf.signature,
            parameters: vec![], // Parameters not cached to save space
            return_type: None,
            is_async: cf.is_async,
            visibility: match cf.visibility.as_str() {
                "Public" => crate::Visibility::Public,
                "Private" => crate::Visibility::Private,
                "Protected" => crate::Visibility::Protected,
                "Internal" => crate::Visibility::Internal,
                _ => crate::Visibility::Private,
            },
            language: self.detect_language(file_path),
            doc_comment: None,
            suggested_tests: vec![],
        }
    }

    /// Detect language from file extension
    fn detect_language(&self, path: &Path) -> Language {
        match path.extension().and_then(|e| e.to_str()) {
            Some("ts") | Some("tsx") | Some("mts") => Language::TypeScript,
            Some("js") | Some("jsx") | Some("mjs") => Language::JavaScript,
            Some("rs") => Language::Rust,
            Some("py") => Language::Python,
            Some("go") => Language::Go,
            Some("java") => Language::Java,
            Some("rb") => Language::Unknown,
            Some("php") => Language::Unknown,
            _ => Language::TypeScript, // Default
        }
    }

    /// Collect all source files from the project
    fn collect_source_files(&self, info: &ProjectInfo) -> LumenResult<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let mut source_files = Vec::new();

        // File extensions to analyze based on language
        let extensions: Vec<&str> = match info.language {
            Language::TypeScript | Language::JavaScript => {
                vec!["ts", "tsx", "mts", "js", "jsx", "mjs"]
            }
            Language::Rust => vec!["rs"],
            Language::Python => vec!["py"],
            Language::Go => vec!["go"],
            Language::Java => vec!["java"],
            _ => vec!["ts", "tsx", "js", "jsx", "rs", "py"],
        };

        // Walk the project directory
        let walker = WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| {
                // Skip common directories to ignore
                e.file_name()
                    .to_str()
                    .map(|s| {
                        ![
                            "node_modules",
                            "target",
                            "dist",
                            "build",
                            ".git",
                            "vendor",
                            ".venv",
                            "venv",
                            "__pycache__",
                            ".next",
                            ".nuxt",
                        ]
                        .contains(&s)
                    })
                    .unwrap_or(true)
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| extensions.contains(&ext))
                    .unwrap_or(false)
            });

        for entry in walker {
            source_files.push(entry.path().to_path_buf());
        }

        Ok(source_files)
    }

    /// Invalidate cache for a specific file
    pub fn invalidate_file(&self, file_path: &Path) -> LumenResult<()> {
        if let Some(ref cache) = self.cache {
            cache.invalidate(file_path)?;
        }
        Ok(())
    }

    /// Clear all cache
    pub fn clear_cache(&self) -> LumenResult<()> {
        if let Some(ref cache) = self.cache {
            cache.clear()?;
        }
        Ok(())
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> LumenResult<Option<crate::cache::CacheStats>> {
        if let Some(ref cache) = self.cache {
            Ok(Some(cache.stats()?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_parallel_analyzer_creation() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = ParallelAnalyzer::with_defaults(
            temp_dir.path().to_path_buf(),
            TestFramework::Vitest,
        )
        .unwrap();

        assert!(analyzer.cache.is_some());
    }

    #[test]
    fn test_collect_source_files() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        fs::write(temp_dir.path().join("test.ts"), "export function test() {}").unwrap();
        fs::write(temp_dir.path().join("test.js"), "function test() {}").unwrap();
        fs::write(
            temp_dir.path().join("node_modules"),
            "should be ignored",
        )
        .unwrap();

        let info = ProjectInfo {
            name: "test".to_string(),
            root: temp_dir.path().to_path_buf(),
            framework: lumenx_core::Framework::NextJs,
            language: Language::TypeScript,
            test_runner: lumenx_core::TestRunner::Vitest,
            package_manager: Some("npm".to_string()),
            dependencies: vec![],
            dev_dependencies: vec![],
            database: None,
            package_json: None,
            cargo_dependencies: None,
        };

        let analyzer = ParallelAnalyzer::with_defaults(
            temp_dir.path().to_path_buf(),
            TestFramework::Vitest,
        )
        .unwrap();

        let files = analyzer.collect_source_files(&info).unwrap();

        // Should find ts and js files but not node_modules
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_language_detection() {
        let temp_dir = TempDir::new().unwrap();
        let analyzer = ParallelAnalyzer::with_defaults(
            temp_dir.path().to_path_buf(),
            TestFramework::Vitest,
        )
        .unwrap();

        assert_eq!(
            analyzer.detect_language(Path::new("test.ts")),
            Language::TypeScript
        );
        assert_eq!(
            analyzer.detect_language(Path::new("test.js")),
            Language::JavaScript
        );
        assert_eq!(
            analyzer.detect_language(Path::new("test.rs")),
            Language::Rust
        );
        assert_eq!(
            analyzer.detect_language(Path::new("test.py")),
            Language::Python
        );
    }
}
