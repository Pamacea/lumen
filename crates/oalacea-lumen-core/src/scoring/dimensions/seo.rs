//! SEO dimension scorer (10% weight)

use super::super::DimensionScore;
use super::super::metrics::MetricValue;

/// SEO scorer
pub struct SeoScorer;

impl SeoScorer {
    pub fn new() -> Self {
        Self
    }

    pub fn score(&self, metrics: &std::collections::HashMap<String, MetricValue>) -> DimensionScore {
        let meta_coverage = metrics.get("seo:meta_coverage")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let og_coverage = metrics.get("seo:og_coverage")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let structured_data = metrics.get("seo:structured_data")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        let lighthouse_seo = metrics.get("seo:lighthouse")
            .and_then(|v| v.as_percentage())
            .unwrap_or(0.0);

        // Weighted formula: meta 30%, og 25%, structured 25%, lighthouse 20%
        let score = meta_coverage * 0.30 + og_coverage * 0.25 + structured_data * 0.25 + lighthouse_seo * 0.20;

        let mut dimension = DimensionScore::new("seo".to_string(), score, 0.10);

        dimension.metrics.insert("meta_coverage".to_string(), MetricValue::Percentage(meta_coverage));
        dimension.metrics.insert("og_coverage".to_string(), MetricValue::Percentage(og_coverage));
        dimension.metrics.insert("structured_data_coverage".to_string(), MetricValue::Percentage(structured_data));
        dimension.metrics.insert("lighthouse_seo".to_string(), MetricValue::Percentage(lighthouse_seo));

        dimension
    }
}

impl Default for SeoScorer {
    fn default() -> Self {
        Self::new()
    }
}
