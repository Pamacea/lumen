//! LumenX History - Score History and Trend Tracking
//!
//! Stores analysis scores in SQLite for:
//! - Historical trend analysis
//! - Branch comparison
//! - Progress tracking over time
//! - Regression detection
//!
//! # Example
//!
//! ```no_run
//! use lumenx_history::HistoryManager;
//! use lumenx_score::ScoringResult;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let history = HistoryManager::new(".lumen/history.db")?;
//!
//! // Store a score
//! # let score = ScoringResult::default();
//! # let project_path = std::path::PathBuf::from(".");
//! history.save_score(&project_path, &score)?;
//!
//! // Get trend
//! let trend = history.get_trend(&project_path, 30)?;
//! println!("30-day trend: {:?}", trend);
//! # Ok(())
//! # }
//! ```

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// History manager errors
#[derive(Error, Debug)]
pub enum HistoryError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("No history found")]
    NoHistory,
}

/// Score history entry
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScoreEntry {
    /// Unique ID
    pub id: i64,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Project path (hashed for privacy)
    pub project_hash: String,
    /// Branch name
    pub branch: Option<String>,
    /// Commit hash
    pub commit: Option<String>,
    /// Overall score
    pub overall_score: f64,
    /// Grade
    pub grade: String,
    /// Dimension scores (JSON)
    pub dimension_scores: String,
    /// Analysis duration (ms)
    pub duration_ms: u64,
    /// Files scanned
    pub files_scanned: usize,
    /// Lines of code
    pub lines_of_code: usize,
    /// Issues count by severity
    pub issues_critical: usize,
    pub issues_high: usize,
    pub issues_medium: usize,
    pub issues_low: usize,
}

/// Trend analysis result
#[derive(Debug, Clone, serde::Serialize)]
pub struct TrendAnalysis {
    /// Current score
    pub current_score: f64,
    /// Previous score (for comparison)
    pub previous_score: Option<f64>,
    /// Score change
    pub delta: f64,
    /// Percentage change
    pub delta_percent: f64,
    /// Trend direction
    pub direction: TrendDirection,
    /// Entries in trend
    pub entry_count: usize,
    /// Average score over period
    pub average_score: f64,
    /// Highest score in period
    pub highest_score: f64,
    /// Lowest score in period
    pub lowest_score: f64,
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Declining,
    Unknown,
}

/// History manager
pub struct HistoryManager {
    /// Database connection
    conn: Connection,
    /// Database file path
    db_path: PathBuf,
}

impl HistoryManager {
    /// Create or open a history database
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, HistoryError> {
        let db_path = db_path.as_ref().to_path_buf();

        // Create parent directory if needed
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrent access
        conn.execute("PRAGMA journal_mode=WAL", [])?;

        // Initialize schema
        let manager = Self { conn, db_path };
        manager.init_schema()?;

        Ok(manager)
    }

    /// Get default history database path
    pub fn default_path() -> Result<PathBuf, HistoryError> {
        let mut path = std::env::current_dir()?;
        path.push(".lumen");
        path.push("history.db");
        Ok(path)
    }

    /// Open default history database
    pub fn open_default() -> Result<Self, HistoryError> {
        Self::new(Self::default_path()?)
    }

