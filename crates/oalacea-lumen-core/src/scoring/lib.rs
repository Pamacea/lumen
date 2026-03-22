//! # Lumen Scoring
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

impl DimensionScores {
    /// Get all dimension scores as a vector
    pub fn all(&self) -> Vec<(&'static str, &DimensionScore)> {
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

    /// Add a metric to this dimension
    pub fn with_metric(mut self, key: String, value: MetricValue) -> Self {
        self.metrics.insert(key, value);
        self
    }

    /// Add metrics from a HashMap
    pub fn with_metrics(mut self, metrics: HashMap<String, MetricValue>) -> Self {
        self.metrics.extend(metrics);
        self
    }

    /// Add issues to this dimension
    pub fn with_issues(mut self, issues: Vec<ScoreIssue>) -> Self {
        self.issues = issues;
        self
    }

    /// Add improvements to this dimension
    pub fn with_improvements(mut self, improvements: Vec<Improvement>) -> Self {
        self.improvements = improvements;
        self
    }

    /// Calculate score impact from issues
    pub fn calculate_issue_penalty(&self) -> f64 {
        self.issues
            .iter()
            .map(|issue| match issue.severity {
                IssueSeverity::Critical => 20.0,
                IssueSeverity::High => 10.0,
                IssueSeverity::Medium => 5.0,
                IssueSeverity::Low => 1.0,
                IssueSeverity::Info => 0.0,
            })
            .sum()
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

impl IssueSeverity {
    pub fn as_str(&self) -> &str {
        match self {
            IssueSeverity::Info => "info",
            IssueSeverity::Low => "low",
            IssueSeverity::Medium => "medium",
            IssueSeverity::High => "high",
            IssueSeverity::Critical => "critical",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            IssueSeverity::Info => "blue",
            IssueSeverity::Low => "cyan",
            IssueSeverity::Medium => "yellow",
            IssueSeverity::High => "orange",
            IssueSeverity::Critical => "red",
        }
    }
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
    Trivial,   // < 30 min
    Low,       // 30 min - 2 hours
    Medium,    // 2 - 6 hours
    High,      // 6 - 16 hours
    VeryHigh,  // > 16 hours
}

impl Effort {
    pub fn as_str(&self) -> &str {
        match self {
            Effort::Trivial => "trivial",
            Effort::Low => "low",
            Effort::Medium => "medium",
            Effort::High => "high",
            Effort::VeryHigh => "very-high",
        }
    }

    pub fn hours(&self) -> f64 {
        match self {
            Effort::Trivial => 0.5,
            Effort::Low => 1.0,
            Effort::Medium => 4.0,
            Effort::High => 10.0,
            Effort::VeryHigh => 24.0,
        }
    }
}

/// Expected score impact
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Impact {
    Minimal,  // +1-3 points
    Low,      // +3-8 points
    Medium,   // +8-20 points
    High,     // +20-40 points
    VeryHigh, // > 40 points
}

impl Impact {
    pub fn as_str(&self) -> &str {
        match self {
            Impact::Minimal => "minimal",
            Impact::Low => "low",
            Impact::Medium => "medium",
            Impact::High => "high",
            Impact::VeryHigh => "very-high",
        }
    }

    pub fn points(&self) -> (f64, f64) {
        match self {
            Impact::Minimal => (1.0, 3.0),
            Impact::Low => (3.0, 8.0),
            Impact::Medium => (8.0, 20.0),
            Impact::High => (20.0, 40.0),
            Impact::VeryHigh => (40.0, 100.0),
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_score_creation() {
        let score = DimensionScore::new("test".to_string(), 85.0, 0.25);
        assert_eq!(score.score, 85.0);
        assert_eq!(score.weight, 0.25);
        assert_eq!(score.weighted, 21.25);
        assert_eq!(score.grade, Grade::B);
    }

    #[test]
    fn test_dimension_scores_weighted_sum() {
        let mut scores = DimensionScores::default();
        scores.coverage.score = 80.0;
        scores.coverage.weighted = 20.0;
        scores.quality.score = 70.0;
        scores.quality.weighted = 14.0;
        scores.performance.score = 90.0;
        scores.performance.weighted = 13.5;
        scores.security.score = 85.0;
        scores.security.weighted = 12.75;
        scores.seo.score = 60.0;
        scores.seo.weighted = 6.0;
        scores.docs.score = 50.0;
        scores.docs.weighted = 2.5;
        scores.uiux.score = 75.0;
        scores.uiux.weighted = 7.5;

        let sum = scores.weighted_sum();
        assert!((sum - 76.25).abs() < 0.01);
    }

    #[test]
    fn test_issue_severity_ordering() {
        assert!(IssueSeverity::Critical > IssueSeverity::High);
        assert!(IssueSeverity::High > IssueSeverity::Medium);
        assert!(IssueSeverity::Medium > IssueSeverity::Low);
        assert!(IssueSeverity::Low > IssueSeverity::Info);
    }

    #[test]
    fn test_score_clamping() {
        let score = DimensionScore::new("test".to_string(), 150.0, 0.25);
        assert_eq!(score.score, 100.0);

        let score2 = DimensionScore::new("test".to_string(), -10.0, 0.25);
        assert_eq!(score2.score, 0.0);
    }
}

// Public API functions to force type exports
pub fn create_score_issue(
    id: String,
    severity: IssueSeverity,
    category: String,
    title: String,
    description: String,
    impact: f64,
) -> ScoreIssue {
    ScoreIssue {
        id,
        severity,
        category,
        title,
        description,
        file: None,
        line: None,
        column: None,
        impact,
        suggestion: None,
    }
}

pub fn issue_severity_from_score(score: f64) -> IssueSeverity {
    match score as i32 {
        0..=20 => IssueSeverity::Critical,
        21..=40 => IssueSeverity::High,
        41..=60 => IssueSeverity::Medium,
        61..=80 => IssueSeverity::Low,
        _ => IssueSeverity::Info,
    }
}
