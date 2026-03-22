//! # Watch Module
//!
//! File watching for continuous analysis.

use std::path::PathBuf;
use std::time::Duration;

/// Watch configuration
#[derive(Debug, Clone)]
pub struct WatchConfig {
    pub root: PathBuf,
    pub debounce_duration: Duration,
    pub ignore_patterns: Vec<String>,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("."),
            debounce_duration: Duration::from_millis(300),
            ignore_patterns: vec![
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
            ],
        }
    }
}
