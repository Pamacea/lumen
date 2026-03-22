//! Main score calculator with 7 dimensions

use super::{
    DimensionScores, DimensionScore, ProjectScore, ScoreMetadata,
    metrics::{MetricValue, MetricThresholds},
};
use std::collections::HashMap;

/// Score weights for each dimension
#[derive(Debug, Clone)]
pub struct ScoreWeights {
    pub coverage: f64,
    pub quality: f64,
    pub performance: f64,
    pub security: f64,
    pub seo: f64,
    pub docs: f64,
    pub uiux: f64,
}

impl Default for ScoreWeights {
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

/// Main score calculator
pub struct ScoreCalculator {
    #[allow(dead_code)]
    thresholds: MetricThresholds,
    weights: ScoreWeights,
}

impl ScoreCalculator {
    /// Create a new calculator with default thresholds
    pub fn new() -> Self {
        Self {
            thresholds: MetricThresholds::default(),
            weights: ScoreWeights::default(),
        }
    }

    /// Create a calculator with custom thresholds
    pub fn with_thresholds(thresholds: MetricThresholds) -> Self {
        Self {
            thresholds,
            weights: ScoreWeights::default(),
        }
    }

    /// Create a calculator with custom thresholds and weights
    pub fn with_thresholds_and_weights(thresholds: MetricThresholds, weights: ScoreWeights) -> Self {
        Self {
            thresholds,
            weights,
        }
    }

    /// Create from lumen_core config (for backwards compatibility)
    pub fn with_config(config: crate::core::config::ScoringConfig) -> Self {
        Self {
            thresholds: MetricThresholds::default(),
            weights: ScoreWeights {
                coverage: config.weights.coverage,
                quality: config.weights.quality,
                performance: config.weights.performance,
                security: config.weights.security,
                seo: config.weights.seo,
                docs: config.weights.docs,
                uiux: config.weights.uiux,
            },
        }
    }

    /// Calculate the overall project score from collected metrics
    pub fn calculate(
        &self,
        project_name: String,
        commit_sha: String,
        metrics: &HashMap<String, MetricValue>,
        metadata: ScoreMetadata,
    ) -> ProjectScore {
        let dimensions = self.calculate_dimensions(metrics);
        let overall = dimensions.weighted_sum();
        let grade = crate::Grade::from_score(overall);

        ProjectScore {
            project_name,
            commit_sha,
            timestamp: chrono::Utc::now().timestamp(),
            overall,
            grade,
            dimensions,
            trend: None,
            metadata,
        }
    }

    /// Calculate all dimension scores
    fn calculate_dimensions(&self, metrics: &HashMap<String, MetricValue>) -> DimensionScores {
        DimensionScores {
            coverage: self.calculate_coverage(metrics),
            quality: self.calculate_quality(metrics),
            performance: self.calculate_performance(metrics),
            security: self.calculate_security(metrics),
            seo: self.calculate_seo(metrics),
            docs: self.calculate_docs(metrics),
            uiux: self.calculate_uiux(metrics),
        }
    }

    fn calculate_coverage(&self, metrics: &HashMap<String, MetricValue>) -> DimensionScore {
        let unit = metrics.get("coverage:unit")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let integration = metrics.get("coverage:integration")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let e2e = metrics.get("coverage:e2e")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let score = unit * 0.40 + integration * 0.35 + e2e * 0.25;

        DimensionScore::new("coverage".to_string(), score, self.weights.coverage)
    }

    fn calculate_quality(&self, metrics: &HashMap<String, MetricValue>) -> DimensionScore {
        let lint = metrics.get("quality:lint_score")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let type_coverage = metrics.get("quality:type_coverage")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let score = (lint + type_coverage) / 2.0;

        DimensionScore::new("quality".to_string(), score, self.weights.quality)
    }

    fn calculate_performance(&self, metrics: &HashMap<String, MetricValue>) -> DimensionScore {
        let backend_latency = metrics.get("performance:backend_latency_ms")
            .and_then(|v| v.as_duration())
            .unwrap_or(1000);

        let score = if backend_latency < 200 {
            100.0
        } else {
            100.0 - ((backend_latency - 200) as f64 / 10.0).min(100.0)
        };

        DimensionScore::new("performance".to_string(), score, self.weights.performance)
    }

    fn calculate_security(&self, metrics: &HashMap<String, MetricValue>) -> DimensionScore {
        let critical = metrics.get("security:critical_vulns")
            .and_then(|v| v.as_count())
            .unwrap_or(0);

        let score = if critical > 0 {
            0.0
        } else {
            100.0
        };

        DimensionScore::new("security".to_string(), score, self.weights.security)
    }

    fn calculate_seo(&self, metrics: &HashMap<String, MetricValue>) -> DimensionScore {
        let meta_coverage = metrics.get("seo:meta_coverage")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        DimensionScore::new("seo".to_string(), meta_coverage, self.weights.seo)
    }

    fn calculate_docs(&self, metrics: &HashMap<String, MetricValue>) -> DimensionScore {
        let readme_score = metrics.get("docs:readme_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        DimensionScore::new("docs".to_string(), readme_score, self.weights.docs)
    }

    fn calculate_uiux(&self, metrics: &HashMap<String, MetricValue>) -> DimensionScore {
        let accessibility = metrics.get("uiux:accessibility_score")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        DimensionScore::new("uiux".to_string(), accessibility, self.weights.uiux)
    }
}

impl Default for ScoreCalculator {
    fn default() -> Self {
        Self::new()
    }
}
