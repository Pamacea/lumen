//! Score history management and statistics

use serde::{Deserialize, Serialize};
use super::{Grade, trend::HistoricalScore};
use std::collections::HashMap;

/// Complete score history for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreHistory {
    /// Project name
    pub project_name: String,
    /// All historical scores
    pub scores: Vec<HistoricalScore>,
    /// Computed statistics
    pub statistics: HistoryStatistics,
}

impl ScoreHistory {
    /// Create a new history from scores
    pub fn new(project_name: String, mut scores: Vec<HistoricalScore>) -> Self {
        // Sort by timestamp (newest first)
        scores.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        let statistics = HistoryStatistics::calculate(&scores);

        Self {
            project_name,
            scores,
            statistics,
        }
    }

    /// Add a new score to history
    pub fn add(&mut self, score: HistoricalScore) {
        self.scores.insert(0, score);
        self.statistics = HistoryStatistics::calculate(&self.scores);
    }

    /// Get the latest score
    pub fn latest(&self) -> Option<&HistoricalScore> {
        self.scores.first()
    }

    /// Get the earliest score
    pub fn earliest(&self) -> Option<&HistoricalScore> {
        self.scores.last()
    }

    /// Get scores within a date range
    pub fn range(&self, start: i64, end: i64) -> Vec<&HistoricalScore> {
        self.scores.iter()
            .filter(|s| s.timestamp >= start && s.timestamp <= end)
            .collect()
    }

    /// Get the last N scores
    pub fn last_n(&self, n: usize) -> Vec<&HistoricalScore> {
        self.scores.iter().take(n).collect()
    }

    /// Get scores by grade
    pub fn by_grade(&self, grade: Grade) -> Vec<&HistoricalScore> {
        self.scores.iter()
            .filter(|s| s.grade == grade)
            .collect()
    }

    /// Get dimension trend over time
    pub fn dimension_trend(&self, dimension: &str) -> Vec<(i64, f64)> {
        self.scores.iter()
            .map(|s| {
                let value = match dimension {
                    "coverage" => s.dimensions.coverage,
                    "quality" => s.dimensions.quality,
                    "performance" => s.dimensions.performance,
                    "security" => s.dimensions.security,
                    "seo" => s.dimensions.seo,
                    "docs" => s.dimensions.docs,
                    "uiux" => s.dimensions.uiux,
                    _ => 0.0,
                };
                (s.timestamp, value)
            })
            .collect()
    }
}

/// Statistical summary of score history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryStatistics {
    /// Average score
    pub avg: f64,
    /// Median score
    pub median: f64,
    /// Minimum score
    pub min: f64,
    /// Maximum score
    pub max: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Count of scores
    pub count: usize,
    /// Best commit (SHA)
    pub best_commit: Option<String>,
    /// Worst commit (SHA)
    pub worst_commit: Option<String>,
    /// Grade distribution
    pub grade_distribution: HashMap<String, usize>,
    /// Dimension averages
    pub dimension_avg: DimensionAverages,
    /// Trend direction
    pub trend_direction: TrendDirection,
    /// Improvement rate (points per scan)
    pub improvement_rate: f64,
}

/// Direction of trend
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

/// Average scores per dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionAverages {
    pub coverage: f64,
    pub quality: f64,
    pub performance: f64,
    pub security: f64,
    pub seo: f64,
    pub docs: f64,
    pub uiux: f64,
}

impl HistoryStatistics {
    /// Calculate statistics from historical scores
    pub fn calculate(scores: &[HistoricalScore]) -> Self {
        if scores.is_empty() {
            return Self::empty();
        }

        let all_scores: Vec<f64> = scores.iter().map(|s| s.score).collect();

        let avg = Self::average(&all_scores);
        let median = Self::median(&mut all_scores.clone());
        let min = all_scores.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = all_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let std_dev = Self::std_deviation(&all_scores, avg);
        let count = scores.len();

        let best_idx = all_scores.iter()
            .position(|&s| s == max)
            .unwrap_or(0);
        let worst_idx = all_scores.iter()
            .position(|&s| s == min)
            .unwrap_or(0);

        let grade_distribution = Self::grade_distribution(scores);

        let dimension_avg = Self::calculate_dimension_averages(scores);

        let trend_direction = Self::calculate_trend(scores);
        let improvement_rate = Self::calculate_improvement_rate(scores);

        Self {
            avg,
            median,
            min,
            max,
            std_dev,
            count,
            best_commit: scores.get(best_idx).map(|s| s.commit_sha.clone()),
            worst_commit: scores.get(worst_idx).map(|s| s.commit_sha.clone()),
            grade_distribution,
            dimension_avg,
            trend_direction,
            improvement_rate,
        }
    }

    fn empty() -> Self {
        Self {
            avg: 0.0,
            median: 0.0,
            min: 0.0,
            max: 0.0,
            std_dev: 0.0,
            count: 0,
            best_commit: None,
            worst_commit: None,
            grade_distribution: HashMap::new(),
            dimension_avg: DimensionAverages::default(),
            trend_direction: TrendDirection::Stable,
            improvement_rate: 0.0,
        }
    }

    fn average(values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.iter().sum::<f64>() / values.len() as f64
    }

