//! Configuration for Lumen

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main Lumen configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// General settings
    pub general: GeneralConfig,
    /// Scoring settings
    pub scoring: ScoringConfig,
    /// Detection settings
    pub detection: DetectionConfig,
    /// Analysis settings
    pub analysis: AnalysisConfig,
    /// Report settings
    pub report: ReportConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            scoring: ScoringConfig::default(),
            detection: DetectionConfig::default(),
            analysis: AnalysisConfig::default(),
            report: ReportConfig::default(),
        }
    }
}

/// Project-specific Lumen configuration (loaded from lumen.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LumenConfig {
    /// Project type override
    pub project_type: Option<String>,
    /// Test runner override
    pub test_runner: Option<String>,
    /// Exclude patterns
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// General configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Verbose output
    #[serde(default)]
    pub verbose: bool,
    /// Quiet mode
    #[serde(default)]
    pub quiet: bool,
    /// No color output
    #[serde(default)]
    pub no_color: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            quiet: false,
            no_color: false,
        }
    }
}

/// Scoring configuration with 7 dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// Dimension weights
    pub weights: DimensionWeights,
    /// Scoring thresholds
    pub thresholds: ScoringThresholds,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            weights: DimensionWeights::default(),
            thresholds: ScoringThresholds::default(),
        }
    }
}

/// Dimension weights (must sum to 1.0)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionWeights {
    /// Test coverage: 25%
    #[serde(default = "default_coverage")]
    pub coverage: f64,
    /// Code quality: 20%
    #[serde(default = "default_quality")]
    pub quality: f64,
    /// Performance: 15%
    #[serde(default = "default_performance")]
    pub performance: f64,
    /// Security: 15%
    #[serde(default = "default_security")]
    pub security: f64,
    /// SEO: 10%
    #[serde(default = "default_seo")]
    pub seo: f64,
    /// Documentation: 5%
    #[serde(default = "default_docs")]
    pub docs: f64,
    /// UI/UX: 10%
    #[serde(default = "default_uiux")]
    pub uiux: f64,
}

impl Default for DimensionWeights {
    fn default() -> Self {
        Self {
            coverage: 0.25,
            quality: 0.20,
            performance: 0.15,
            security: 0.15,
            seo: 0.10,
            docs: 0.05,
            uiux: 0.10,
        }
    }
}

fn default_coverage() -> f64 { 0.25 }
fn default_quality() -> f64 { 0.20 }
fn default_performance() -> f64 { 0.15 }
fn default_security() -> f64 { 0.15 }
fn default_seo() -> f64 { 0.10 }
fn default_docs() -> f64 { 0.05 }
fn default_uiux() -> f64 { 0.10 }

/// Scoring thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringThresholds {
    /// Excellent threshold
    #[serde(default = "default_excellent")]
    pub excellent: f64,
    /// Good threshold
    #[serde(default = "default_good")]
    pub good: f64,
}

impl Default for ScoringThresholds {
    fn default() -> Self {
        Self {
            excellent: 80.0,
            good: 60.0,
        }
    }
}

fn default_excellent() -> f64 { 80.0 }
fn default_good() -> f64 { 60.0 }

/// Detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionConfig {
    /// Paths to exclude from detection
    #[serde(default)]
    pub exclude_paths: Vec<String>,
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self {
            exclude_paths: vec![
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".git".to_string(),
                "vendor".to_string(),
            ],
        }
    }
}

/// Analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisConfig {
    /// Enable static analysis
    #[serde(default = "default_true")]
    pub static_analysis: bool,
    /// Enable security analysis
    #[serde(default = "default_true")]
    pub security: bool,
    /// Enable dependency analysis
    #[serde(default = "default_true")]
    pub dependencies: bool,
    /// Enable performance analysis
    #[serde(default = "default_true")]
    pub performance: bool,
    /// Enable SEO analysis
    #[serde(default = "default_true")]
    pub seo: bool,
    /// Enable UI/UX analysis
    #[serde(default = "default_true")]
    pub uiux: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            static_analysis: true,
            security: true,
            dependencies: true,
            performance: true,
            seo: true,
            uiux: true,
        }
    }
}

fn default_true() -> bool { true }

/// Report configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Output directory
    #[serde(default)]
    pub output_dir: PathBuf,
    /// Report formats
    #[serde(default)]
    pub formats: Vec<ReportFormat>,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./lumen-reports"),
            formats: vec![ReportFormat::Markdown, ReportFormat::Json],
        }
    }
}

/// Report format
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ReportFormat {
    #[serde(rename = "md")]
    Markdown,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "html")]
    Html,
    #[serde(rename = "junit")]
    JUnit,
}
