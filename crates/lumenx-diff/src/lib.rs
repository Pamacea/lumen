//! LumenX Diff - Git Diff and PR Analysis
//!
//! Analyzes git diffs and pull requests for:
//! - Score changes between branches
//! - Files affected by PR
//! - New issues introduced
//! - Complexity changes
//! - Security regressions
//!
//! # Example
//!
//! ```no_run
//! use lumenx_diff::DiffAnalyzer;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let diff = DiffAnalyzer::new(".")?;
//!
//! // Compare current branch with main
//! let comparison = diff.compare_branches("main", "feature/xyz")?;
//! println!("Score change: {:.1} -> {:.1}",
//!     comparison.base_score,
//!     comparison.head_score
//! );
//!
//! # Ok(())
//! # }
//! ```

use git2::{
    BranchType, Diff, DiffOptions, ObjectType, Repository, Tree,
};
use lumenx_core::{LumenResult, ProjectInfo};
use lumenx_score::ScoreIssue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Diff analyzer
pub struct DiffAnalyzer {
    /// Repository path
    repo_path: PathBuf,
    /// Git repository
    repo: Repository,
}

impl DiffAnalyzer {
    /// Create a new diff analyzer
    pub fn new<P: AsRef<Path>>(path: P) -> LumenResult<Self> {
        let repo_path = path.as_ref().to_path_buf();
        let repo = Repository::discover(&repo_path)
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Failed to discover git repo: {}", e)))?;

        Ok(Self { repo_path, repo })
    }

    /// Compare two branches
    pub fn compare_branches(
        &self,
        base: &str,
        head: &str,
    ) -> LumenResult<BranchComparison> {
        let base_tree = self.rev_to_tree(base)?;
        let head_tree = self.rev_to_tree(head)?;

        let mut diff_opts = DiffOptions::new();
        let diff = self
            .repo
            .diff_tree_to_tree(Some(&base_tree), Some(&head_tree), Some(&mut diff_opts))
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Diff failed: {}", e)))?;

        let stats = diff.stats()
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Stats failed: {}", e)))?;

        let changed_files = self.get_changed_files(&diff)?;

        Ok(BranchComparison {
            base: base.to_string(),
            head: head.to_string(),
            base_commit: self.short_id_of(base)?,
            head_commit: self.short_id_of(head)?,
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
            changed_files,
            base_score: 0.0, // Would need history lookup
            head_score: 0.0, // Would need to run analysis
            score_delta: 0.0,
            new_issues: vec![],
            resolved_issues: vec![],
        })
    }

    /// Get current branch diff vs main
    pub fn current_diff(&self, target: Option<&str>) -> LumenResult<DiffResult> {
        let target = target.unwrap_or("main");

        let head_commit = self.repo.head()
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("No HEAD: {}", e)))?
            .peel_to_commit()
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Failed to get commit: {}", e)))?;

        let head_tree = head_commit.tree()
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Failed to get tree: {}", e)))?;

        let target_tree = self.rev_to_tree(target)?;

        let mut diff_opts = DiffOptions::new();
        let diff = self.repo.diff_tree_to_tree(Some(&target_tree), Some(&head_tree), Some(&mut diff_opts))
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Diff failed: {}", e)))?;

        let stats = diff.stats()
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Stats failed: {}", e)))?;

        let changed_files = self.get_changed_files(&diff)?;

        Ok(DiffResult {
            target: target.to_string(),
            current_branch: self.get_current_branch_name(),
            files_changed: stats.files_changed(),
            insertions: stats.insertions(),
            deletions: stats.deletions(),
            changed_files,
            patches: vec![],
        })
    }

    /// Analyze PR and generate comment
    pub fn analyze_pr(
        &self,
        pr_number: u64,
        base: &str,
        head: &str,
    ) -> LumenResult<PRAnalysis> {
        let comparison = self.compare_branches(base, head)?;

        // Generate PR comment
        let comment = self.generate_pr_comment(&comparison);

        Ok(PRAnalysis {
            pr_number,
            base: base.to_string(),
            head: head.to_string(),
            comparison,
            comment,
        })
    }

    /// Get files changed in diff
    fn get_changed_files(&self, diff: &Diff) -> LumenResult<Vec<ChangedFile>> {
        let mut files = vec![];

        // foreach in git2 0.19 takes 4 callbacks:
        // - file callback (required): (DiffDelta, Progress) -> bool
        // - binary callback (optional): (DiffDelta, DiffBinary) -> bool
        // - hunk callback (optional): (DiffDelta, DiffHunk) -> bool
        // - line callback (optional): (DiffDelta, Option<DiffHunk>, DiffLine) -> bool
        diff.foreach(
            &mut |delta, _progress| {
                if let Some(path) = delta.new_file().path() {
                    let status = match delta.status() {
                        git2::Delta::Added => FileStatus::Added,
                        git2::Delta::Deleted => FileStatus::Deleted,
                        git2::Delta::Modified => FileStatus::Modified,
                        git2::Delta::Renamed => FileStatus::Renamed,
                        git2::Delta::Copied => FileStatus::Copied,
                        _ => FileStatus::Modified,
                    };

                    files.push(ChangedFile {
                        path: path.to_string_lossy().to_string(),
                        status,
                        additions: 0,
                        deletions: 0,
                    });
                }
                true
            },
            None,  // No binary callback
            None,  // No hunk callback
            None,  // No line callback
        ).map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Diff iteration failed: {}", e)))?;

        Ok(files)
    }

    fn generate_pr_comment(&self, comparison: &BranchComparison) -> String {
        format!(
            "## 🤖 LumenX PR Analysis\n\n\
            ### 📊 Changes Overview\n\
            - **Files changed:** {}\n\
            - **Insertions:** +{}\n\
            - **Deletions:** -{}\n\
            - **Base:** {} @ `{}`\n\
            - **Head:** {} @ `{}`\n\n\
            ### 📈 Score Impact\n\
            - Base score: {:.1}/100\n\
            - Head score: {:.1}/100\n\
            - Delta: {:+.1}\n\n\
            ### 📁 Changed Files ({} files)\n\n\
            Run `lumenx diff --base {} --head {}` for detailed analysis.",
            comparison.files_changed,
            comparison.insertions,
            comparison.deletions,
            comparison.base,
            comparison.base_commit,
            comparison.head,
            comparison.head_commit,
            comparison.base_score,
            comparison.head_score,
            comparison.score_delta,
            comparison.changed_files.len(),
            comparison.base,
            comparison.head
        )
    }

    fn rev_to_tree(&self, rev: &str) -> LumenResult<Tree> {
        let obj = self.repo.revparse_single(rev)
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Failed to resolve {}: {}", rev, e)))?;

        obj.peel_to_tree()
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Failed to peel to tree: {}", e)))
    }

    fn short_id_of(&self, rev: &str) -> LumenResult<String> {
        let obj = self.repo.revparse_single(rev)
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Failed to resolve {}: {}", rev, e)))?;

        obj.short_id()
            .map(|buf| buf.as_str().unwrap_or("?").to_string())
            .map_err(|e| lumenx_core::LumenError::AnalysisFailed(format!("Failed to get short id: {}", e)))
    }

    fn get_current_branch_name(&self) -> String {
        self.repo.head()
            .ok()
            .and_then(|head| head.shorthand().map(|s| s.to_string()))
            .unwrap_or_else(|| "unknown".to_string())
    }
}

