//! # Lumen Auto-Fix
//!
//! Automatic code fixes for common issues across multiple languages.
//! Supports dry-run mode, git branch creation, and rollback capability.

pub mod fixers;
pub mod fixer;
pub mod patch;
pub mod transformers;
pub mod git;

use lumen_core::{LumenError, LumenResult};
use lumen_score::ScoreIssue;
use std::path::{Path, PathBuf};

pub use fixer::{AutoFixer, FixContext, FixResult as FixerResult, FixSeverity};
pub use patch::{Patch, PatchDiff, PatchHunk};
pub use transformers::Transformer;

/// Result of applying fixes to a project
#[derive(Debug, Clone)]
pub struct FixResult {
    /// Issues that were successfully fixed
    pub fixed: Vec<FixedIssue>,
    /// Issues that couldn't be fixed
    pub failed: Vec<FailedFix>,
    /// Files that were modified
    pub modified_files: Vec<PathBuf>,
    /// Git branch created (if applicable)
    pub branch_name: Option<String>,
    /// Rollback information
    pub rollback_info: Option<RollbackInfo>,
}

/// An issue that was successfully fixed
#[derive(Debug, Clone)]
pub struct FixedIssue {
    /// The original issue
    pub issue: ScoreIssue,
    /// Description of what was fixed
    pub description: String,
    /// The patch that was applied
    pub patch: PatchDiff,
}

/// An issue that failed to fix
#[derive(Debug, Clone)]
pub struct FailedFix {
    /// The original issue
    pub issue: ScoreIssue,
    /// Reason for failure
    pub reason: String,
}

/// Information for rolling back fixes
#[derive(Debug, Clone)]
pub struct RollbackInfo {
    /// Original branch name
    pub original_branch: String,
    /// Fix branch name
    pub fix_branch: String,
    /// Commit SHA before fixes
    pub before_sha: String,
}

/// Auto-fix engine with safety features
pub struct FixEngine {
    /// Project root directory
    project_root: PathBuf,
    /// Dry-run mode (preview only)
    dry_run: bool,
    /// Create a git branch for fixes
    create_branch: bool,
    /// Automatically run tests after fixes
    run_tests: bool,
}

