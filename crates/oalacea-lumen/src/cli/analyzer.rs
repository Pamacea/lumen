//! # Real Code Analyzer
//!
//! Performs actual static analysis on source code files:
//! - Pattern-based vulnerability detection
//! - Code quality metrics (complexity, patterns)
//! - Security scanning (SQLi, XSS, secrets, unsafe code)
//! - Line-by-line analysis with precise locations

use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::{HashMap, HashSet};
use regex::Regex;
use once_cell::sync::Lazy;

use oalacea_lumen_core::{
    scoring::{ScoreIssue, IssueSeverity, MetricValue},
    Language,
};

/// Analysis result for a single file
#[derive(Debug, Clone)]
pub struct FileAnalysis {
    pub path: PathBuf,
    pub language: Language,
    pub lines: usize,
    pub issues: Vec<ScoreIssue>,
    pub metrics: FileMetrics,
}

/// Metrics collected from a file
#[derive(Debug, Clone, Default)]
pub struct FileMetrics {
    pub unwrap_count: usize,
    pub expect_count: usize,
    pub clone_count: usize,
    pub todo_count: usize,
    pub panic_count: usize,
    pub unsafe_count: usize,
    pub unwrap_usage_ratio: f64,
    pub comment_ratio: f64,
    pub avg_line_length: f64,
    pub long_lines: usize,
}

/// Real code analyzer
pub struct CodeAnalyzer {
    /// Project root path
    project_path: PathBuf,
    /// Detected language
    language: Language,
    /// Source files found
    source_files: Vec<PathBuf>,
}

impl CodeAnalyzer {
    /// Create a new analyzer
    pub fn new(project_path: &Path, language: Language) -> Self {
        Self {
            project_path: project_path.to_path_buf(),
            language,
            source_files: Vec::new(),
        }
    }

