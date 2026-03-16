//! Trend analysis for score history

use serde::{Deserialize, Serialize};
use crate::{Grade, ProjectScore};

/// Historical trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Current score
    pub current: f64,
    /// Previous score (if available)
    pub previous: Option<f64>,
    /// Trend direction
    pub direction: TrendDirection,
    /// Delta from previous score
    pub delta: ScoreDelta,
    /// Moving averages
    pub moving_avg: MovingAverage,
    /// Prediction for future scores
    pub prediction: Option<ScorePrediction>,
}

/// Direction of the trend
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrendDirection {
    /// Score is improving
    Improving,
    /// Score is stable
    Stable,
    /// Score is declining
    Declining,
}

/// Score delta (change from previous)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreDelta {
    /// Overall score change
    pub overall: f64,
    /// Coverage score change
    pub coverage: f64,
    /// Quality score change
    pub quality: f64,
    /// Performance score change
    pub performance: f64,
    /// Security score change
    pub security: f64,
    /// SEO score change
    pub seo: f64,
    /// Documentation score change
    pub docs: f64,
    /// UI/UX score change
    pub uiux: f64,
}

impl ScoreDelta {
    /// Create a zero delta
    pub fn zero() -> Self {
        Self {
            overall: 0.0,
            coverage: 0.0,
            quality: 0.0,
            performance: 0.0,
            security: 0.0,
            seo: 0.0,
            docs: 0.0,
            uiux: 0.0,
        }
    }

    /// Check if the overall change is positive (improving)
    pub fn is_improving(&self) -> bool {
        self.overall > 2.0
    }

    /// Check if the overall change is negative (declining)
    pub fn is_declining(&self) -> bool {
        self.overall < -2.0
    }

    /// Check if the change is significant
    pub fn is_significant(&self) -> bool {
        self.overall.abs() > 2.0
    }

    /// Get the dimension with the largest improvement
    pub fn best_improvement(&self) -> Option<&'static str> {
        let mut best = None;
        let mut max_delta = 0.0;

        let deltas = [
            ("coverage", self.coverage),
            ("quality", self.quality),
            ("performance", self.performance),
            ("security", self.security),
            ("seo", self.seo),
            ("docs", self.docs),
            ("uiux", self.uiux),
        ];

        for (name, delta) in deltas {
            if delta > max_delta {
                max_delta = delta;
                best = Some(name);
            }
        }

        best
    }

    /// Get the dimension with the largest decline
    pub fn worst_decline(&self) -> Option<&'static str> {
        let mut worst = None;
        let mut min_delta = 0.0;

        let deltas = [
            ("coverage", self.coverage),
            ("quality", self.quality),
            ("performance", self.performance),
            ("security", self.security),
            ("seo", self.seo),
            ("docs", self.docs),
            ("uiux", self.uiux),
        ];

        for (name, delta) in deltas {
            if delta < min_delta {
                min_delta = delta;
                worst = Some(name);
            }
        }

        worst
    }
}

/// Moving average statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovingAverage {
    /// 7-period moving average
    pub ma7: f64,
    /// 30-period moving average
    pub ma30: f64,
    /// 90-period moving average
    pub ma90: f64,
}

impl MovingAverage {
    /// Calculate moving averages from a series of scores
    pub fn calculate(scores: &[f64]) -> Self {
        Self {
            ma7: Self::ma(scores, 7),
            ma30: Self::ma(scores, 30),
            ma90: Self::ma(scores, 90),
        }
    }

    fn ma(scores: &[f64], window: usize) -> f64 {
        if scores.is_empty() {
            return 0.0;
        }
        let window = window.min(scores.len());
        let sum: f64 = scores.iter().rev().take(window).sum();
        sum / window as f64
    }

    /// Check if the trend is accelerating (ma7 > ma30 > ma90)
    pub fn is_accelerating(&self) -> bool {
        self.ma7 > self.ma30 && self.ma30 > self.ma90
    }

    /// Check if the trend is decelerating (ma7 < ma30 < ma90)
    pub fn is_decelerating(&self) -> bool {
        self.ma7 < self.ma30 && self.ma30 < self.ma90
    }
}

/// Score prediction based on trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorePrediction {
    /// Predicted score in 7 days
    pub score_7d: f64,
    /// Predicted grade in 7 days
    pub grade_7d: Grade,
    /// Predicted score in 30 days
    pub score_30d: f64,
    /// Predicted grade in 30 days
    pub grade_30d: Grade,
    /// Confidence level (0-1)
    pub confidence: f64,
    /// Velocity (points per day)
    pub velocity: f64,
}

