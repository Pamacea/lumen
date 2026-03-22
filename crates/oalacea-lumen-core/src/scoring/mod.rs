//! # Scoring Module
//!
//! 7-Dimension quality scoring system with configurable weights and trend analysis.

pub mod calculator;
pub mod dimensions;
pub mod grade;
pub mod metrics;
pub mod trend;
pub mod history;
pub mod types;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use calculator::ScoreCalculator;
pub use grade::{Grade, GradeSystem};
pub use history::ScoreHistory;
pub use trend::{TrendAnalysis, HistoricalScore, TrendDirection, ScoreDelta, MovingAverage};
pub use metrics::MetricValue;

/// Overall project score with all 7 dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectScore {
    /// Project name
    pub project_name: String,
    /// Git commit SHA
    pub commit_sha: String,
    /// Timestamp of scoring
    pub timestamp: i64,
    /// Overall weighted score (0-100)
    pub overall: f64,
    /// Grade (A+/A/A-/B+/B/B-/C+/C/C-/D+/D/F)
    pub grade: Grade,
    /// Individual dimension scores
    pub dimensions: DimensionScores,
    /// Historical trend analysis (if available)
    pub trend: Option<TrendAnalysis>,
    /// Metadata about the scoring run
    pub metadata: ScoreMetadata,
}

/// Individual dimension scores (7 dimensions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScores {
    /// Test coverage (25%)
    pub coverage: DimensionScore,
    /// Code quality (20%)
    pub quality: DimensionScore,
    /// Performance (15%)
    pub performance: DimensionScore,
    /// Security (15%)
    pub security: DimensionScore,
    /// SEO (10%)
    pub seo: DimensionScore,
    /// Documentation (5%)
    pub docs: DimensionScore,
    /// UI/UX (10%)
    pub uiux: DimensionScore,
}

impl Default for DimensionScores {
    fn default() -> Self {
        Self {
            coverage: DimensionScore::new("coverage".to_string(), 0.0, 0.25),
            quality: DimensionScore::new("quality".to_string(), 0.0, 0.20),
            performance: DimensionScore::new("performance".to_string(), 0.0, 0.15),
            security: DimensionScore::new("security".to_string(), 0.0, 0.15),
            seo: DimensionScore::new("seo".to_string(), 0.0, 0.10),
            docs: DimensionScore::new("docs".to_string(), 0.0, 0.05),
            uiux: DimensionScore::new("uiux".to_string(), 0.0, 0.10),
        }
    }
}

impl DimensionScores {
    /// Calculate overall weighted score
    pub fn weighted_sum(&self) -> f64 {
        self.coverage.weighted
            + self.quality.weighted
            + self.performance.weighted
            + self.security.weighted
            + self.seo.weighted
            + self.docs.weighted
            + self.uiux.weighted
    }

    /// Get all dimension scores as a vector
    pub fn all(&self) -> Vec<(&str, &DimensionScore)> {
        vec![
            ("coverage", &self.coverage),
            ("quality", &self.quality),
            ("performance", &self.performance),
            ("security", &self.security),
            ("seo", &self.seo),
            ("docs", &self.docs),
            ("uiux", &self.uiux),
        ]
    }

    /// Get a dimension by name
    pub fn get(&self, name: &str) -> Option<&DimensionScore> {
        match name {
            "coverage" => Some(&self.coverage),
            "quality" => Some(&self.quality),
            "performance" => Some(&self.performance),
            "security" => Some(&self.security),
            "seo" => Some(&self.seo),
            "docs" => Some(&self.docs),
            "uiux" => Some(&self.uiux),
            _ => None,
        }
    }

    /// Get the minimum score across all dimensions
    pub fn min_score(&self) -> f64 {
        self.all()
            .iter()
            .map(|(_, s)| s.score)
            .fold(f64::INFINITY, f64::min)
    }

    /// Get the maximum score across all dimensions
    pub fn max_score(&self) -> f64 {
        self.all()
            .iter()
            .map(|(_, s)| s.score)
            .fold(0.0, f64::max)
    }
}

/// Score for a single dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    /// Dimension name
    pub name: String,
    /// Raw score (0-100)
    pub score: f64,
    /// Weight in overall calculation (0-1)
    pub weight: f64,
    /// Weighted score (score * weight)
    pub weighted: f64,
    /// Grade
    pub grade: Grade,
    /// Metrics collected for this dimension
    pub metrics: HashMap<String, MetricValue>,
    /// Issues found that affected the score
    pub issues: Vec<ScoreIssue>,
    /// Improvements suggested
    pub improvements: Vec<Improvement>,
}

impl DimensionScore {
    /// Create a new dimension score
    pub fn new(name: String, score: f64, weight: f64) -> Self {
        let weighted = score * weight;
        Self {
            name,
            score: score.clamp(0.0, 100.0),
            weight,
            weighted,
            grade: Grade::from_score(score),
            metrics: HashMap::new(),
            issues: Vec::new(),
            improvements: Vec::new(),
        }
    }
}

/// Issue affecting score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreIssue {
    /// Issue ID
    pub id: String,
    /// Severity
    pub severity: IssueSeverity,
    /// Category
    pub category: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// File location (if applicable)
    pub file: Option<String>,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Column number (if applicable)
    pub column: Option<usize>,
    /// Score impact (points lost)
    pub impact: f64,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Issue severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Info = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Suggested improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Improvement {
    /// Improvement ID
    pub id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Estimated effort required
    pub effort: Effort,
    /// Expected score impact
    pub impact: Impact,
    /// Dimension this improvement applies to
    pub dimension: String,
}

/// Effort required for improvement
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Effort {
    Trivial,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Expected score impact
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Impact {
    Minimal,
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Metadata about the scoring run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreMetadata {
    /// Lumen version
    pub scorer_version: String,
    /// Duration of the scan in milliseconds
    pub scan_duration_ms: u64,
    /// Number of files scanned
    pub files_scanned: usize,
    /// Total lines of code
    pub lines_of_code: usize,
    /// Language breakdown (lines per language)
    pub language_breakdown: HashMap<String, usize>,
    /// Configuration profile used
    pub profile: String,
}

impl Default for ScoreMetadata {
    fn default() -> Self {
        Self {
            scorer_version: env!("CARGO_PKG_VERSION").to_string(),
            scan_duration_ms: 0,
            files_scanned: 0,
            lines_of_code: 0,
            language_breakdown: HashMap::new(),
            profile: "standard".to_string(),
        }
    }
}
