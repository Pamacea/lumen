//! UI/UX dimension scorer (10% weight)

use super::super::{DimensionScore, ScoreIssue, IssueSeverity};
use super::super::metrics::MetricValue;

/// UI/UX scorer
pub struct UiuxScorer;

impl UiuxScorer {
    pub fn new() -> Self {
        Self
    }

    pub fn score(&self, metrics: &std::collections::HashMap<String, MetricValue>) -> DimensionScore {
        let layout_quality = metrics.get("uiux:layout_quality")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let responsive = metrics.get("uiux:responsive_score")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let accessibility = metrics.get("uiux:accessibility")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let design_consistency = metrics.get("uiux:design_consistency")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        // Component reuse rate
        let component_reuse = metrics.get("uiux:component_reuse")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        // Weighted formula: layout 25%, responsive 20%, a11y 25%, design 15%, reuse 15%
        let score = layout_quality * 0.25 + responsive * 0.20 + accessibility * 0.25
            + design_consistency * 0.15 + component_reuse * 0.15;

        let mut dimension = DimensionScore::new("uiux".to_string(), score, 0.10);

        dimension.metrics.insert("layout_quality".to_string(), MetricValue::Float(layout_quality));
        dimension.metrics.insert("responsive_score".to_string(), MetricValue::Percentage(responsive));
        dimension.metrics.insert("accessibility_score".to_string(), MetricValue::Percentage(accessibility));
        dimension.metrics.insert("design_consistency".to_string(), MetricValue::Float(design_consistency));
        dimension.metrics.insert("component_reuse".to_string(), MetricValue::Percentage(component_reuse));

        if accessibility < 70.0 {
            dimension.issues.push(ScoreIssue {
                id: "poor-accessibility".to_string(),
                severity: IssueSeverity::High,
                category: "uiux".to_string(),
                title: "Poor accessibility score".to_string(),
                description: format!("Accessibility score is {:.1}%, below recommended 70%", accessibility),
                file: None,
                line: None,
                column: None,
                impact: 15.0,
                suggestion: Some("Improve ARIA labels, keyboard navigation, and color contrast".to_string()),
            });
        }

        if responsive < 70.0 {
            dimension.issues.push(ScoreIssue {
                id: "poor-responsive".to_string(),
                severity: IssueSeverity::Medium,
                category: "uiux".to_string(),
                title: "Poor responsive design".to_string(),
                description: "Site doesn't display well on mobile devices".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 10.0,
                suggestion: Some("Add responsive breakpoints and test on mobile devices".to_string()),
            });
        }

        dimension
    }
}

impl Default for UiuxScorer {
    fn default() -> Self {
        Self::new()
    }
}