impl ScorePrediction {
    /// Calculate prediction from historical scores
    pub fn calculate(scores: &[f64]) -> Self {
        if scores.len() < 3 {
            return Self::default();
        }

        // Calculate velocity using linear regression
        let n = scores.len() as f64;
        let sum_x: f64 = (0..scores.len()).map(|i| i as f64).sum();
        let sum_y: f64 = scores.iter().sum();
        let sum_xy: f64 = scores.iter().enumerate()
            .map(|(i, y)| i as f64 * y)
            .sum();
        let sum_x2: f64 = (0..scores.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        let intercept = (sum_y - slope * sum_x) / n;

        let velocity = slope;

        // Predict 7 and 30 days ahead
        let score_7d = (intercept + slope * (scores.len() as f64 + 7.0)).clamp(0.0, 100.0);
        let score_30d = (intercept + slope * (scores.len() as f64 + 30.0)).clamp(0.0, 100.0);

        // Calculate confidence based on data variance
        let variance = Self::calculate_variance(scores);
        let confidence = (1.0 / (1.0 + variance / 100.0)).min(1.0).max(0.1);

        Self {
            score_7d,
            grade_7d: Grade::from_score(score_7d),
            score_30d,
            grade_30d: Grade::from_score(score_30d),
            confidence,
            velocity,
        }
    }

    fn calculate_variance(scores: &[f64]) -> f64 {
        if scores.len() < 2 {
            return 0.0;
        }
        let mean = scores.iter().sum::<f64>() / scores.len() as f64;
        scores.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / scores.len() as f64
    }

    /// Check if the prediction is optimistic (improving trend)
    pub fn is_optimistic(&self) -> bool {
        self.velocity > 0.1
    }

    /// Check if the prediction is pessimistic (declining trend)
    pub fn is_pessimistic(&self) -> bool {
        self.velocity < -0.1
    }
}

impl Default for ScorePrediction {
    fn default() -> Self {
        Self {
            score_7d: 50.0,
            grade_7d: Grade::F,
            score_30d: 50.0,
            grade_30d: Grade::F,
            confidence: 0.0,
            velocity: 0.0,
        }
    }
}

impl TrendAnalysis {
    /// Calculate trend analysis from current and historical scores
    pub fn calculate(current: &ProjectScore, history: &[HistoricalScore]) -> Self {
        let current_score = current.overall;
        let previous = history.first().map(|h| h.score);

        let direction = if let Some(prev) = previous {
            let delta = current_score - prev;
            const SIGNIFICANCE: f64 = 2.0;
            if delta > SIGNIFICANCE {
                TrendDirection::Improving
            } else if delta < -SIGNIFICANCE {
                TrendDirection::Declining
            } else {
                TrendDirection::Stable
            }
        } else {
            TrendDirection::Stable
        };

        let delta = if let Some(prev) = previous {
            ScoreDelta {
                overall: current_score - prev,
                coverage: current.dimensions.coverage.score - history.first().map(|h| h.dimensions.coverage).unwrap_or(0.0),
                quality: current.dimensions.quality.score - history.first().map(|h| h.dimensions.quality).unwrap_or(0.0),
                performance: current.dimensions.performance.score - history.first().map(|h| h.dimensions.performance).unwrap_or(0.0),
                security: current.dimensions.security.score - history.first().map(|h| h.dimensions.security).unwrap_or(0.0),
                seo: current.dimensions.seo.score - history.first().map(|h| h.dimensions.seo).unwrap_or(0.0),
                docs: current.dimensions.docs.score - history.first().map(|h| h.dimensions.docs).unwrap_or(0.0),
                uiux: current.dimensions.uiux.score - history.first().map(|h| h.dimensions.uiux).unwrap_or(0.0),
            }
        } else {
            ScoreDelta::zero()
        };

        // Collect all scores for moving average and prediction
        let all_scores: Vec<f64> = std::iter::once(current_score)
            .chain(history.iter().map(|h| h.score))
            .collect();

        let moving_avg = MovingAverage::calculate(&all_scores);
        let prediction = Some(ScorePrediction::calculate(&all_scores));

        Self {
            current: current_score,
            previous,
            direction,
            delta,
            moving_avg,
            prediction,
        }
    }

    /// Get a summary of the trend
    pub fn summary(&self) -> String {
        match self.direction {
            TrendDirection::Improving => {
                format!("↑ Improving (+{:.1} points)", self.delta.overall)
            }
            TrendDirection::Declining => {
                format!("↓ Declining ({:.1} points)", self.delta.overall)
            }
            TrendDirection::Stable => {
                format!("→ Stable ({:+.1} points)", self.delta.overall)
            }
        }
    }

    /// Check if any dimension has a critical decline (>10 points)
    pub fn has_critical_decline(&self) -> Option<&'static str> {
        let deltas = [
            ("coverage", self.delta.coverage),
            ("quality", self.delta.quality),
            ("performance", self.delta.performance),
            ("security", self.delta.security),
            ("seo", self.delta.seo),
            ("docs", self.delta.docs),
            ("uiux", self.delta.uiux),
        ];

        for (name, delta) in deltas {
            if delta < -10.0 {
                return Some(name);
            }
        }

        None
    }
}

/// A single historical score snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalScore {
    /// Commit SHA
    pub commit_sha: String,
    /// Timestamp
    pub timestamp: i64,
    /// Overall score
    pub score: f64,
    /// Grade
    pub grade: Grade,
    /// Individual dimension scores
    pub dimensions: DimensionSnapshot,
}

/// Snapshot of dimension scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionSnapshot {
    pub coverage: f64,
    pub quality: f64,
    pub performance: f64,
    pub security: f64,
    pub seo: f64,
    pub docs: f64,
    pub uiux: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_moving_average() {
        let scores = vec![50.0, 60.0, 70.0, 80.0, 90.0];
        let ma = MovingAverage::calculate(&scores);

        assert_eq!(ma.ma7, 70.0); // All 5 values
        assert_eq!(ma.ma30, 70.0); // All 5 values
    }

    #[test]
    fn test_score_delta() {
        let delta = ScoreDelta {
            overall: 5.0,
            coverage: 10.0,
            quality: -5.0,
            performance: 2.0,
            security: 0.0,
            seo: 3.0,
            docs: -2.0,
            uiux: 1.0,
        };

        assert!(delta.is_improving());
        assert!(!delta.is_declining());
        assert!(delta.is_significant());
        assert_eq!(delta.best_improvement(), Some("coverage"));
        assert_eq!(delta.worst_decline(), Some("quality"));
    }

    #[test]
    fn test_score_prediction() {
        let scores = vec![70.0, 72.0, 74.0, 76.0, 78.0];
        let prediction = ScorePrediction::calculate(&scores);

        assert!(prediction.is_optimistic());
        assert!(!prediction.is_pessimistic());
        assert!(prediction.velocity > 0.0);
        assert!(prediction.score_30d > prediction.score_7d);
    }
}