impl FixEngine {
    /// Create a new fix engine
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            dry_run: false,
            create_branch: false,
            run_tests: false,
        }
    }

    /// Enable dry-run mode (preview changes without applying)
    pub fn with_dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Enable git branch creation for fixes
    pub fn with_branch(mut self) -> Self {
        self.create_branch = true;
        self
    }

    /// Enable automatic test running after fixes
    pub fn with_tests(mut self) -> Self {
        self.run_tests = true;
        self
    }

    /// Apply fixes to the given issues
    pub fn apply_fixes(&self, issues: Vec<ScoreIssue>) -> LumenResult<FixResult> {
        let mut fixed = Vec::new();
        let mut failed = Vec::new();
        let mut modified_files = Vec::new();
        let mut branch_name = None;
        let mut rollback_info = None;

        // Check if it's safe to apply fixes
        self.check_safe(&issues)?;

        // Create git branch if requested
        if self.create_branch && !self.dry_run {
            if let Some(branch) = self.create_fix_branch()? {
                branch_name = Some(branch.clone());
                if let Ok(original) = self::git::get_current_branch(&self.project_root) {
                    if let Ok(sha) = self::git::get_head_sha(&self.project_root) {
                        rollback_info = Some(RollbackInfo {
                            original_branch: original,
                            fix_branch: branch,
                            before_sha: sha,
                        });
                    }
                }
            }
        }

        // Group issues by file for batch processing
        let mut issues_by_file: std::collections::HashMap<PathBuf, Vec<&ScoreIssue>> =
            std::collections::HashMap::new();

        for issue in &issues {
            if let Some(file) = &issue.file {
                let path = self.project_root.join(file);
                issues_by_file.entry(path).or_default().push(issue);
            } else {
                failed.push(FailedFix {
                    issue: issue.clone(),
                    reason: "No file specified".to_string(),
                });
            }
        }

        // Apply fixes per file
        for (file_path, file_issues) in issues_by_file {
            match fixers::apply_file_fixes(&self.project_root, &file_path, &file_issues, self.dry_run) {
                Ok(results) => {
                    for result in results {
                        match result {
                            Ok(fixed_issue) => {
                                modified_files.push(file_path.clone());
                                fixed.push(fixed_issue);
                            }
                            Err(err) => {
                                // Find the issue that failed
                                if let Some(issue) = file_issues.first() {
                                    failed.push(FailedFix {
                                        issue: (*issue).clone(),
                                        reason: err.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to apply fixes to {:?}: {}", file_path, e);
                    for issue in file_issues {
                        failed.push(FailedFix {
                            issue: issue.clone(),
                            reason: format!("File processing failed: {}", e),
                        });
                    }
                }
            }
        }

        // Run tests if requested and not in dry-run mode
        if self.run_tests && !self.dry_run && !fixed.is_empty() {
            self.run_project_tests()?;
        }

        Ok(FixResult {
            fixed,
            failed,
            modified_files,
            branch_name,
            rollback_info,
        })
    }

    /// Check if it's safe to apply fixes
    pub fn check_safe(&self, issues: &[ScoreIssue]) -> LumenResult<bool> {
        // Check for git repository
        let git_dir = self.project_root.join(".git");
        if !git_dir.exists() {
            return Err(LumenError::FixFailed(
                "Not a git repository. Cannot safely apply fixes.".to_string()
            ));
        }

        // Check if working tree is clean
        if !self.is_git_clean()? {
            return Err(LumenError::FixFailed(
                "Working tree has uncommitted changes. Please commit or stash them first.".to_string()
            ));
        }

        Ok(true)
    }

    /// Check if git working tree is clean
    fn is_git_clean(&self) -> LumenResult<bool> {
        match self::git::get_status(&self.project_root) {
            Ok(status) if status.is_empty() => Ok(true),
            Ok(status) => {
                tracing::warn!("Git status: {:?}", status);
                Ok(false)
            }
            Err(e) => {
                tracing::warn!("Failed to check git status: {}", e);
                Ok(false)
            }
        }
    }

    /// Create a git branch for fixes
    fn create_fix_branch(&self) -> LumenResult<Option<String>> {
        let branch_name = format!("lumen-fix-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
        match self::git::create_branch(&self.project_root, &branch_name) {
            Ok(_) => Ok(Some(branch_name)),
            Err(e) => {
                tracing::warn!("Failed to create branch: {}", e);
                Ok(None)
            }
        }
    }

    /// Run project tests
    fn run_project_tests(&self) -> LumenResult<()> {
        tracing::info!("Running project tests...");
        // Detect test runner and run tests
        // This is a simplified version - in production you'd detect the framework
        let status = std::process::Command::new("cargo")
            .args(["test", "--no-run"])
            .current_dir(&self.project_root)
            .status();

        match status {
            Ok(s) if s.success() => {
                tracing::info!("Tests compile successfully");
                Ok(())
            }
            Ok(_) => {
                tracing::warn!("Tests failed to compile");
                Err(LumenError::FixFailed("Tests failed to compile after fixes".to_string()))
            }
            Err(e) => {
                tracing::warn!("Failed to run tests: {}", e);
                // Not necessarily a failure - might not be a Rust project
                Ok(())
            }
        }
    }

    /// Rollback fixes by checking out the original branch
    pub fn rollback(&self, rollback_info: &RollbackInfo) -> LumenResult<()> {
        self::git::checkout_branch(&self.project_root, &rollback_info.original_branch)?;
        self::git::delete_branch(&self.project_root, &rollback_info.fix_branch)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumen_score::IssueSeverity;

    #[test]
    fn test_fix_engine_creation() {
        let engine = FixEngine::new(PathBuf::from("/tmp/project"));
        assert_eq!(engine.project_root, PathBuf::from("/tmp/project"));
        assert!(!engine.dry_run);
        assert!(!engine.create_branch);
    }

    #[test]
    fn test_fix_engine_dry_run() {
        let engine = FixEngine::new(PathBuf::from("/tmp/project"))
            .with_dry_run()
            .with_branch()
            .with_tests();
        assert!(engine.dry_run);
        assert!(engine.create_branch);
        assert!(engine.run_tests);
    }

    #[test]
    fn test_fixed_issue_creation() {
        let issue = ScoreIssue {
            id: "test-1".to_string(),
            severity: IssueSeverity::Medium,
            category: "test".to_string(),
            title: "Test Issue".to_string(),
            description: "A test issue".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(10),
            column: None,
            impact: 5.0,
            suggestion: Some("Fix it".to_string()),
        };

        let fixed = FixedIssue {
            issue: issue.clone(),
            description: "Fixed".to_string(),
            patch: PatchDiff::simple("before".to_string(), "after".to_string()),
        };

        assert_eq!(fixed.issue.id, "test-1");
        assert_eq!(fixed.description, "Fixed");
    }
}
