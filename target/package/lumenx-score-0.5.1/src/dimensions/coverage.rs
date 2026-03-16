//! Coverage dimension scorer (25% weight)

use crate::{DimensionScore, ScoreIssue, IssueSeverity};
use crate::metrics::{MetricValue, MetricThresholds};

/// Coverage scorer
pub struct CoverageScorer {
    thresholds: MetricThresholds,
}

impl CoverageScorer {
    pub fn new(thresholds: MetricThresholds) -> Self {
        Self { thresholds }
    }

    pub fn score(&self, metrics: &std::collections::HashMap<String, MetricValue>) -> DimensionScore {
        let unit = metrics.get("coverage:unit")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let integration = metrics.get("coverage:integration")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let e2e = metrics.get("coverage:e2e")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        // Weighted formula: unit 40%, integration 35%, e2e 25%
        let score = unit * 0.40 + integration * 0.35 + e2e * 0.25;

        let mut dimension = DimensionScore::new("coverage".to_string(), score, 0.25);

        dimension.metrics.insert("unit_coverage".to_string(), MetricValue::Percentage(unit));
        dimension.metrics.insert("integration_coverage".to_string(), MetricValue::Percentage(integration));
        dimension.metrics.insert("e2e_coverage".to_string(), MetricValue::Percentage(e2e));

        // Generate issues
        if unit < self.thresholds.coverage_unit_good {
            dimension.issues.push(ScoreIssue {
                id: "low-unit-coverage".to_string(),
                severity: if unit < 50.0 { IssueSeverity::Critical } else { IssueSeverity::High },
                category: "coverage".to_string(),
                title: "Low unit test coverage".to_string(),
                description: format!("Unit test coverage is {:.1}%, below the recommended {:.1}%", unit, self.thresholds.coverage_unit_good),
                file: None,
                line: None,
                column: None,
                impact: (self.thresholds.coverage_unit_good - unit) * 0.4,
                suggestion: Some("Add unit tests for uncovered code paths".to_string()),
            });
        }

        dimension
    }
}
