//! # Diff Module
//!
//! Git diff and PR analysis for Oalacea Lumen.

use std::path::PathBuf;

/// Re-exports from core
pub use oalacea_lumen_core::prelude::*;

/// Diff analyzer for Git changes
#[derive(Debug, Clone)]
pub struct DiffAnalyzer {
    pub root: PathBuf,
}

/// Result of diff analysis
#[derive(Debug, Clone)]
pub struct DiffResult {
    /// Commit SHA
    pub commit_sha: String,
    /// Files changed
    pub changed_files: Vec<ChangedFile>,
    /// Lines added
    pub lines_added: usize,
    /// Lines removed
    pub lines_removed: usize,
}

/// A file that was changed
#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: String,
    pub status: FileStatus,
    pub additions: usize,
    pub deletions: usize,
    pub patch: String,
}

/// Status of a changed file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}
