//! Performance analyzer
//!
//! Analyzes code patterns to detect potential performance issues including:
//! - Backend latency patterns (API response times estimation)
//! - Database query patterns (N+1 queries, missing indexes)
//! - Caching strategies detection
//! - Async/await usage analysis
//! - Bundle size estimation

use crate::analyze::parsers::{html::HtmlParser, rust::RustAnalyzer, typescript::TypeScriptAnalyzer};
use oalacea_lumen_core::{LumenResult, Project, ProjectInfo, Framework, Language};
use oalacea_lumen_core::scoring::{IssueSeverity, MetricValue, ScoreIssue};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

/// Performance analyzer
pub struct PerformanceAnalyzer {
    project_root: String,
}

impl PerformanceAnalyzer {
    /// Create a new performance analyzer
    pub fn new(project_root: String) -> Self {
        Self { project_root }
    }

    /// Analyze the project for performance issues
    pub fn analyze(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Framework-specific analysis
        match info.framework {
            Framework::NextJs | Framework::Remix | Framework::Nuxt | Framework::Astro => {
                let (framework_metrics, framework_issues) = self.analyze_frontend_framework(info)?;
                metrics.extend(framework_metrics);
                issues.extend(framework_issues);
            }
            Framework::Express | Framework::Fastify | Framework::NestJS => {
                let (backend_metrics, backend_issues) = self.analyze_nodejs_backend(info)?;
                metrics.extend(backend_metrics);
                issues.extend(backend_issues);
            }
            Framework::Axum | Framework::ActixWeb | Framework::Rocket => {
                let (rust_metrics, rust_issues) = self.analyze_rust_backend(info)?;
                metrics.extend(rust_metrics);
                issues.extend(rust_issues);
            }
            _ => {
                // Generic analysis
                let (generic_metrics, generic_issues) = self.analyze_generic(info)?;
                metrics.extend(generic_metrics);
                issues.extend(generic_issues);
            }
        }

        // Cross-cutting performance checks
        let (bundle_metrics, bundle_issues) = self.analyze_bundle_size(info)?;
        metrics.extend(bundle_metrics);
        issues.extend(bundle_issues);

        let (async_metrics, async_issues) = self.analyze_async_usage(info)?;
        metrics.extend(async_metrics);
        issues.extend(async_issues);

        // Calculate overall performance score
        let base_score = 100.0;
        let penalty: f64 = issues.iter().map(|i| match i.severity {
            IssueSeverity::Critical => 20.0,
            IssueSeverity::High => 10.0,
            IssueSeverity::Medium => 5.0,
            IssueSeverity::Low => 2.0,
            IssueSeverity::Info => 0.0,
        }).sum();

        metrics.insert(
            "performance:overall_score".to_string(),
            MetricValue::Percentage((base_score - penalty).max(0.0)),
        );

        Ok((metrics, issues))
    }

