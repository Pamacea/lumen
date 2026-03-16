//! Performance dimension scorer (15% weight)

use crate::{DimensionScore, ScoreIssue, IssueSeverity};
use crate::metrics::{MetricValue, Normalize, MetricThresholds};

/// Performance scorer
pub struct PerformanceScorer {
    thresholds: MetricThresholds,
}

impl PerformanceScorer {
    pub fn new(thresholds: MetricThresholds) -> Self {
        Self { thresholds }
    }

    pub fn score(&self, metrics: &std::collections::HashMap<String, MetricValue>) -> DimensionScore {
        // Backend latency (lower is better)
        let backend_latency = metrics.get("perf:backend_latency")
            .and_then(|v| v.as_duration())
            .unwrap_or(0) as f64;

        let backend_score = Normalize::duration_ms(
            backend_latency as u64,
            self.thresholds.backend_latency_good as f64,
            self.thresholds.backend_latency_bad as f64,
        );

        // Frontend LCP (lower is better)
        let lcp = metrics.get("perf:lcp")
            .and_then(|v| v.as_duration())
            .unwrap_or(0) as f64;

        let lcp_score = Normalize::duration_ms(
            lcp as u64,
            self.thresholds.frontend_lcp_good as f64,
            self.thresholds.frontend_lcp_bad as f64,
        );

        // Bundle size (lower is better)
        let bundle_kb = metrics.get("perf:bundle_size")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let bundle_score = Normalize::sigmoid_reverse(
            bundle_kb,
            self.thresholds.bundle_size_good as f64,
            self.thresholds.bundle_size_bad as f64,
        );

        // DB query time (lower is better)
        let db_time = metrics.get("perf:db_query")
            .and_then(|v| v.as_duration())
            .unwrap_or(0) as f64;

        let db_score = Normalize::duration_ms(
            db_time as u64,
            self.thresholds.db_query_good as f64,
            self.thresholds.db_query_bad as f64,
        );

        // Weighted formula: backend 40%, lcp 30%, bundle 15%, db 15%
        let score = backend_score * 0.40 + lcp_score * 0.30 + bundle_score * 0.15 + db_score * 0.15;

        let mut dimension = DimensionScore::new("performance".to_string(), score, 0.15);

        dimension.metrics.insert("backend_latency_ms".to_string(), MetricValue::Duration(backend_latency as u64));
        dimension.metrics.insert("backend_score".to_string(), MetricValue::Percentage(backend_score));
        dimension.metrics.insert("lcp_ms".to_string(), MetricValue::Duration(lcp as u64));
        dimension.metrics.insert("lcp_score".to_string(), MetricValue::Percentage(lcp_score));
        dimension.metrics.insert("bundle_kb".to_string(), MetricValue::Float(bundle_kb));
        dimension.metrics.insert("bundle_score".to_string(), MetricValue::Percentage(bundle_score));
        dimension.metrics.insert("db_query_ms".to_string(), MetricValue::Duration(db_time as u64));
        dimension.metrics.insert("db_score".to_string(), MetricValue::Percentage(db_score));

        // Generate issues
        if backend_latency > self.thresholds.backend_latency_bad as f64 {
            dimension.issues.push(ScoreIssue {
                id: "slow-backend".to_string(),
                severity: IssueSeverity::High,
                category: "performance".to_string(),
                title: "Slow backend response time".to_string(),
                description: format!("Backend latency is {:.0}ms, above the threshold of {}ms", backend_latency, self.thresholds.backend_latency_bad),
                file: None,
                line: None,
                column: None,
                impact: 15.0,
                suggestion: Some("Optimize database queries, add caching, or consider scaling".to_string()),
            });
        }

        dimension
    }
}
