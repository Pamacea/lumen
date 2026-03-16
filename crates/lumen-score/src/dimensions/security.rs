//! Security dimension scorer (15% weight)

use crate::{DimensionScore, ScoreIssue, IssueSeverity};
use crate::metrics::{MetricValue, MetricThresholds};

/// Security scorer
pub struct SecurityScorer {
    thresholds: MetricThresholds,
}

impl SecurityScorer {
    pub fn new(thresholds: MetricThresholds) -> Self {
        Self { thresholds }
    }

    pub fn score(&self, metrics: &std::collections::HashMap<String, MetricValue>) -> DimensionScore {
        // Vulnerability counts (inverse - fewer is better)
        let critical = metrics.get("sec:vuln_critical")
            .and_then(|v| v.as_count())
            .unwrap_or(0);

        let high = metrics.get("sec:vuln_high")
            .and_then(|v| v.as_count())
            .unwrap_or(0);

        let medium = metrics.get("sec:vuln_medium")
            .and_then(|v| v.as_count())
            .unwrap_or(0);

        let vuln_score = 100.0
            - (critical as f64 * self.thresholds.critical_vuln_penalty)
            - (high as f64 * self.thresholds.high_vuln_penalty)
            - (medium as f64 * self.thresholds.medium_vuln_penalty);

        // Insecure patterns
        let insecure_patterns = metrics.get("sec:insecure_patterns")
            .and_then(|v| v.as_count())
            .unwrap_or(0);

        let pattern_score = (100.0 - (insecure_patterns as f64 * 5.0)).max(0.0);

        // Dependency health
        let dep_health = metrics.get("sec:dependency_health")
            .and_then(|v| v.as_percentage())
            .unwrap_or(100.0);

        // Secrets exposure (inverse - fewer is better)
        let secrets = metrics.get("sec:secrets_found")
            .and_then(|v| v.as_count())
            .unwrap_or(0);

        let secrets_score = (100.0 - (secrets as f64 * 25.0)).max(0.0);

        // Weighted formula: vulns 35%, patterns 30%, deps 20%, secrets 15%
        let score = vuln_score * 0.35 + pattern_score * 0.30 + dep_health * 0.20 + secrets_score * 0.15;

        let mut dimension = DimensionScore::new("security".to_string(), score, 0.15);

        dimension.metrics.insert("critical_vulns".to_string(), MetricValue::Count(critical));
        dimension.metrics.insert("high_vulns".to_string(), MetricValue::Count(high));
        dimension.metrics.insert("medium_vulns".to_string(), MetricValue::Count(medium));
        dimension.metrics.insert("vulnerability_score".to_string(), MetricValue::Percentage(vuln_score));
        dimension.metrics.insert("insecure_patterns".to_string(), MetricValue::Count(insecure_patterns));
        dimension.metrics.insert("pattern_score".to_string(), MetricValue::Percentage(pattern_score));
        dimension.metrics.insert("dependency_health".to_string(), MetricValue::Percentage(dep_health));
        dimension.metrics.insert("secrets_found".to_string(), MetricValue::Count(secrets));
        dimension.metrics.insert("secrets_score".to_string(), MetricValue::Percentage(secrets_score));

        // Generate issues
        if critical > 0 {
            dimension.issues.push(ScoreIssue {
                id: "critical-vulnerabilities".to_string(),
                severity: IssueSeverity::Critical,
                category: "security".to_string(),
                title: format!("{} critical vulnerabilities found", critical),
                description: "Critical vulnerabilities must be addressed immediately".to_string(),
                file: None,
                line: None,
                column: None,
                impact: critical as f64 * self.thresholds.critical_vuln_penalty,
                suggestion: Some("Update affected dependencies and patch vulnerabilities immediately".to_string()),
            });
        }

        if secrets > 0 {
            dimension.issues.push(ScoreIssue {
                id: "secrets-exposed".to_string(),
                severity: IssueSeverity::Critical,
                category: "security".to_string(),
                title: format!("{} potential secrets exposed in code", secrets),
                description: "Secrets (API keys, passwords) should never be committed to code".to_string(),
                file: None,
                line: None,
                column: None,
                impact: secrets as f64 * 25.0,
                suggestion: Some("Remove secrets from code and use environment variables".to_string()),
            });
        }

        dimension
    }
}
