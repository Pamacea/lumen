//! Metric values and normalization functions

use serde::{Deserialize, Serialize};

/// A metric value that can be of different types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MetricValue {
    /// Floating point value
    Float(f64),
    /// Integer value
    Integer(i64),
    /// Percentage value (0-100)
    Percentage(f64),
    /// Duration in milliseconds
    Duration(u64),
    /// Count of items
    Count(usize),
    /// Boolean value
    Boolean(bool),
    /// String value (for categorical data)
    String(String),
}

impl MetricValue {
    /// Get the value as f64 if possible
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            MetricValue::Float(v) => Some(*v),
            MetricValue::Integer(v) => Some(*v as f64),
            MetricValue::Percentage(v) => Some(*v),
            MetricValue::Duration(v) => Some(*v as f64),
            MetricValue::Count(v) => Some(*v as f64),
            MetricValue::Boolean(v) => Some(if *v { 100.0 } else { 0.0 }),
            MetricValue::String(_) => None,
        }
    }

    /// Get the value as a percentage (0-100)
    pub fn as_percentage(&self) -> Option<f64> {
        match self {
            MetricValue::Percentage(v) => Some(*v),
            MetricValue::Boolean(v) => Some(if *v { 100.0 } else { 0.0 }),
            MetricValue::Float(v) if (0.0..=1.0).contains(v) => Some(v * 100.0),
            _ => None,
        }
    }

    /// Get the value as duration (milliseconds)
    pub fn as_duration(&self) -> Option<u64> {
        match self {
            MetricValue::Duration(v) => Some(*v),
            _ => None,
        }
    }

    /// Get the value as a count
    pub fn as_count(&self) -> Option<usize> {
        match self {
            MetricValue::Count(v) => Some(*v),
            MetricValue::Integer(v) if *v >= 0 => Some(*v as usize),
            _ => None,
        }
    }

    /// Get the value as a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            MetricValue::Boolean(v) => Some(*v),
            MetricValue::Count(1) => Some(true),
            MetricValue::Count(0) => Some(false),
            MetricValue::Integer(1) => Some(true),
            MetricValue::Integer(0) => Some(false),
            _ => None,
        }
    }
}

impl From<f64> for MetricValue {
    fn from(v: f64) -> Self {
        MetricValue::Float(v)
    }
}

impl From<i64> for MetricValue {
    fn from(v: i64) -> Self {
        MetricValue::Integer(v)
    }
}

impl From<usize> for MetricValue {
    fn from(v: usize) -> Self {
        MetricValue::Count(v)
    }
}

impl From<bool> for MetricValue {
    fn from(v: bool) -> Self {
        MetricValue::Boolean(v)
    }
}

impl From<String> for MetricValue {
    fn from(v: String) -> Self {
        MetricValue::String(v)
    }
}

impl<'a> From<&'a str> for MetricValue {
    fn from(v: &'a str) -> Self {
        MetricValue::String(v.to_string())
    }
}

/// Normalization functions for converting raw metrics to 0-100 scores
pub struct Normalize;

impl Normalize {
    /// Normalize a value where higher is better (0 -> 0, target -> 100)
    pub fn sigmoid(value: f64, bad_threshold: f64, good_threshold: f64) -> f64 {
        if value <= bad_threshold {
            0.0
        } else if value >= good_threshold {
            100.0
        } else {
            let ratio = (value - bad_threshold) / (good_threshold - bad_threshold);
            ratio * 100.0
        }
    }

    /// Normalize a value where lower is better (good -> 100, bad -> 0)
    pub fn sigmoid_reverse(value: f64, good_threshold: f64, bad_threshold: f64) -> f64 {
        if value <= good_threshold {
            100.0
        } else if value >= bad_threshold {
            0.0
        } else {
            let ratio = (bad_threshold - value) / (bad_threshold - good_threshold);
            ratio * 100.0
        }
    }

    /// Normalize a duration (lower is better)
    pub fn duration_ms(duration_ms: u64, good_ms: f64, bad_ms: f64) -> f64 {
        Self::sigmoid_reverse(duration_ms as f64, good_ms, bad_ms)
    }

    /// Normalize a percentage (already 0-100)
    pub fn percentage(value: f64) -> f64 {
        value.clamp(0.0, 100.0)
    }

    /// Normalize a count to 0-100 based on expected range
    pub fn count(count: usize, expected_max: usize) -> f64 {
        if expected_max == 0 {
            return 0.0;
        }
        let ratio = count as f64 / expected_max as f64;
        (ratio * 100.0).clamp(0.0, 100.0)
    }

    /// Inverse normalize a count (lower is better)
    pub fn count_inverse(count: usize, acceptable: usize, critical: usize) -> f64 {
        if count <= acceptable {
            100.0
        } else if count >= critical {
            0.0
        } else {
            let ratio = (critical - count) as f64 / (critical - acceptable) as f64;
            ratio * 100.0
        }
    }

