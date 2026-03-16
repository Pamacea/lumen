//! Documentation dimension scorer (5% weight)

use crate::{DimensionScore, ScoreIssue, IssueSeverity};
use crate::metrics::MetricValue;

/// Documentation scorer
pub struct DocsScorer;

impl DocsScorer {
    pub fn new() -> Self {
        Self
    }

    pub fn score(&self, metrics: &std::collections::HashMap<String, MetricValue>) -> DimensionScore {
        let readme_score = metrics.get("docs:readme_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let api_coverage = metrics.get("docs:api_coverage")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let comment_ratio = metrics.get("docs:comment_ratio")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Scale comment ratio: 10% is ideal (100 points)
        let comment_score = (comment_ratio * 1000.0).min(100.0);

        let has_changelog = metrics.get("docs:has_changelog")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let changelog_score = if has_changelog { 100.0 } else { 0.0 };

        // Weighted formula: readme 35%, api 30%, comments 20%, changelog 15%
        let score = readme_score * 0.35 + api_coverage * 0.30 + comment_score * 0.20 + changelog_score * 0.15;

        let mut dimension = DimensionScore::new("docs".to_string(), score, 0.05);

        dimension.metrics.insert("readme_score".to_string(), MetricValue::Float(readme_score));
        dimension.metrics.insert("api_coverage".to_string(), MetricValue::Percentage(api_coverage));
        dimension.metrics.insert("comment_ratio".to_string(), MetricValue::Float(comment_ratio));
        dimension.metrics.insert("comment_score".to_string(), MetricValue::Percentage(comment_score));
        dimension.metrics.insert("has_changelog".to_string(), MetricValue::Boolean(has_changelog));

        if readme_score < 50.0 {
            dimension.issues.push(ScoreIssue {
                id: "poor-readme".to_string(),
                severity: IssueSeverity::Medium,
                category: "docs".to_string(),
                title: "Poor README documentation".to_string(),
                description: "README lacks essential information like installation, usage, or examples".to_string(),
                file: Some("README.md".to_string()),
                line: None,
                column: None,
                impact: 10.0,
                suggestion: Some("Add project description, installation instructions, usage examples, and contributing guidelines".to_string()),
            });
        }

        dimension
    }
}

impl Default for DocsScorer {
    fn default() -> Self {
        Self::new()
    }
}