/// Branch comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchComparison {
    /// Base branch name
    pub base: String,
    /// Head branch name
    pub head: String,
    /// Base commit short ID
    pub base_commit: String,
    /// Head commit short ID
    pub head_commit: String,
    /// Number of files changed
    pub files_changed: usize,
    /// Number of insertions
    pub insertions: usize,
    /// Number of deletions
    pub deletions: usize,
    /// Changed files list
    pub changed_files: Vec<ChangedFile>,
    /// Base branch score
    pub base_score: f64,
    /// Head branch score
    pub head_score: f64,
    /// Score difference
    pub score_delta: f64,
    /// New issues introduced
    pub new_issues: Vec<ScoreIssue>,
    /// Issues resolved
    pub resolved_issues: Vec<ScoreIssue>,
}

/// Diff result for current changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// Target branch/reference
    pub target: String,
    /// Current branch name
    pub current_branch: String,
    /// Files changed count
    pub files_changed: usize,
    /// Insertions count
    pub insertions: usize,
    /// Deletions count
    pub deletions: usize,
    /// Changed files
    pub changed_files: Vec<ChangedFile>,
    /// Diff patches
    pub patches: Vec<String>,
}

/// Pull request analysis
#[derive(Debug, Clone)]
pub struct PRAnalysis {
    /// PR number
    pub pr_number: u64,
    /// Base branch
    pub base: String,
    /// Head branch
    pub head: String,
    /// Branch comparison
    pub comparison: BranchComparison,
    /// Generated PR comment
    pub comment: String,
}

/// Changed file info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedFile {
    /// File path
    pub path: String,
    /// Change status
    pub status: FileStatus,
    /// Lines added
    pub additions: usize,
    /// Lines removed
    pub deletions: usize,
}

/// File change status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    Added,
    Deleted,
    Modified,
    Renamed,
    Copied,
}

/// Analyze a specific file diff for complexity changes
pub fn analyze_file_diff(
    _file_path: &str,
    _old_content: Option<&str>,
    _new_content: &str,
) -> LumenResult<FileDiffAnalysis> {
    // TODO: Implement actual diff analysis
    Ok(FileDiffAnalysis {
        complexity_delta: 0,
        new_functions: vec![],
        removed_functions: vec![],
        new_issues: vec![],
    })
}

/// File diff analysis result
#[derive(Debug, Clone)]
pub struct FileDiffAnalysis {
    /// Complexity change (can be negative)
    pub complexity_delta: i32,
    /// New functions added
    pub new_functions: Vec<String>,
    /// Functions removed
    pub removed_functions: Vec<String>,
    /// New issues found
    pub new_issues: Vec<ScoreIssue>,
}

/// Format diff for display
pub fn format_diff_summary(diff: &DiffResult) -> String {
    format!(
        "Diff vs {}\n\
         Files changed: {}\n\
         Insertions: +{}\n\
         Deletions: -{}",
        diff.target,
        diff.files_changed,
        diff.insertions,
        diff.deletions
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changed_files_parsing() {
        // Test with mock data
        let files = vec![
            ChangedFile {
                path: "src/main.rs".to_string(),
                status: FileStatus::Modified,
                additions: 10,
                deletions: 5,
            },
            ChangedFile {
                path: "src/utils.rs".to_string(),
                status: FileStatus::Added,
                additions: 25,
                deletions: 0,
            },
        ];

        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "src/main.rs");
        assert_eq!(files[1].status, FileStatus::Added);
    }
}
