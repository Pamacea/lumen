//! Quality dimension scorer (20% weight)

use crate::{DimensionScore, ScoreIssue, IssueSeverity};
use crate::metrics::{MetricValue, Normalize, MetricThresholds};

/// Quality scorer
pub struct QualityScorer {
    thresholds: MetricThresholds,
}

impl QualityScorer {
    pub fn new(thresholds: MetricThresholds) -> Self {
        Self { thresholds }
    }

    pub fn score(&self, metrics: &std::collections::HashMap<String, MetricValue>) -> DimensionScore {
        // Complexity score (inverse - lower complexity is better)
        let complexity = metrics.get("quality:complexity_avg")
            .and_then(|v| v.as_f64())
            .unwrap_or(10.0);

        let complexity_score = Normalize::sigmoid_reverse(
            complexity,
            self.thresholds.complexity_avg_good,
            self.thresholds.complexity_avg_bad,
        );

        // Duplication score (inverse - lower duplication is better)
        let duplication = metrics.get("quality:duplication_rate")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let duplication_score = 100.0 - (duplication * 2.0).min(100.0);

        // Lint score (already 0-100)
        let lint = metrics.get("quality:lint_score")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        // Type coverage (already 0-100)
        let type_coverage = metrics.get("quality:type_coverage")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        // Weighted formula: complexity 30%, duplication 30%, lint 25%, types 15%
        let score = complexity_score * 0.30 + duplication_score * 0.30 + lint * 0.25 + type_coverage * 0.15;

        let mut dimension = DimensionScore::new("quality".to_string(), score, 0.20);

        dimension.metrics.insert("complexity_avg".to_string(), MetricValue::Float(complexity));
        dimension.metrics.insert("complexity_score".to_string(), MetricValue::Percentage(complexity_score));
        dimension.metrics.insert("duplication_rate".to_string(), MetricValue::Percentage(duplication));
        dimension.metrics.insert("duplication_score".to_string(), MetricValue::Percentage(duplication_score));
        dimension.metrics.insert("lint_score".to_string(), MetricValue::Percentage(lint));
        dimension.metrics.insert("type_coverage".to_string(), MetricValue::Percentage(type_coverage));

        // Generate issues
        if complexity > self.thresholds.complexity_avg_bad {
            dimension.issues.push(ScoreIssue {
                id: "high-complexity".to_string(),
                severity: IssueSeverity::Medium,
                category: "quality".to_string(),
                title: "High cyclomatic complexity".to_string(),
                description: format!("Average complexity is {:.1}, above the threshold of {:.1}", complexity, self.thresholds.complexity_avg_bad),
                file: None,
                line: None,
                column: None,
                impact: (complexity - self.thresholds.complexity_avg_bad) * 2.0,
                suggestion: Some("Refactor complex functions into smaller, simpler ones".to_string()),
            });
        }

        if duplication > self.thresholds.duplication_rate_bad {
            dimension.issues.push(ScoreIssue {
                id: "code-duplication".to_string(),
                severity: IssueSeverity::Medium,
                category: "quality".to_string(),
                title: "High code duplication".to_string(),
                description: format!("Code duplication rate is {:.1}%", duplication),
                file: None,
                line: None,
                column: None,
                impact: duplication * 2.0,
                suggestion: Some("Extract duplicated code into reusable functions".to_string()),
            });
        }

        dimension
    }
}