    /// Normalize based on target value with optimal range
    pub fn optimal_range(value: f64, min_optimal: f64, max_optimal: f64, hard_min: f64, hard_max: f64) -> f64 {
        if (min_optimal..=max_optimal).contains(&value) {
            100.0
        } else if value < hard_min || value > hard_max {
            0.0
        } else if value < min_optimal {
            let ratio = (value - hard_min) / (min_optimal - hard_min);
            ratio * 100.0
        } else {
            // value > max_optimal
            let ratio = (hard_max - value) / (hard_max - max_optimal);
            ratio * 100.0
        }
    }
}

/// Threshold configuration for specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricThresholds {
    // Coverage thresholds
    pub coverage_unit_excellent: f64,
    pub coverage_unit_good: f64,
    pub coverage_integration_good: f64,
    pub coverage_e2e_good: f64,

    // Performance thresholds (milliseconds)
    pub backend_latency_good: u64,
    pub backend_latency_bad: u64,
    pub frontend_lcp_good: u64,
    pub frontend_lcp_bad: u64,
    pub db_query_good: u64,
    pub db_query_bad: u64,

    // Security thresholds
    pub critical_vuln_penalty: f64,
    pub high_vuln_penalty: f64,
    pub medium_vuln_penalty: f64,

    // Quality thresholds
    pub complexity_avg_good: f64,
    pub complexity_avg_bad: f64,
    pub duplication_rate_bad: f64,

    // Documentation thresholds
    pub comment_ratio_good: f64,

    // Bundle size thresholds (KB)
    pub bundle_size_good: usize,
    pub bundle_size_bad: usize,
}

impl Default for MetricThresholds {
    fn default() -> Self {
        Self {
            coverage_unit_excellent: 90.0,
            coverage_unit_good: 75.0,
            coverage_integration_good: 60.0,
            coverage_e2e_good: 40.0,

            backend_latency_good: 100,
            backend_latency_bad: 500,
            frontend_lcp_good: 1200,
            frontend_lcp_bad: 2500,
            db_query_good: 10,
            db_query_bad: 100,

            critical_vuln_penalty: 20.0,
            high_vuln_penalty: 10.0,
            medium_vuln_penalty: 5.0,

            complexity_avg_good: 5.0,
            complexity_avg_bad: 15.0,
            duplication_rate_bad: 5.0,

            comment_ratio_good: 0.10, // 10%

            bundle_size_good: 100,
            bundle_size_bad: 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_sigmoid() {
        assert_eq!(Normalize::sigmoid(0.0, 0.0, 100.0), 0.0);
        assert_eq!(Normalize::sigmoid(100.0, 0.0, 100.0), 100.0);
        assert_eq!(Normalize::sigmoid(50.0, 0.0, 100.0), 50.0);
    }

    #[test]
    fn test_normalize_sigmoid_reverse() {
        assert_eq!(Normalize::sigmoid_reverse(0.0, 0.0, 100.0), 100.0);
        assert_eq!(Normalize::sigmoid_reverse(100.0, 0.0, 100.0), 0.0);
        assert_eq!(Normalize::sigmoid_reverse(50.0, 0.0, 100.0), 50.0);
    }

    #[test]
    fn test_normalize_duration() {
        assert_eq!(Normalize::duration_ms(50, 100.0, 500.0), 100.0);
        assert_eq!(Normalize::duration_ms(500, 100.0, 500.0), 0.0);
        assert_eq!(Normalize::duration_ms(300, 100.0, 500.0), 50.0);
    }

    #[test]
    fn test_metric_value_conversions() {
        let p = MetricValue::Percentage(75.0);
        assert_eq!(p.as_f64(), Some(75.0));
        assert_eq!(p.as_percentage(), Some(75.0));

        let d = MetricValue::Duration(150);
        assert_eq!(d.as_f64(), Some(150.0));
        assert_eq!(d.as_duration(), Some(150));

        let b = MetricValue::Boolean(true);
        assert_eq!(b.as_bool(), Some(true));
        assert_eq!(b.as_f64(), Some(100.0));

        let c = MetricValue::Count(42);
        assert_eq!(c.as_count(), Some(42));
        assert_eq!(c.as_f64(), Some(42.0));
    }

    #[test]
    fn test_optimal_range() {
        // Within optimal range
        assert_eq!(Normalize::optimal_range(50.0, 40.0, 60.0, 0.0, 100.0), 100.0);
        // Below optimal
        assert_eq!(Normalize::optimal_range(20.0, 40.0, 60.0, 0.0, 100.0), 50.0);
        // Above optimal
        assert_eq!(Normalize::optimal_range(80.0, 40.0, 60.0, 0.0, 100.0), 50.0);
        // Outside hard limits
        assert_eq!(Normalize::optimal_range(-10.0, 40.0, 60.0, 0.0, 100.0), 0.0);
        assert_eq!(Normalize::optimal_range(110.0, 40.0, 60.0, 0.0, 100.0), 0.0);
    }
}