    fn median(values: &mut [f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let len = values.len();
        if len % 2 == 0 {
            (values[len / 2 - 1] + values[len / 2]) / 2.0
        } else {
            values[len / 2]
        }
    }

    fn std_deviation(values: &[f64], mean: f64) -> f64 {
        if values.len() <= 1 {
            return 0.0;
        }
        let variance = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        variance.sqrt()
    }

    fn grade_distribution(scores: &[HistoricalScore]) -> HashMap<String, usize> {
        let mut dist = HashMap::new();
        for score in scores {
            let grade = score.grade.as_letter().to_string();
            *dist.entry(grade).or_insert(0) += 1;
        }
        dist
    }

    fn calculate_dimension_averages(scores: &[HistoricalScore]) -> DimensionAverages {
        if scores.is_empty() {
            return DimensionAverages::default();
        }

        let sum_coverage: f64 = scores.iter().map(|s| s.dimensions.coverage).sum();
        let sum_quality: f64 = scores.iter().map(|s| s.dimensions.quality).sum();
        let sum_performance: f64 = scores.iter().map(|s| s.dimensions.performance).sum();
        let sum_security: f64 = scores.iter().map(|s| s.dimensions.security).sum();
        let sum_seo: f64 = scores.iter().map(|s| s.dimensions.seo).sum();
        let sum_docs: f64 = scores.iter().map(|s| s.dimensions.docs).sum();
        let sum_uiux: f64 = scores.iter().map(|s| s.dimensions.uiux).sum();

        let n = scores.len() as f64;

        DimensionAverages {
            coverage: sum_coverage / n,
            quality: sum_quality / n,
            performance: sum_performance / n,
            security: sum_security / n,
            seo: sum_seo / n,
            docs: sum_docs / n,
            uiux: sum_uiux / n,
        }
    }

    fn calculate_trend(scores: &[HistoricalScore]) -> TrendDirection {
        if scores.len() < 2 {
            return TrendDirection::Stable;
        }

        let recent_avg: f64 = scores.iter().take(scores.len().min(5))
            .map(|s| s.score)
            .sum::<f64>() / scores.len().min(5) as f64;

        let older_avg: f64 = scores.iter().skip(scores.len().min(5)).take(5)
            .map(|s| s.score)
            .sum::<f64>() / scores.len().saturating_sub(5).min(5).max(1) as f64;

        const THRESHOLD: f64 = 2.0;

        if recent_avg - older_avg > THRESHOLD {
            TrendDirection::Up
        } else if older_avg - recent_avg > THRESHOLD {
            TrendDirection::Down
        } else {
            TrendDirection::Stable
        }
    }

    fn calculate_improvement_rate(scores: &[HistoricalScore]) -> f64 {
        if scores.len() < 2 {
            return 0.0;
        }

        let first = scores.last().map(|s| s.score).unwrap_or(0.0);
        let last = scores.first().map(|s| s.score).unwrap_or(0.0);

        // Simple linear rate: (last - first) / (count - 1)
        (last - first) / (scores.len() as f64 - 1.0)
    }
}

impl Default for DimensionAverages {
    fn default() -> Self {
        Self {
            coverage: 0.0,
            quality: 0.0,
            performance: 0.0,
            security: 0.0,
            seo: 0.0,
            docs: 0.0,
            uiux: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::trend::DimensionSnapshot;

    fn make_score(sha: &str, score: f64) -> HistoricalScore {
        HistoricalScore {
            commit_sha: sha.to_string(),
            timestamp: 0,
            score,
            grade: Grade::from_score(score),
            dimensions: DimensionSnapshot {
                coverage: score,
                quality: score,
                performance: score,
                security: score,
                seo: score,
                docs: score,
                uiux: score,
            },
        }
    }

    #[test]
    fn test_history_statistics() {
        let scores = vec![
            make_score("a", 70.0),
            make_score("b", 80.0),
            make_score("c", 75.0),
            make_score("d", 85.0),
        ];

        let stats = HistoryStatistics::calculate(&scores);

        assert_eq!(stats.count, 4);
        assert!((stats.avg - 77.5).abs() < 0.1);
        assert_eq!(stats.min, 70.0);
        assert_eq!(stats.max, 85.0);
        assert_eq!(stats.best_commit, Some("d".to_string()));
        assert_eq!(stats.worst_commit, Some("a".to_string()));
    }

    #[test]
    fn test_improvement_rate() {
        // Scores are stored from most recent to oldest
        let scores = vec![
            make_score("d", 85.0),  // most recent
            make_score("c", 80.0),
            make_score("b", 75.0),
            make_score("a", 70.0),  // oldest
        ];

        let rate = HistoryStatistics::calculate_improvement_rate(&scores);
        assert_eq!(rate, 5.0); // (85 - 70) / 3 = 15 / 3 = 5
    }

    #[test]
    fn test_trend_direction() {
        // Improving: newest scores (first 5) average higher than older scores
        let improving = vec![
            make_score("j", 90.0),  // most recent
            make_score("i", 88.0),
            make_score("h", 86.0),
            make_score("g", 84.0),
            make_score("f", 82.0),  // recent avg ~86
            make_score("e", 78.0),
            make_score("d", 76.0),
            make_score("c", 74.0),
            make_score("b", 72.0),
            make_score("a", 70.0),  // oldest (older avg ~74)
        ];

        // Declining: newest scores (first 5) average lower than older scores
        let declining = vec![
            make_score("j", 70.0),  // most recent
            make_score("i", 72.0),
            make_score("h", 74.0),
            make_score("g", 76.0),
            make_score("f", 78.0),  // recent avg ~74
            make_score("e", 82.0),
            make_score("d", 84.0),
            make_score("c", 86.0),
            make_score("b", 88.0),
            make_score("a", 90.0),  // oldest (older avg ~86)
        ];

        assert_eq!(HistoryStatistics::calculate_trend(&improving), TrendDirection::Up);
        assert_eq!(HistoryStatistics::calculate_trend(&declining), TrendDirection::Down);
    }
}
