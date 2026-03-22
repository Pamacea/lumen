//! Error types for Lumen

use std::path::PathBuf;
use thiserror::Error;

/// Main error type for Lumen
#[derive(Error, Debug)]
pub enum LumenError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("Project not found at: {0}")]
    ProjectNotFound(PathBuf),

    #[error("No framework detected")]
    NoFrameworkDetected,

    #[error("Unsupported framework: {0}")]
    UnsupportedFramework(String),

    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    #[error("Scoring failed: {0}")]
    ScoringFailed(String),

    #[error("Report generation failed: {0}")]
    ReportGenerationFailed(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid glob pattern: {0}")]
    InvalidGlob(String),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Fix application failed: {0}")]
    FixFailed(String),

    #[error("Watch error: {0}")]
    WatchError(String),
}

/// Result type alias
pub type LumenResult<T> = Result<T, LumenError>;