    /// Find all source files recursively
    pub fn find_source_files(&mut self) -> Result<Vec<PathBuf>> {
        let extensions: &[&str] = match self.language {
            Language::Rust => &["rs"],
            Language::TypeScript | Language::JavaScript => &["ts", "tsx", "js", "jsx"],
            Language::Python => &["py"],
            Language::Go => &["go"],
            Language::Java => &["java"],
            Language::CSharp => &["cs"],
            Language::Unknown => &["rs", "ts", "tsx", "js", "jsx", "go", "py", "java", "cs"],
        };

        // Directories to scan (recursive)
        let scan_dirs = [
            "src", "lib", "app", "components", "pages", "services",
            "utils", "helpers", "handlers", "routes", "controllers",
            "models", "types", "api", "server", "client"
        ];

        let mut sources = Vec::new();

        // First, scan common source directories
        for dir in &scan_dirs {
            let dir_path = self.project_path.join(dir);
            if dir_path.is_dir() {
                self.scan_directory_recursive(&dir_path, extensions, &mut sources)?;
            }
        }

        // Also scan root directory for standalone files
        if let Ok(entries) = fs::read_dir(&self.project_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if extensions.contains(&ext.to_string_lossy().as_ref()) {
                            sources.push(path);
                        }
                    }
                }
            }
        }

        self.source_files = sources.clone();
        Ok(sources)
    }

    /// Recursively scan a directory for source files
    fn scan_directory_recursive(
        &self,
        dir_path: &Path,
        extensions: &[&str],
        sources: &mut Vec<PathBuf>,
    ) -> Result<()> {
        // Exclude common directories to skip
        let exclude_dirs = [
            "node_modules", "target", "dist", "build", ".git",
            "vendor", ".venv", "venv", "__pycache__", ".next",
            ".nuxt", ".cache", "coverage", ".idea", ".vscode"
        ];

        let dir_name = dir_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if exclude_dirs.contains(&dir_name) {
            return Ok(());
        }

        if let Ok(entries) = fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    self.scan_directory_recursive(&path, extensions, sources)?;
                } else if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if extensions.contains(&ext.to_string_lossy().as_ref()) {
                            sources.push(path);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Analyze all source files and collect issues
    pub fn analyze_all(&self) -> Result<Vec<ScoreIssue>> {
        let mut all_issues = Vec::new();

        for file_path in &self.source_files {
            match self.analyze_file(file_path) {
                Ok(analysis) => {
                    all_issues.extend(analysis.issues);
                }
                Err(e) => {
                    // Log but don't fail on individual file errors
                    eprintln!("Warning: Failed to analyze {}: {}", file_path.display(), e);
                }
            }
        }

        Ok(all_issues)
    }

    /// Analyze a single file
    pub fn analyze_file(&self, file_path: &Path) -> Result<FileAnalysis> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let lines: Vec<&str> = content.lines().collect();
        let line_count = lines.len();

        let language = self.detect_file_language(file_path);

        let mut issues = Vec::new();
        let mut metrics = FileMetrics::default();

        // Run security analysis
        issues.extend(self.scan_security_issues(file_path, &lines, language));

        // Run quality analysis
        let quality_issues = self.scan_quality_issues(file_path, &lines, language, &mut metrics);
        issues.extend(quality_issues);

        Ok(FileAnalysis {
            path: file_path.to_path_buf(),
            language,
            lines: line_count,
            issues,
            metrics,
        })
    }

    /// Detect language from file extension
    fn detect_file_language(&self, file_path: &Path) -> Language {
        if let Some(ext) = file_path.extension() {
            match ext.to_string_lossy().as_ref() {
                "rs" => Language::Rust,
                "ts" | "tsx" => Language::TypeScript,
                "js" | "jsx" => Language::JavaScript,
                "py" => Language::Python,
                "go" => Language::Go,
                _ => Language::Unknown,
            }
        } else {
            Language::Unknown
        }
    }

    /// Scan for security vulnerabilities
    fn scan_security_issues(
        &self,
        file_path: &Path,
        lines: &[&str],
        language: Language,
    ) -> Vec<ScoreIssue> {
        let mut issues = Vec::new();
        let path_str = file_path.display().to_string();

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;

            // SQL Injection patterns
            if self.contains_sql_injection(line, language) {
                issues.push(ScoreIssue {
                    id: format!("sqli-{}-{}", path_str, line_num),
                    severity: IssueSeverity::High,
                    category: "Security".to_string(),
                    title: "Potential SQL Injection".to_string(),
                    description: "Direct string concatenation in SQL query".to_string(),
                    file: Some(path_str.clone()),
                    line: Some(line_num),
                    column: None,
                    impact: 15.0,
                    suggestion: Some("Use parameterized queries or prepared statements".to_string()),
                });
            }

            // XSS patterns
            if self.contains_xss(line, language) {
                issues.push(ScoreIssue {
                    id: format!("xss-{}-{}", path_str, line_num),
                    severity: IssueSeverity::High,
                    category: "Security".to_string(),
                    title: "Potential XSS Vulnerability".to_string(),
                    description: "Unescaped user input in HTML context".to_string(),
                    file: Some(path_str.clone()),
                    line: Some(line_num),
                    column: None,
                    impact: 15.0,
                    suggestion: Some("Sanitize user input before rendering".to_string()),
                });
            }

            // Hardcoded secrets
            if let Some(secret_type) = self.detect_secret(line) {
                issues.push(ScoreIssue {
                    id: format!("secret-{}-{}", path_str, line_num),
                    severity: IssueSeverity::Critical,
                    category: "Security".to_string(),
                    title: format!("Hardcoded {}", secret_type),
                    description: format!("{} detected in source code", secret_type),
                    file: Some(path_str.clone()),
                    line: Some(line_num),
                    column: None,
                    impact: 20.0,
                    suggestion: Some("Move to environment variables or secret management".to_string()),
                });
            }

            // Unsafe Rust code
            if language == Language::Rust && line.contains("unsafe") {
                issues.push(ScoreIssue {
                    id: format!("unsafe-{}-{}", path_str, line_num),
                    severity: IssueSeverity::Medium,
                    category: "Security".to_string(),
                    title: "Unsafe Rust Code".to_string(),
                    description: "Usage of unsafe block bypasses Rust safety guarantees".to_string(),
                    file: Some(path_str.clone()),
                    line: Some(line_num),
                    column: None,
                    impact: 5.0,
                    suggestion: Some("Document safety invariants or consider safe alternative".to_string()),
                });
            }
        }

        issues
    }

    /// Check for SQL injection patterns
    fn contains_sql_injection(&self, line: &str, language: Language) -> bool {
        let line_lower = line.to_lowercase();

        // Patterns that suggest SQL queries
        let sql_keywords = ["select", "insert", "update", "delete", "drop", "create"];
        let has_sql = sql_keywords.iter().any(|kw| line_lower.contains(kw));

        if !has_sql {
            return false;
        }

        // Check for dangerous concatenation patterns
        match language {
            Language::Rust => {
                // format!("SELECT * FROM users WHERE id = {}", user_input)
                // &format!(...), format.concat!(...)
                line.contains("format!(") || line.contains("&format!")
                    || (line.contains("+") && line_lower.contains("'"))
            }
            Language::TypeScript | Language::JavaScript => {
                // `SELECT * FROM users WHERE id = ${userInput}`
                // "SELECT ... " + userInput
                line.contains("${") && line.contains("SELECT")
                    || (line.contains("+") && line_lower.contains("'") && line_lower.contains("select"))
            }
            Language::Python => {
                // f"SELECT * FROM users WHERE id = {user_input}"
                // "SELECT ... " + user_input
                line.starts_with("f\"") || line.starts_with("f'")
                    || (line.contains("+") && line_lower.contains("'") && line_lower.contains("select"))
            }
            _ => false,
        }
    }

    /// Check for XSS patterns
    fn contains_xss(&self, line: &str, language: Language) -> bool {
        let line_lower = line.to_lowercase();

        // HTML context patterns
        let html_keywords = ["innerhtml", "outerhtml", "insertadjacenthtml", "dangerouslysetinnerhtml"];
        let has_html_context = html_keywords.iter().any(|kw| line_lower.contains(kw));

        if !has_html_context {
            return false;
        }

        match language {
            Language::TypeScript | Language::JavaScript => {
                // el.innerHTML = userInput
                // {__html: userInput}
                line_lower.contains("=") && line_lower.contains("innerhtml")
                    || line_lower.contains("__html")
            }
            Language::Rust => {
                // In Rust web frameworks, check for unescaped rendering
                line_lower.contains("html!") && (line.contains("{") || line.contains("{{"))
            }
            Language::Python => {
                // Django/Jinja2 patterns with autoescape off
                line_lower.contains("autoescape") || line_lower.contains("safe")
            }
            _ => false,
        }
    }

    /// Detect hardcoded secrets
    fn detect_secret(&self, line: &str) -> Option<&'static str> {
        let line_lower = line.to_lowercase();

        // Common secret patterns
        let secret_patterns: &[(&str, &str)] = &[
            ("password", "password"),
            ("passwd", "password"),
            ("api_key", "API key"),
            ("apikey", "API key"),
            ("api-key", "API key"),
            ("secret", "secret"),
            ("token", "access token"),
            ("private_key", "private key"),
            ("private-key", "private key"),
            ("auth_token", "auth token"),
            ("access_token", "access token"),
            ("refresh_token", "refresh token"),
            ("bearer", "bearer token"),
            ("connection_string", "connection string"),
            ("connstring", "connection string"),
            ("mongodb://", "MongoDB URI"),
            ("postgresql://", "PostgreSQL URI"),
            ("mysql://", "MySQL URI"),
            ("redis://", "Redis URI"),
            ("aws_access_key", "AWS access key"),
            ("aws_secret", "AWS secret key"),
            ("ghp_", "GitHub personal access token"),
            ("gho_", "GitHub OAuth token"),
            ("ghu_", "GitHub user token"),
            ("ghs_", "GitHub server token"),
            ("ghr_", "GitHub refresh token"),
            ("sk_live_", "Stripe live key"),
            ("sk_test_", "Stripe test key"),
            ("pk_live_", "Stripe publishable key"),
        ];

        for (pattern, secret_type) in secret_patterns {
            if line_lower.contains(pattern) {
                // Check if it's a hardcoded value (not a variable reference)
                let has_equals = line.contains('=');
                let has_quotes = line.contains('"') || line.contains('\'');
                let has_env_var = line.contains("env:") || line.contains("std::env")
                    || line.contains("process.env") || line.contains("os.getenv")
                    || line.contains("dotenv");

                if has_equals && has_quotes && !has_env_var {
                    return Some(secret_type);
                }
            }
        }

        // Check for JWT-like patterns
        if line.contains("eyJ") && (line.contains('.') || line.contains("Bearer")) {
            return Some("JWT token");
        }

        None
    }

    /// Scan for code quality issues
    fn scan_quality_issues(
        &self,
        file_path: &Path,
        lines: &[&str],
        language: Language,
        metrics: &mut FileMetrics,
    ) -> Vec<ScoreIssue> {
        let mut issues = Vec::new();
        let path_str = file_path.display().to_string();

        let mut unwrap_count = 0;
        let mut expect_count = 0;
        let mut clone_count = 0;
        let mut todo_count = 0;
        let mut panic_count = 0;
        let mut unsafe_count = 0;
        let mut comment_lines = 0;
        let mut total_line_length = 0;
        let mut long_lines = 0;

        for (idx, line) in lines.iter().enumerate() {
            let line_num = idx + 1;
            let trimmed = line.trim();
            total_line_length += line.len();

            if line.len() > 100 {
                long_lines += 1;
            }

            // Count comment lines
            if trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*") {
                comment_lines += 1;
            }

            match language {
                Language::Rust => {
                    // .unwrap() patterns
                    if trimmed.contains(".unwrap()") {
                        unwrap_count += 1;
                        issues.push(ScoreIssue {
                            id: format!("unwrap-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Medium,
                            category: "Quality".to_string(),
                            title: "Use of unwrap()".to_string(),
                            description: "unwrap() will panic on None/Err".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 3.0,
                            suggestion: Some("Use proper error handling with ? or match".to_string()),
                        });
                    }

                    // .expect() patterns
                    if trimmed.contains(".expect(") {
                        expect_count += 1;
                        issues.push(ScoreIssue {
                            id: format!("expect-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Medium,
                            category: "Quality".to_string(),
                            title: "Use of expect()".to_string(),
                            description: "expect() will panic with custom message".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 3.0,
                            suggestion: Some("Use proper error handling with ? or match".to_string()),
                        });
                    }

                    // .clone() patterns
                    if trimmed.contains(".clone()") {
                        clone_count += 1;
                        if clone_count > 5 {
                            issues.push(ScoreIssue {
                                id: format!("clone-{}-{}", path_str, line_num),
                                severity: IssueSeverity::Low,
                                category: "Performance".to_string(),
                                title: "Excessive cloning".to_string(),
                                description: "Multiple .clone() calls may impact performance".to_string(),
                                file: Some(path_str.clone()),
                                line: Some(line_num),
                                column: None,
                                impact: 2.0,
                                suggestion: Some("Consider borrowing or using references".to_string()),
                            });
                        }
                    }

                    // panic! patterns
                    if trimmed.contains("panic!") || trimmed.contains("unreachable!") {
                        panic_count += 1;
                        issues.push(ScoreIssue {
                            id: format!("panic-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Medium,
                            category: "Quality".to_string(),
                            title: "Explicit panic".to_string(),
                            description: "Code will panic at runtime".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 5.0,
                            suggestion: Some("Use Result<T, E> for error handling".to_string()),
                        });
                    }

                    // TODO/FIXME comments
                    if trimmed.to_lowercase().contains("todo!") || trimmed.to_lowercase().contains("todo(") {
                        todo_count += 1;
                        issues.push(ScoreIssue {
                            id: format!("todo-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Low,
                            category: "Quality".to_string(),
                            title: "TODO marker".to_string(),
                            description: "Incomplete code marked with TODO".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 1.0,
                            suggestion: Some("Complete the implementation or file an issue".to_string()),
                        });
                    }

                    // unsafe blocks (already counted in security)
                    if trimmed.contains("unsafe") {
                        unsafe_count += 1;
                    }
                }

                Language::TypeScript | Language::JavaScript => {
                    // any type usage
                    if trimmed.contains(": any") || trimmed.contains("<any>") || trimmed.contains("Any") {
                        issues.push(ScoreIssue {
                            id: format!("any-type-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Medium,
                            category: "Quality".to_string(),
                            title: "Use of 'any' type".to_string(),
                            description: "Using 'any' disables type checking".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 3.0,
                            suggestion: Some("Use specific types or 'unknown' with validation".to_string()),
                        });
                    }

                    // @ts-ignore / @ts-nocheck
                    if trimmed.contains("@ts-ignore") || trimmed.contains("@ts-nocheck") {
                        issues.push(ScoreIssue {
                            id: format!("ts-ignore-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Medium,
                            category: "Quality".to_string(),
                            title: "TypeScript check suppressed".to_string(),
                            description: "Type safety is being bypassed".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 5.0,
                            suggestion: Some("Fix the type error instead of suppressing it".to_string()),
                        });
                    }

                    // console.log in production code
                    if trimmed.contains("console.log") || trimmed.contains("console.debug") {
                        issues.push(ScoreIssue {
                            id: format!("console-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Low,
                            category: "Quality".to_string(),
                            title: "Console statement".to_string(),
                            description: "Console.log statement in source code".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 1.0,
                            suggestion: Some("Remove or replace with proper logging".to_string()),
                        });
                    }

                    // TODO/FIXME comments
                    let lower = trimmed.to_lowercase();
                    if lower.contains("// todo") || lower.contains("// fixme")
                        || lower.contains("/* todo") || lower.contains("/* fixme") {
                        todo_count += 1;
                        issues.push(ScoreIssue {
                            id: format!("todo-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Low,
                            category: "Quality".to_string(),
                            title: "TODO marker".to_string(),
                            description: "Incomplete code marked with TODO/FIXME".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 1.0,
                            suggestion: Some("Complete the implementation or file an issue".to_string()),
                        });
                    }
                }

                Language::Python => {
                    // TODO/FIXME comments
                    let lower = trimmed.to_lowercase();
                    if lower.contains("# todo") || lower.contains("# fixme") {
                        todo_count += 1;
                        issues.push(ScoreIssue {
                            id: format!("todo-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Low,
                            category: "Quality".to_string(),
                            title: "TODO marker".to_string(),
                            description: "Incomplete code marked with TODO/FIXME".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 1.0,
                            suggestion: Some("Complete the implementation or file an issue".to_string()),
                        });
                    }

                    // bare except
                    if trimmed.contains("except:") || trimmed.contains("except Exception:") {
                        issues.push(ScoreIssue {
                            id: format!("bare-except-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Medium,
                            category: "Quality".to_string(),
                            title: "Bare except clause".to_string(),
                            description: "Catching all exceptions is dangerous".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 5.0,
                            suggestion: Some("Catch specific exceptions only".to_string()),
                        });
                    }

                    // print in production code
                    if trimmed.starts_with("print(") && !lower.contains("logging") {
                        issues.push(ScoreIssue {
                            id: format!("print-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Low,
                            category: "Quality".to_string(),
                            title: "Print statement".to_string(),
                            description: "Print statement in source code".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 1.0,
                            suggestion: Some("Use proper logging framework".to_string()),
                        });
                    }
                }

                Language::Go => {
                    // TODO/FIXME comments
                    let lower = trimmed.to_lowercase();
                    if lower.contains("// todo") || lower.contains("// fixme") {
                        todo_count += 1;
                        issues.push(ScoreIssue {
                            id: format!("todo-{}-{}", path_str, line_num),
                            severity: IssueSeverity::Low,
                            category: "Quality".to_string(),
                            title: "TODO marker".to_string(),
                            description: "Incomplete code marked with TODO/FIXME".to_string(),
                            file: Some(path_str.clone()),
                            line: Some(line_num),
                            column: None,
                            impact: 1.0,
                            suggestion: Some("Complete the implementation or file an issue".to_string()),
                        });
                    }
                }

                _ => {}
            }
        }

        // Update metrics
        metrics.unwrap_count = unwrap_count;
        metrics.expect_count = expect_count;
        metrics.clone_count = clone_count;
        metrics.todo_count = todo_count;
        metrics.panic_count = panic_count;
        metrics.unsafe_count = unsafe_count;
        metrics.long_lines = long_lines;

        if !lines.is_empty() {
            metrics.comment_ratio = comment_lines as f64 / lines.len() as f64;
            metrics.avg_line_length = total_line_length as f64 / lines.len() as f64;
        }

        // Check for very long lines issue
        if long_lines > 10 {
            issues.push(ScoreIssue {
                id: format!("long-lines-{}", path_str),
                severity: IssueSeverity::Low,
                category: "Quality".to_string(),
                title: "Many long lines".to_string(),
                description: format!("{} lines exceed 100 characters", long_lines),
                file: Some(path_str.clone()),
                line: None,
                column: None,
                impact: 2.0,
                suggestion: Some("Break long lines for better readability".to_string()),
            });
        }

        // Check for low comment ratio
        if lines.len() > 50 && metrics.comment_ratio < 0.05 {
            issues.push(ScoreIssue {
                id: format!("low-comments-{}", path_str),
                severity: IssueSeverity::Low,
                category: "Documentation".to_string(),
                title: "Low comment ratio".to_string(),
                description: format!("Only {:.1}% of lines are comments", metrics.comment_ratio * 100.0),
                file: Some(path_str.clone()),
                line: None,
                column: None,
                impact: 2.0,
                suggestion: Some("Add documentation comments to explain complex logic".to_string()),
            });
        }

        issues
    }

    /// Calculate quality metrics from all files
    pub fn calculate_quality_metrics(&self) -> Result<HashMap<String, MetricValue>> {
        let mut metrics = HashMap::new();

        let total_files = self.source_files.len();
        metrics.insert("total_files".to_string(), MetricValue::Count(total_files));

        let mut total_lines = 0;
        let mut total_unwrap = 0;
        let mut total_clone = 0;
        let mut total_panic = 0;
        let mut total_unsafe = 0;
        let mut total_todo = 0;

        for file_path in &self.source_files {
            if let Ok(analysis) = self.analyze_file(file_path) {
                total_lines += analysis.lines;
                total_unwrap += analysis.metrics.unwrap_count;
                total_clone += analysis.metrics.clone_count;
                total_panic += analysis.metrics.panic_count;
                total_unsafe += analysis.metrics.unsafe_count;
                total_todo += analysis.metrics.todo_count;
            }
        }

        metrics.insert("total_lines".to_string(), MetricValue::Count(total_lines));
        metrics.insert("unwrap_count".to_string(), MetricValue::Count(total_unwrap));
        metrics.insert("clone_count".to_string(), MetricValue::Count(total_clone));
        metrics.insert("panic_count".to_string(), MetricValue::Count(total_panic));
        metrics.insert("unsafe_count".to_string(), MetricValue::Count(total_unsafe));
        metrics.insert("todo_count".to_string(), MetricValue::Count(total_todo));

        // Calculate derived metrics
        let unwrap_per_1000 = if total_lines > 0 {
            (total_unwrap as f64 / total_lines as f64) * 1000.0
        } else {
            0.0
        };
        metrics.insert("unwrap_per_1000_lines".to_string(), MetricValue::Float(unwrap_per_1000));

        Ok(metrics)
    }
}

/// Regex patterns for common issues (compiled once)
static SECRET_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Generic API keys
        Regex::new(r##"(?i)(api[_-]?key|apikey)\s*=\s*['"]([a-zA-Z0-9_-]{20,})['"]"##).unwrap(),
        // JWT tokens
        Regex::new(r"eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+").unwrap(),
        // AWS keys
        Regex::new(r##"(?i)aws[_-]?(access[_-]?key[_-]?id|secret[_-]?access[_-]?key)\s*=\s*['"]([A-Z0-9]{20})['"]"##).unwrap(),
        // GitHub tokens
        Regex::new(r"(ghp_|gho_|ghu_|ghs_|ghr_)[a-zA-Z0-9]{36}").unwrap(),
        // Stripe keys
        Regex::new(r"(sk_live_|sk_test_|pk_live_|pk_test_)[a-zA-Z0-9]{24,}").unwrap(),
        // Password/Passwd
        Regex::new(r##"(?i)(password|passwd)\s*=\s*['"]([^"']{4,})['"]"##).unwrap(),
    ]
});

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sql_injection_detection_rust() {
        let analyzer = CodeAnalyzer::new(Path::new("."), Language::Rust);

        assert!(analyzer.contains_sql_injection(
            "format!(\"SELECT * FROM users WHERE id = {}\", user_input)",
            Language::Rust
        ));

        assert!(!analyzer.contains_sql_injection(
            "client.query(\"SELECT * FROM users\", &[])",
            Language::Rust
        ));
    }

    #[test]
    fn test_sql_injection_detection_typescript() {
        let analyzer = CodeAnalyzer::new(Path::new("."), Language::TypeScript);

        assert!(analyzer.contains_sql_injection(
            "const query = `SELECT * FROM users WHERE id = ${userId}`",
            Language::TypeScript
        ));
    }

    #[test]
    fn test_secret_detection() {
        let analyzer = CodeAnalyzer::new(Path::new("."), Language::Rust);

        assert!(analyzer.detect_secret("const API_KEY = \"sk_live_1234567890abcdef\"").is_some());
        assert!(analyzer.detect_secret("password = \"hardcoded123\"").is_some());
        assert!(analyzer.detect_secret("let api_key = std::env::var(\"API_KEY\")?").is_none());
    }

    #[test]
    fn test_xss_detection() {
        let analyzer = CodeAnalyzer::new(Path::new("."), Language::TypeScript);

        assert!(analyzer.contains_xss(
            "element.innerHTML = userInput",
            Language::TypeScript
        ));

        assert!(!analyzer.contains_xss(
            "element.textContent = userInput",
            Language::TypeScript
        ));
    }
}
