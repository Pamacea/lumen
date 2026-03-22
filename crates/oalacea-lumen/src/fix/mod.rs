//! # Fix Module
//!
//! Automatic code fixes for Oalacea Lumen.

use std::path::PathBuf;

/// Auto-fix configuration
#[derive(Debug, Clone)]
pub struct FixConfig {
    pub root: PathBuf,
    pub dry_run: bool,
    pub strategy: FixStrategy,
}

/// Fix strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixStrategy {
    Safe,
    Aggressive,
    Interactive,
}

/// Auto-fixer
pub struct AutoFixer {
    config: FixConfig,
}

/// Result of applying fixes
#[derive(Debug, Clone)]
pub struct FixResult {
    pub file: String,
    pub fixes_applied: usize,
    pub patches: Vec<Patch>,
}

/// A code patch
#[derive(Debug, Clone)]
pub struct Patch {
    pub file: String,
    pub old_content: String,
    pub new_content: String,
    pub line_start: usize,
    pub line_end: usize,
}
