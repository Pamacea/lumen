//! Watch Mode for LumenX
//!
//! Monitors project files and automatically re-runs analysis on changes.
//! Supports debouncing, file filtering, and daemon mode.

use lumenx_core::LumenResult;
use notify::{recommended_watcher, RecursiveMode, Watcher, Event, EventKind, Error as NotifyError};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

/// Watch event
#[derive(Debug, Clone)]
pub enum WatchEvent {
    /// File changed
    Changed(PathBuf),
    /// File created
    Created(PathBuf),
    /// File deleted
    Deleted(PathBuf),
    /// Multiple files changed (batch event)
    Batch(Vec<PathBuf>),
}

/// Watch configuration
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// Paths to watch
    pub include_paths: Vec<String>,
    /// Paths to ignore
    pub exclude_paths: Vec<String>,
    /// File patterns to include
    pub include_patterns: Vec<String>,
    /// File patterns to ignore
    pub exclude_patterns: Vec<String>,
    /// Debounce delay (ms)
    pub debounce_ms: u64,
    /// Whether to run on startup
    pub run_on_startup: bool,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            include_paths: vec![".".to_string()],
            exclude_paths: vec![
                "node_modules".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                ".git".to_string(),
                "vendor".to_string(),
            ],
            include_patterns: vec![
                "*.rs".to_string(),
                "*.ts".to_string(),
                "*.tsx".to_string(),
                "*.js".to_string(),
                "*.jsx".to_string(),
                "*.py".to_string(),
                "*.go".to_string(),
                "*.java".to_string(),
                "*.css".to_string(),
                "*.html".to_string(),
            ],
            exclude_patterns: vec![
                "*.test.*".to_string(),
                "*.spec.*".to_string(),
                "*.test.ts".to_string(),
                "*.spec.ts".to_string(),
                "*.test.js".to_string(),
                "*.spec.js".to_string(),
                "*.min.js".to_string(),
                "*.min.css".to_string(),
                "*.d.ts".to_string(),
            ],
            debounce_ms: 300,
            run_on_startup: true,
        }
    }
}

/// Watch mode handler
pub struct WatchHandler {
    /// Configuration
    config: WatchConfig,
    /// Callback for when changes are detected
    callback: Box<dyn Fn(Vec<PathBuf>) + Send>,
    /// Project root
    project_root: PathBuf,
}

impl WatchHandler {
    /// Create a new watch handler
    pub fn new<F>(
        project_root: PathBuf,
        config: WatchConfig,
        callback: F,
    ) -> LumenResult<Self>
    where
        F: Fn(Vec<PathBuf>) + Send + 'static,
    {
        Ok(Self {
            config,
            callback: Box::new(callback),
            project_root,
        })
    }

    /// Start watching for file changes
    pub fn start(&mut self) -> LumenResult<()> {
        let (tx, rx) = channel::<Result<Event, NotifyError>>();

        let mut watcher = recommended_watcher(tx)
            .map_err(|e| lumenx_core::LumenError::WatchError(e.to_string()))?;

        // Add all include paths to watcher
        for path in &self.config.include_paths {
            let full_path = self.project_root.join(path);
            if full_path.exists() {
                watcher.watch(&full_path, RecursiveMode::Recursive)
                    .map_err(|e| lumenx_core::LumenError::WatchError(e.to_string()))?;
            }
        }

        println!("👀 Watching for changes...");
        println!("   Press Ctrl+C to stop");

        // Handle events with debouncing
        let mut pending_files: Vec<PathBuf> = Vec::new();
        let mut debounce_timer = std::time::Instant::now();

        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    if self.should_process_event(&event) {
                        if let Some(path) = self.extract_path(&event) {
                            pending_files.push(path);
                            debounce_timer = std::time::Instant::now();
                        }
                    }
                }
                Ok(Err(e)) => {
                    eprintln!("Watch error: {}", e);
                }
                Err(_) => {
                    // Timeout - check if we should trigger
                    if !pending_files.is_empty()
                        && debounce_timer.elapsed() >= Duration::from_millis(self.config.debounce_ms)
                    {
                        // Deduplicate
                        pending_files.sort();
                        pending_files.dedup();
                        let files = std::mem::take(&mut pending_files);

                        println!("📝 Changed files:");
                        for file in &files {
                            println!("   - {}", file.display());
                        }

                        // Run callback
                        (self.callback)(files);
                    }
                }
            }
        }
    }

    fn should_process_event(&self, event: &Event) -> bool {
        // Check event kind
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {}
            EventKind::Any => return false,
            _ => return false,
        }

        // Extract path
        let path = match event.paths.first() {
            Some(p) => p,
            None => return false,
        };

        // Check exclude paths
        for exclude in &self.config.exclude_paths {
            if path.to_string_lossy().contains(exclude) {
                return false;
            }
        }

        // Check exclude patterns
        for pattern in &self.config.exclude_patterns {
            if self.matches_pattern(path, pattern) {
                return false;
            }
        }

        true
    }

    fn extract_path(&self, event: &Event) -> Option<PathBuf> {
        event.paths.first().cloned()
    }

    fn matches_pattern(&self, path: &Path, pattern: &str) -> bool {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Simple glob pattern matching
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return file_name.starts_with(prefix) && file_name.ends_with(suffix);
            }
        }

        file_name.contains(pattern)
            || path.extension()
                .and_then(|e| e.to_str())
                .map_or(false, |e| pattern.ends_with(e))
    }
}

/// Watch mode stats
#[derive(Debug, Clone)]
pub struct WatchStats {
    /// Number of scans run
    pub scans_run: usize,
    /// Files monitored
    pub files_monitored: usize,
    /// Total changes detected
    pub total_changes: usize,
    /// Average scan duration (ms)
    pub avg_scan_duration_ms: u64,
}
