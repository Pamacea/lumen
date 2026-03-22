//! # Analyze Module
//!
//! Static and dynamic code analysis using tree-sitter AST parsing.

// TODO: Re-enable analyzers after implementing full AST support
// pub mod analyzers;
pub mod ast;
pub mod parsers;

// Re-exports from core
pub use oalacea_lumen_core::prelude::*;

/// Static analyzer configuration
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Enable parallel analysis
    pub parallel: bool,
    /// Maximum number of threads
    pub max_threads: Option<usize>,
    /// Files to exclude
    pub exclude_patterns: Vec<String>,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            parallel: true,
            max_threads: None,
            exclude_patterns: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
        }
    }
}

/// Analysis result
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// File analyzed
    pub file: String,
    /// Language detected
    pub language: String,
    /// Issues found
    pub issues: Vec<Issue>,
    /// Metrics collected
    pub metrics: AnalysisMetrics,
    /// Analysis duration in ms
    pub duration_ms: u64,
}

/// Issue found during analysis
#[derive(Debug, Clone)]
pub struct Issue {
    pub id: String,
    pub severity: IssueSeverity,
    pub category: String,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
}

/// Metrics collected during analysis
#[derive(Debug, Clone, Default)]
pub struct AnalysisMetrics {
    pub lines_of_code: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub function_count: usize,
    pub class_count: usize,
    pub complexity: usize,
}