    /// Analyze frontend frameworks (Next.js, Remix, etc.)
    fn analyze_frontend_framework(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Check for server components usage (Next.js 13+)
        let app_dir = Path::new(&self.project_root).join("app");
        let pages_dir = Path::new(&self.project_root).join("pages");

        let has_app_dir = app_dir.exists();
        let has_pages_dir = pages_dir.exists();

        metrics.insert(
            "performance:uses_app_router".to_string(),
            MetricValue::Boolean(has_app_dir),
        );

        // Check for dynamic imports
        let dynamic_import_count = self.count_pattern_in_files(info, r"import\(|lazy\(")?;
        metrics.insert(
            "performance:dynamic_imports".to_string(),
            MetricValue::Count(dynamic_import_count),
        );

        if dynamic_import_count == 0 && has_app_dir {
            issues.push(ScoreIssue {
                id: "perf-001".to_string(),
                severity: IssueSeverity::Medium,
                category: "performance".to_string(),
                title: "No code splitting detected".to_string(),
                description: "Dynamic imports help reduce initial bundle size by splitting code into chunks loaded on demand.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 5.0,
                suggestion: Some("Use dynamic imports for route segments, heavy components, or non-critical features:\n\n```typescript\n// Instead of:\nimport HeavyComponent from './HeavyComponent'\n\n// Use:\nconst HeavyComponent = dynamic(() => import('./HeavyComponent'))\n```".to_string()),
            });
        }

        // Check for image optimization
        let uses_next_image = self.count_pattern_in_files(info, r#"from ['"]next/image['"]"#)?;
        let has_unoptimized_img = self.count_pattern_in_files(info, r"<img\s")?;

        metrics.insert(
            "performance:uses_optimized_images".to_string(),
            MetricValue::Boolean(uses_next_image > 0),
        );
        metrics.insert(
            "performance:unoptimized_img_tags".to_string(),
            MetricValue::Count(has_unoptimized_img),
        );

        if has_unoptimized_img > 0 && uses_next_image > 0 {
            issues.push(ScoreIssue {
                id: "perf-002".to_string(),
                severity: IssueSeverity::Medium,
                category: "performance".to_string(),
                title: "Unoptimized img tags detected".to_string(),
                description: format!("Found {} unoptimized <img> tags. Next.js Image component provides automatic optimization.", has_unoptimized_img),
                file: None,
                line: None,
                column: None,
                impact: 3.0 * has_unoptimized_img as f64,
                suggestion: Some("Replace <img> tags with Next.js Image component:\n\n```tsx\nimport Image from 'next/image'\n\nexport default function Page() {\n  return (\n    <Image\n      src=\"/image.jpg\"\n      alt=\"Description\"\n      width={500}\n      height={300}\n    />\n  )\n}\n```".to_string()),
            });
        }

        // Check for fetch caching with revalidation
        let has_cache_strategy = self.count_pattern_in_files(info, r"revalidate|fetch\(.*,\s*\{\s*next:\s*\{")?;
        metrics.insert(
            "performance:uses_cache_strategy".to_string(),
            MetricValue::Boolean(has_cache_strategy > 0),
        );

        if has_cache_strategy == 0 && has_app_dir {
            issues.push(ScoreIssue {
                id: "perf-003".to_string(),
                severity: IssueSeverity::Low,
                category: "performance".to_string(),
                title: "No fetch caching strategy detected".to_string(),
                description: "Next.js 13+ supports fetch caching with revalidation. Using cache strategies reduces server load and improves response times.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 2.0,
                suggestion: Some("Add revalidation to fetch calls:\n\n```typescript\n// Static data with revalidation\nfetch('https://api.example.com/data', {\n  next: { revalidate: 60 } // Revalidate every 60 seconds\n})\n\n// Static data (no revalidation)\nfetch('https://api.example.com/data', {\n  cache: 'force-cache'\n})\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Analyze Node.js backend (Express, Fastify, NestJS)
    fn analyze_nodejs_backend(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Check for database query patterns
        let n_plus_one_patterns = self.detect_n_plus_one_queries(info)?;
        metrics.insert(
            "performance:n_plus_one_queries".to_string(),
            MetricValue::Count(n_plus_one_patterns.len()),
        );

        for pattern in n_plus_one_patterns {
            issues.push(ScoreIssue {
                id: "perf-004".to_string(),
                severity: IssueSeverity::High,
                category: "performance".to_string(),
                title: "Potential N+1 query detected".to_string(),
                description: "N+1 queries occur when executing one query to fetch data, then additional queries for each related record.".to_string(),
                file: Some(pattern.file.clone()),
                line: Some(pattern.line),
                column: None,
                impact: 10.0,
                suggestion: Some("Use eager loading or JOIN queries:\n\n```typescript\n// Instead of:\nconst posts = await db.post.findMany();\nfor (const post of posts) {\n  post.author = await db.user.findUnique({ where: { id: post.authorId } });\n}\n\n// Use:\nconst posts = await db.post.findMany({\n  include: { author: true }\n});\n```".to_string()),
            });
        }

        // Check for caching implementation
        let has_redis = info.dependencies.iter().any(|d| d.to_lowercase().contains("redis") || d.to_lowercase().contains("ioredis") || d.to_lowercase().contains("cache-manager"));
        let has_caching = self.count_pattern_in_files(info, r"(cache|Cache)\.set|cache\(|@Cacheable")?;

        metrics.insert(
            "performance:has_caching".to_string(),
            MetricValue::Boolean(has_redis || has_caching > 0),
        );

        if !has_redis && has_caching == 0 {
            issues.push(ScoreIssue {
                id: "perf-005".to_string(),
                severity: IssueSeverity::Medium,
                category: "performance".to_string(),
                title: "No caching layer detected".to_string(),
                description: "Implementing caching reduces database load and improves response times for frequently accessed data.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 5.0,
                suggestion: Some("Consider adding a caching layer:\n\n```typescript\n// Using cache-manager\nimport { CacheManager } from 'cache-manager';\n\nconst cache = new CacheManager({\n  store: 'memory',\n  ttl: 60 // seconds\n});\n\n// Cache expensive queries\nconst data = await cache.get('key', () => {\n  return db.query('SELECT * FROM large_table');\n});\n```".to_string()),
            });
        }

        // Check for streaming responses
        let has_streaming = self.count_pattern_in_files(info, r"stream|res\.write\(|pipe\(")?;
        metrics.insert(
            "performance:uses_streaming".to_string(),
            MetricValue::Boolean(has_streaming > 0),
        );

        // Estimate backend latency from code patterns
        let estimated_latency_ms = self.estimate_backend_latency(info)?;
        metrics.insert(
            "performance:estimated_backend_latency_ms".to_string(),
            MetricValue::Duration(estimated_latency_ms),
        );

        if estimated_latency_ms > 500 {
            issues.push(ScoreIssue {
                id: "perf-006".to_string(),
                severity: IssueSeverity::Medium,
                category: "performance".to_string(),
                title: "High estimated backend latency".to_string(),
                description: format!("Estimated response time is {}ms. Consider optimizing database queries, adding caching, or implementing query batching.", estimated_latency_ms),
                file: None,
                line: None,
                column: None,
                impact: 5.0,
                suggestion: Some("Backend optimization strategies:\n\n1. Add database indexes on frequently queried columns\n2. Use connection pooling\n3. Implement query result caching\n4. Use pagination for large result sets\n5. Consider read replicas for read-heavy workloads".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Analyze Rust backend (Axum, Actix Web, Rocket)
    fn analyze_rust_backend(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Check for async fn usage
        let async_functions = self.count_pattern_in_files(info, r"async\s+fn\s+\w+")?;
        let total_functions = self.count_pattern_in_files(info, r"\bfn\s+\w+")?;

        let async_ratio = if total_functions > 0 {
            (async_functions as f64 / total_functions as f64) * 100.0
        } else {
            0.0
        };

        metrics.insert(
            "performance:async_function_ratio".to_string(),
            MetricValue::Percentage(async_ratio),
        );

        // Check for tokio spawn
        let has_parallel_tasks = self.count_pattern_in_files(info, r"tokio::spawn|tokio::join|join_all")?;
        metrics.insert(
            "performance:uses_parallel_tasks".to_string(),
            MetricValue::Boolean(has_parallel_tasks > 0),
        );

        // Check for blocking calls in async context
        let blocking_patterns = self.count_pattern_in_files(info, r"\.await.*\{\s*std::|std::fs::|std::thread::")?;
        metrics.insert(
            "performance:blocking_in_async_context".to_string(),
            MetricValue::Count(blocking_patterns),
        );

        if blocking_patterns > 0 {
            issues.push(ScoreIssue {
                id: "perf-007".to_string(),
                severity: IssueSeverity::High,
                category: "performance".to_string(),
                title: "Blocking operations in async context".to_string(),
                description: format!("Found {} potential blocking operations in async functions. This can cause executor thread starvation.", blocking_patterns),
                file: None,
                line: None,
                column: None,
                impact: 10.0,
                suggestion: Some("Use async alternatives or spawn blocking tasks:\n\n```rust\n// Instead of:\nasync fn handler() -> Result<Json<Vec<User>>> {\n    // Blocking IO in async context!\n    let data = std::fs::read_to_string(\"data.json\")?;\n    // ...\n}\n\n// Use:\nuse tokio::fs;\n\nasync fn handler() -> Result<Json<Vec<User>>> {\n    let data = fs::read_to_string(\"data.json\").await?;\n    // ...\n}\n\n// Or for truly blocking operations:\nuse tokio::task;\n\nasync fn handler() -> Result<Json<Vec<User>>> {\n    let data = task::spawn_blocking(|| {\n        std::fs::read_to_string(\"data.json\")\n    }).await??;\n    // ...\n}\n```".to_string()),
            });
        }

        // Check for database connection pool
        let has_pool = info.cargo_dependencies.as_ref()
            .map(|deps| deps.iter().any(|(k, _)| {
                k.contains("sqlx") || k.contains("sea-orm") || k.contains("diesel")
            }))
            .unwrap_or(false);

        metrics.insert(
            "performance:uses_connection_pool".to_string(),
            MetricValue::Boolean(has_pool),
        );

        // Check for deadpool usage
        let has_deadpool = self.count_pattern_in_files(info, r"deadpool|Pool::new")?;
        metrics.insert(
            "performance:uses_deadpool".to_string(),
            MetricValue::Boolean(has_deadpool > 0),
        );

        // Estimate latency
        let estimated_latency_ms = self.estimate_rust_latency(info)?;
        metrics.insert(
            "performance:estimated_backend_latency_ms".to_string(),
            MetricValue::Duration(estimated_latency_ms),
        );

        // Check for cloning hot paths
        let excessive_clones = self.count_pattern_in_files(info, r"\.clone\(\)")?;
        metrics.insert(
            "performance:clone_count".to_string(),
            MetricValue::Count(excessive_clones),
        );

        if excessive_clones > 100 {
            issues.push(ScoreIssue {
                id: "perf-008".to_string(),
                severity: IssueSeverity::Medium,
                category: "performance".to_string(),
                title: "Excessive cloning detected".to_string(),
                description: format!("Found {} .clone() calls. Excessive cloning can cause memory allocation overhead and hurt performance.", excessive_clones),
                file: None,
                line: None,
                column: None,
                impact: 3.0,
                suggestion: Some("Consider using references instead of cloning:\n\n```rust\n// Instead of:\nfn process(data: Vec<Data>) -> Result {\n    for item in data.clone() { // Unnecessary clone!\n        // ...\n    }\n}\n\n// Use:\nfn process(data: &[Data]) -> Result {\n    for item in data {\n        // Borrow instead of clone\n    }\n}\n\n// Or use Arc for shared ownership:\nuse std::sync::Arc;\n\nfn process(data: Arc<Vec<Data>>) -> Result {\n    // Multiple references, no clone\n}\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Generic performance analysis
    fn analyze_generic(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Check for basic performance patterns
        let has_loop_in_loop = self.count_pattern_in_files(info, r"for.*for")?;
        metrics.insert(
            "performance:nested_loops".to_string(),
            MetricValue::Count(has_loop_in_loop),
        );

        if has_loop_in_loop > 3 {
            issues.push(ScoreIssue {
                id: "perf-009".to_string(),
                severity: IssueSeverity::Medium,
                category: "performance".to_string(),
                title: "Nested loops detected".to_string(),
                description: "Multiple nested loops can lead to O(n^2) or worse time complexity.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 5.0,
                suggestion: Some("Consider using more efficient algorithms or data structures:\n\n```typescript\n// Instead of nested loops:\nfor (const item of items) {\n  for (const other of others) {\n    if (item.id === other.id) { /* ... */ }\n  }\n}\n\n// Use a Map/Set for O(1) lookups:\nconst otherMap = new Map(others.map(o => [o.id, o]));\nfor (const item of items) {\n  const other = otherMap.get(item.id);\n  if (other) { /* ... */ }\n}\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Analyze bundle size
    fn analyze_bundle_size(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Check package.json for dependencies that affect bundle size
        let heavy_deps = info.dependencies.iter()
            .filter(|d| {
                matches!(
                    d.as_str(),
                    "moment" | "lodash" | "axios" | "rxjs" | "@mui/material" |
                    "@material-ui/core" | "antd" | "bootstrap" | "jquery"
                )
            })
            .count();

        metrics.insert(
            "performance:heavy_dependencies".to_string(),
            MetricValue::Count(heavy_deps),
        );

        if heavy_deps > 0 {
            let heavy_list: Vec<_> = info.dependencies.iter()
                .filter(|d| {
                    matches!(
                        d.as_str(),
                        "moment" | "lodash" | "axios" | "rxjs" | "@mui/material" |
                        "@material-ui/core" | "antd" | "bootstrap" | "jquery"
                    )
                })
                .map(|d| d.as_str())
                .collect();

            issues.push(ScoreIssue {
                id: "perf-010".to_string(),
                severity: IssueSeverity::Medium,
                category: "performance".to_string(),
                title: "Heavy dependencies detected".to_string(),
                description: format!("Found {} dependencies that significantly increase bundle size: {}. Consider lighter alternatives.", heavy_deps, heavy_list.join(", ")),
                file: None,
                line: None,
                column: None,
                impact: 3.0 * heavy_deps as f64,
                suggestion: Some("Consider lighter alternatives:\n\n```\nmoment -> date-fns or Intl.DateTimeFormat\nlodash -> lodash-es (tree-shakeable) or native methods\naxios -> native fetch API\nrxjs -> signals or native async/await\n@mui/material -> @mui/material (tree-shake) or headless UI\nbootstrap -> Tailwind CSS or custom CSS\njquery -> vanilla JavaScript\n```".to_string()),
            });
        }

        // Estimate bundle size based on dependencies
        let estimated_kb = self.estimate_bundle_size(info)?;
        metrics.insert(
            "performance:estimated_bundle_size_kb".to_string(),
            MetricValue::Integer(estimated_kb as i64),
        );

        if estimated_kb > 500 {
            issues.push(ScoreIssue {
                id: "perf-011".to_string(),
                severity: IssueSeverity::High,
                category: "performance".to_string(),
                title: "Large estimated bundle size".to_string(),
                description: format!("Estimated bundle size is {}KB. Target is under 200KB for optimal load times.", estimated_kb),
                file: None,
                line: None,
                column: None,
                impact: 8.0,
                suggestion: Some("Reduce bundle size:\n\n1. Use dynamic imports for code splitting\n2. Use tree-shakeable libraries (ES modules)\n3. Remove unused dependencies\n4. Enable production optimizations\n5. Consider server-side rendering for initial content\n6. Use compression (gzip/brotli)".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Analyze async/await usage
    fn analyze_async_usage(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Check for Promise.all equivalent
        let has_parallel = self.count_pattern_in_files(info, r"Promise\.all|Promise\.allSettled|tokio::join|join_all")?;
        metrics.insert(
            "performance:uses_parallel_async".to_string(),
            MetricValue::Boolean(has_parallel > 0),
        );

        // Check for sequential awaits that could be parallel
        let sequential_await_patterns = self.count_pattern_in_files(info, r"await.*\n.*await")?;
        metrics.insert(
            "performance:sequential_await_chains".to_string(),
            MetricValue::Count(sequential_await_patterns),
        );

        // Check for fire-and-forget (missing await)
        // Note: Rust regex doesn't support lookarounds, so we count potential patterns
        // This is a simple heuristic - may have false positives
        let missing_await = self.count_pattern_in_files(info, r"(?:const|let|var)\s+\w+\s*=\s*[a-zA-Z_]\w*\(")?;
        metrics.insert(
            "performance:potential_fire_and_forget".to_string(),
            MetricValue::Count(missing_await),
        );

        Ok((metrics, issues))
    }

    /// Detect N+1 query patterns
    fn detect_n_plus_one_queries(&self, info: &ProjectInfo) -> LumenResult<Vec<NPlusOnePattern>> {
        let mut patterns = Vec::new();

        let source_files = self.find_source_files(info)?;
        let loop_query_regex = Regex::new(r"(for|while|forEach)\s*\([^)]*\)\s*\{[^}]*\.(find|query|select|get)\s*\(").unwrap();

        for file in source_files {
            let content = std::fs::read_to_string(&file).unwrap_or_default();
            for (line_idx, line) in content.lines().enumerate() {
                if loop_query_regex.is_match(line) {
                    patterns.push(NPlusOnePattern {
                        file: file.to_string_lossy().to_string(),
                        line: line_idx + 1,
                        pattern: line.to_string(),
                    });
                }
            }
        }

        Ok(patterns)
    }

    /// Estimate backend latency from code patterns
    fn estimate_backend_latency(&self, info: &ProjectInfo) -> LumenResult<u64> {
        let mut estimated_ms = 100u64; // Base latency

        // Add latency for database queries
        let db_queries = self.count_pattern_in_files(info, r"\.find\(|\.select\(|\.query\(|\.execute\(")?;
        estimated_ms += (db_queries as u64) * 10;

        // Add latency for external API calls
        let api_calls = self.count_pattern_in_files(info, r"fetch\(|axios\.|http\.|request\(")?;
        estimated_ms += (api_calls as u64) * 50;

        // Add latency for file operations
        let file_ops = self.count_pattern_in_files(info, r"readFile|writeFile|fs\.|readToPath")?;
        estimated_ms += (file_ops as u64) * 5;

        Ok(estimated_ms)
    }

    /// Estimate Rust backend latency
    fn estimate_rust_latency(&self, info: &ProjectInfo) -> LumenResult<u64> {
        let mut estimated_ms = 50u64; // Rust is faster by default

        let db_queries = self.count_pattern_in_files(info, r"\.fetch\(|\.execute\(|query_as\(")?;
        estimated_ms += (db_queries as u64) * 5;

        let external_calls = self.count_pattern_in_files(info, r"reqwest::|surf::|hyper::")?;
        estimated_ms += (external_calls as u64) * 25;

        Ok(estimated_ms)
    }

    /// Estimate bundle size based on dependencies
    fn estimate_bundle_size(&self, info: &ProjectInfo) -> LumenResult<usize> {
        let mut size_kb = 50usize; // Base application size

        // Add size for common dependencies (estimated sizes in KB)
        let dependency_sizes: HashMap<&str, usize> = [
            ("react", 45),
            ("react-dom", 130),
            ("next", 80),
            ("@mui/material", 350),
            ("@material-ui/core", 300),
            ("antd", 500),
            ("lodash", 70),
            ("moment", 70),
            ("axios", 15),
            ("rxjs", 40),
            ("vue", 40),
            ("@angular/core", 150),
        ].into();

        for dep in &info.dependencies {
            if let Some(&size) = dependency_sizes.get(dep.as_str()) {
                size_kb += size;
            } else {
                size_kb += 10; // Default estimate for other packages
            }
        }

        Ok(size_kb)
    }

    /// Count pattern occurrences in source files
    fn count_pattern_in_files(&self, info: &ProjectInfo, pattern: &str) -> LumenResult<usize> {
        let regex = Regex::new(pattern).unwrap();
        let mut count = 0;

        let source_files = self.find_source_files(info)?;
        for file in source_files {
            let content = std::fs::read_to_string(&file).unwrap_or_default();
            count += regex.find_iter(&content).count();
        }

        Ok(count)
    }

    /// Find all source files in the project
    fn find_source_files(&self, info: &ProjectInfo) -> LumenResult<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        let extensions = match info.language {
            Language::TypeScript | Language::JavaScript => &["ts", "tsx", "js", "jsx"][..],
            Language::Rust => &["rs"][..],
            Language::Python => &["py"][..],
            Language::Go => &["go"][..],
            _ => &["ts", "tsx", "js", "jsx", "rs", "py"],
        };

        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_str().unwrap_or("")) {
                        // Skip node_modules and similar
                        let path_str = path.to_string_lossy();
                        if !path_str.contains("node_modules")
                            && !path_str.contains("target")
                            && !path_str.contains(".next")
                            && !path_str.contains("dist")
                        {
                            files.push(path.to_path_buf());
                        }
                    }
                }
            }
        }

        Ok(files)
    }
}

/// N+1 query pattern detected
#[derive(Debug, Clone)]
struct NPlusOnePattern {
    file: String,
    line: usize,
    pattern: String,
}

/// Public function for backward compatibility with the module interface
pub fn analyze(project: &oalacea_lumen_core::Project) -> Vec<ScoreIssue> {
    let analyzer = PerformanceAnalyzer::new(project.info.root.to_string_lossy().to_string());
    let (_, issues) = analyzer.analyze(&project.info).unwrap_or_default();
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_analyzer_creation() {
        let analyzer = PerformanceAnalyzer::new("/tmp".to_string());
        assert_eq!(analyzer.project_root, "/tmp");
    }

    #[test]
    fn test_estimate_bundle_size() {
        // This is a basic test - real implementation would need mock ProjectInfo
        let info = ProjectInfo {
            name: "test".to_string(),
            root: std::path::PathBuf::from("/tmp"),
            framework: Framework::NextJs,
            language: Language::TypeScript,
            test_runner: oalacea_lumen_core::TestRunner::Vitest,
            package_manager: Some("npm".to_string()),
            dependencies: vec!["react".to_string(), "next".to_string()],
            dev_dependencies: vec![],
            database: None,
            package_json: None,
            cargo_dependencies: None,
        };

        let analyzer = PerformanceAnalyzer::new("/tmp".to_string());
        let size = analyzer.estimate_bundle_size(&info).unwrap();
        assert!(size > 50); // Should have at least the base size
    }
}