    fn init_schema(&self) -> Result<(), HistoryError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS score_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                project_hash TEXT NOT NULL,
                branch TEXT,
                commit TEXT,
                overall_score REAL NOT NULL,
                grade TEXT NOT NULL,
                dimension_scores TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                files_scanned INTEGER NOT NULL,
                lines_of_code INTEGER NOT NULL,
                issues_critical INTEGER NOT NULL DEFAULT 0,
                issues_high INTEGER NOT NULL DEFAULT 0,
                issues_medium INTEGER NOT NULL DEFAULT 0,
                issues_low INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        // Create index for faster queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_timestamp
             ON score_history(timestamp DESC)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_project_hash
             ON score_history(project_hash, timestamp DESC)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_branch
             ON score_history(project_hash, branch, timestamp DESC)",
            [],
        )?;

        Ok(())
    }

    /// Save a scoring result to history
    pub fn save_score(
        &self,
        project_path: &Path,
        score: &ScoringResult,
    ) -> Result<i64, HistoryError> {
        let project_hash = self.hash_project_path(project_path);
        let timestamp = Utc::now();

        // Convert dimension scores to JSON
        let dimension_scores = serde_json::to_string(&score.dimensions)?;

        self.conn.execute(
            "INSERT INTO score_history (
                timestamp, project_hash, branch, commit,
                overall_score, grade, dimension_scores,
                duration_ms, files_scanned, lines_of_code,
                issues_critical, issues_high, issues_medium, issues_low
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                timestamp.to_rfc3339(),
                project_hash,
                score.branch,
                score.commit,
                score.overall_score,
                &score.grade,
                dimension_scores,
                score.duration_ms,
                score.files_scanned,
                score.lines_of_code,
                score.issues_critical,
                score.issues_high,
                score.issues_medium,
                score.issues_low,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Get score history for a project
    pub fn get_history(
        &self,
        project_path: &Path,
        limit: Option<usize>,
    ) -> Result<Vec<ScoreEntry>, HistoryError> {
        let project_hash = self.hash_project_path(project_path);
        let limit = limit.unwrap_or(100);

        let mut stmt = self.conn.prepare(
            "SELECT * FROM score_history
             WHERE project_hash = ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        )?;

        let entries = stmt.query_map(params![project_hash, limit], |row| {
            Ok(ScoreEntry {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                project_hash: row.get(2)?,
                branch: row.get(3)?,
                commit: row.get(4)?,
                overall_score: row.get(5)?,
                grade: row.get(6)?,
                dimension_scores: row.get(7)?,
                duration_ms: row.get(8)?,
                files_scanned: row.get(9)?,
                lines_of_code: row.get(10)?,
                issues_critical: row.get(11)?,
                issues_high: row.get(12)?,
                issues_medium: row.get(13)?,
                issues_low: row.get(14)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Get trend analysis for a project
    pub fn get_trend(
        &self,
        project_path: &Path,
        days: usize,
    ) -> Result<TrendAnalysis, HistoryError> {
        let project_hash = self.hash_project_path(project_path);
        let since = Utc::now() - chrono::Duration::days(days as i64);

        let mut stmt = self.conn.prepare(
            "SELECT overall_score FROM score_history
             WHERE project_hash = ?1
             AND timestamp >= ?2
             ORDER BY timestamp ASC"
        )?;

        let scores: Vec<f64> = stmt.query_map(params![project_hash, since.to_rfc3339()], |row| {
            row.get(0)
        })?.collect::<Result<Vec<_>, _>>()?;

        if scores.is_empty() {
            return Ok(TrendAnalysis {
                current_score: 0.0,
                previous_score: None,
                delta: 0.0,
                delta_percent: 0.0,
                direction: TrendDirection::Unknown,
                entry_count: 0,
                average_score: 0.0,
                highest_score: 0.0,
                lowest_score: 0.0,
            });
        }

        let current_score = *scores.last().unwrap_or(&0.0);
        let previous_score = scores.first().copied();
        let delta = current_score - previous_score.unwrap_or(current_score);
        let delta_percent = if previous_score.unwrap_or(1.0) > 0.0 {
            (delta / previous_score.unwrap_or(1.0)) * 100.0
        } else {
            0.0
        };

        let direction = if delta.abs() < 1.0 {
            TrendDirection::Stable
        } else if delta > 0.0 {
            TrendDirection::Improving
        } else {
            TrendDirection::Declining
        };

        let highest_score = scores.iter().cloned().fold(0.0_f64, f64::max);
        let lowest_score = scores.iter().cloned().fold(100.0_f64, f64::min);
        let average_score = if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        Ok(TrendAnalysis {
            current_score,
            previous_score,
            delta,
            delta_percent,
            direction,
            entry_count: scores.len(),
            average_score,
            highest_score,
            lowest_score,
        })
    }

    /// Compare scores between branches
    pub fn compare_branches(
        &self,
        project_path: &Path,
        branch1: &str,
        branch2: &str,
    ) -> Result<BranchComparison, HistoryError> {
        let project_hash = self.hash_project_path(project_path);

        let get_latest_score = |branch: &str| -> Result<Option<f64>, HistoryError> {
            let mut stmt = self.conn.prepare(
                "SELECT overall_score FROM score_history
                 WHERE project_hash = ?1 AND branch = ?2
                 ORDER BY timestamp DESC LIMIT 1"
            )?;

            let result = stmt.query_row(params![project_hash, branch], |row| row.get(0)).optional()?;
            Ok(result)
        };

        let score1 = get_latest_score(branch1)?;
        let score2 = get_latest_score(branch2)?;

        Ok(BranchComparison {
            branch1: branch1.to_string(),
            branch2: branch2.to_string(),
            score1,
            score2,
            delta: score2.unwrap_or(0.0) - score1.unwrap_or(0.0),
        })
    }

    /// Get regression detection (scores that decreased significantly)
    pub fn detect_regressions(
        &self,
        project_path: &Path,
        threshold: f64,
        limit: usize,
    ) -> Result<Vec<Regression>, HistoryError> {
        let project_hash = self.hash_project_path(project_path);

        let mut stmt = self.conn.prepare(
            "SELECT
                timestamp,
                overall_score,
                grade,
                branch,
                commit
             FROM score_history
             WHERE project_hash = ?1
             ORDER BY timestamp DESC
             LIMIT ?2"
        )?;

        let entries: Vec<(DateTime<Utc>, f64, String, Option<String>, Option<String>)>
            = stmt.query_map(params![project_hash, limit * 2], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })?.collect::<Result<Vec<_>, _>>()?;

        let mut regressions = vec![];

        for window in entries.windows(2) {
            let (prev_ts, prev_score, _, _, _) = &window[1];
            let (curr_ts, curr_score, _, _, _) = &window[0];

            let delta = prev_score - curr_score;
            if delta >= threshold {
                regressions.push(Regression {
                    from: *prev_ts,
                    to: *curr_ts,
                    previous_score: *prev_score,
                    current_score: *curr_score,
                    drop: delta,
                });
            }
        }

        Ok(regressions)
    }

    /// Clean old history entries
    pub fn cleanup_old(&self, project_path: &Path, keep_days: usize) -> Result<usize, HistoryError> {
        let project_hash = self.hash_project_path(project_path);
        let cutoff = Utc::now() - chrono::Duration::days(keep_days as i64);

        let count = self.conn.execute(
            "DELETE FROM score_history
             WHERE project_hash = ?1 AND timestamp < ?2",
            params![project_hash, cutoff.to_rfc3339()],
        )?;

        Ok(count)
    }

    /// Get statistics about stored history
    pub fn get_stats(&self) -> Result<HistoryStats, HistoryError> {
        let total_entries: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM score_history",
            [],
            |row| row.get(0),
        )?;

        let unique_projects: i64 = self.conn.query_row(
            "SELECT COUNT(DISTINCT project_hash) FROM score_history",
            [],
            |row| row.get(0),
        )?;

        let oldest_entry: Option<String> = self.conn.query_row(
            "SELECT MIN(timestamp) FROM score_history",
            [],
            |row| row.get(0),
        )?;

        let newest_entry: Option<String> = self.conn.query_row(
            "SELECT MAX(timestamp) FROM score_history",
            [],
            |row| row.get(0),
        )?;

        Ok(HistoryStats {
            total_entries: total_entries as usize,
            unique_projects: unique_projects as usize,
            oldest_entry,
            newest_entry,
        })
    }

    /// Hash project path for privacy
    fn hash_project_path(&self, path: &Path) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let path_str = path.to_string_lossy();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        path_str.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Branch comparison result
#[derive(Debug, Clone, serde::Serialize)]
pub struct BranchComparison {
    /// First branch name
    pub branch1: String,
    /// Second branch name
    pub branch2: String,
    /// Score of branch 1
    pub score1: Option<f64>,
    /// Score of branch 2
    pub score2: Option<f64>,
    /// Score difference (branch2 - branch1)
    pub delta: f64,
}

/// Regression detected
#[derive(Debug, Clone, serde::Serialize)]
pub struct Regression {
    /// From timestamp
    pub from: DateTime<Utc>,
    /// To timestamp
    pub to: DateTime<Utc>,
    /// Previous score
    pub previous_score: f64,
    /// Current score
    pub current_score: f64,
    /// Score drop
    pub drop: f64,
}

/// History statistics
#[derive(Debug, Clone, serde::Serialize)]
pub struct HistoryStats {
    /// Total entries stored
    pub total_entries: usize,
    /// Number of unique projects
    pub unique_projects: usize,
    /// Oldest entry timestamp
    pub oldest_entry: Option<String>,
    /// Newest entry timestamp
    pub newest_entry: Option<String>,
}

/// Simple scoring result for history storage
#[derive(Debug, Clone, Default)]
pub struct ScoringResult {
    /// Overall score
    pub overall_score: f64,
    /// Grade
    pub grade: String,
    /// Dimension scores
    pub dimensions: std::collections::HashMap<String, f64>,
    /// Analysis duration in ms
    pub duration_ms: u64,
    /// Files scanned
    pub files_scanned: usize,
    /// Lines of code
    pub lines_of_code: usize,
    /// Critical issues
    pub issues_critical: usize,
    /// High issues
    pub issues_high: usize,
    /// Medium issues
    pub issues_medium: usize,
    /// Low issues
    pub issues_low: usize,
    /// Branch name
    pub branch: Option<String>,
    /// Commit hash
    pub commit: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_project_path() {
        let history = HistoryManager::new(":memory:").unwrap();

        let path1 = PathBuf::from("/home/user/project");
        let path2 = PathBuf::from("/home/user/project");
        let path3 = PathBuf::from("/home/user/other");

        assert_eq!(history.hash_project_path(&path1), history.hash_project_path(&path2));
        assert_ne!(history.hash_project_path(&path1), history.hash_project_path(&path3));
    }
}
