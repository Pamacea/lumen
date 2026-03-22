//! # History Module
//!
//! Score history and trend tracking using SQLite.

use std::path::PathBuf;

/// Re-exports from core
pub use oalacea_lumen_core::{scoring::*, LumenResult as Result};

/// Score history manager
pub struct ScoreHistory {
    db_path: PathBuf,
}

/// Historical score entry
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub project_name: String,
    pub commit_sha: String,
    pub timestamp: i64,
    pub overall_score: f64,
    pub grade: Grade,
}

/// Trend analysis result
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub direction: TrendDirection,
    pub average_delta: f64,
    pub entries: Vec<HistoryEntry>,
}

/// Direction of score trend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
}
