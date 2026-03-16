//! Patch generation and application

use serde::{Deserialize, Serialize};
use lumen_core::{LumenError, LumenResult, Project};
use lumen_analyze::ScoreIssue;

/// A patch that can be applied to a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Patch {
    /// File to patch
    pub file: String,
    /// Issue ID this patch addresses
    pub issue_id: String,
    /// Description of the patch
    pub description: String,
    /// hunks that make up the patch
    pub hunks: Vec<PatchHunk>,
}

impl Patch {
    /// Create a patch from an issue
    pub fn from_issue(issue: &ScoreIssue, project: &Project) -> LumenResult<Self> {
        let file = issue.file.clone().ok_or_else(|| {
            LumenError::InvalidPath("No file specified for patch".to_string())
        })?;

        Ok(Self {
            file: file.clone(),
            issue_id: issue.id.clone(),
            description: issue.title.clone(),
            hunks: vec![],
        })
    }

    /// Apply the patch to a file
    pub fn apply(&self, project: &Project) -> LumenResult<()> {
        let content = std::fs::read_to_string(&self.file)?;
        let patched = self.apply_to_content(&content)?;
        std::fs::write(&self.file, &patched)?;
        Ok(())
    }

    /// Apply patch to content
    fn apply_to_content(&self, content: &str) -> LumenResult<String> {
        let mut result = content.to_string();

        for hunk in &self.hunks {
            result = hunk.apply(&result)?;
        }

        Ok(result)
    }

    /// Generate a unified diff
    pub fn unified_diff(&self) -> String {
        let mut diff = String::new();
        diff.push_str(&format!("--- a/{}\n", self.file));
        diff.push_str(&format!("+++ b/{}\n", self.file));

        for hunk in &self.hunks {
            diff.push_str(&hunk.to_string());
        }

        diff
    }
}

/// A single hunk in a patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchHunk {
    /// Old line range (start, end)
    pub old_range: (usize, usize),
    /// New line range (start, end)
    pub new_range: (usize, usize),
    /// Lines to remove (prefixed with "-")
    pub remove: Vec<String>,
    /// Lines to add (prefixed with "+")
    pub add: Vec<String>,
}

impl PatchHunk {
    /// Create a new hunk
    pub fn new(old_range: (usize, usize), new_range: (usize, usize)) -> Self {
        Self {
            old_range,
            new_range,
            remove: Vec::new(),
            add: Vec::new(),
        }
    }

    /// Add lines to remove
    pub fn with_remove(mut self, lines: Vec<String>) -> Self {
        self.remove = lines;
        self
    }

    /// Add lines to add
    pub fn with_add(mut self, lines: Vec<String>) -> Self {
        self.add = lines;
        self
    }

    /// Apply this hunk to content
    fn apply(&self, content: &str) -> LumenResult<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result = Vec::new();

        // Copy lines before the hunk
        for (i, line) in lines.iter().enumerate() {
            if i == self.old_range.0 {
                break;
            }
            result.push(line.to_string());
        }

        // Add new lines
        result.extend(self.add.clone());

        // Skip removed lines and continue
        for (i, line) in lines.iter().enumerate() {
            if i >= self.old_range.1 {
                result.push(line.to_string());
            }
        }

        Ok(result.join("\n"))
    }
}

impl std::fmt::Display for PatchHunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "@@ -{},{} +{},{} @@",
            self.old_range.0 + 1,
            self.old_range.1 - self.old_range.0,
            self.new_range.0 + 1,
            self.new_range.1 - self.new_range.0
        )?;

        for line in &self.remove {
            writeln!(f, "-{}", line)?;
        }

        for line in &self.add {
            writeln!(f, "+{}", line)?;
        }

        Ok(())
    }
}

/// A diff between two strings
#[derive(Debug, Clone)]
pub struct PatchDiff {
    /// Original content
    pub original: String,
    /// Modified content
    pub modified: String,
    /// Unified diff string
    pub diff: String,
}

impl Default for PatchDiff {
    fn default() -> Self {
        Self {
            original: String::new(),
            modified: String::new(),
            diff: String::new(),
        }
    }
}

impl PatchDiff {
    /// Create a simple diff between two strings
    pub fn simple(original: String, modified: String) -> Self {
        let diff = Self::compute_unified_diff(&original, &modified);

        Self {
            original,
            modified,
            diff,
        }
    }

    /// Compute a unified diff
    fn compute_unified_diff(original: &str, modified: &str) -> String {
        // Simple implementation - in real code use diffy library
        let mut diff = String::new();
        diff.push_str("--- original\n");
        diff.push_str("+++ modified\n");

        let orig_lines: Vec<_> = original.lines().collect();
        let mod_lines: Vec<_> = modified.lines().collect();

        // Simple line-by-line comparison
        let max_lines = orig_lines.len().max(mod_lines.len());

        for i in 0..max_lines {
            let orig = orig_lines.get(i);
            let mod_ = mod_lines.get(i);

            match (orig, mod_) {
                (Some(o), Some(m)) if o == m => {
                    diff.push_str(&format!(" {}\n", o));
                }
                (Some(o), Some(m)) => {
                    diff.push_str(&format!("-{}\n", o));
                    diff.push_str(&format!("+{}\n", m));
                }
                (Some(o), None) => {
                    diff.push_str(&format!("-{}\n", o));
                }
                (None, Some(m)) => {
                    diff.push_str(&format!("+{}\n", m));
                }
                _ => {}
            }
        }

        diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_creation() {
        let hunk = PatchHunk::new((10, 15), (10, 12));
        assert_eq!(hunk.old_range, (10, 15));
        assert_eq!(hunk.new_range, (10, 12));
    }

    #[test]
    fn test_patch_diff_simple() {
        let diff = PatchDiff::simple(
            "line 1\nline 2\nline 3".to_string(),
            "line 1\nmodified\nline 3".to_string(),
        );

        assert!(diff.diff.contains("-line 2"));
        assert!(diff.diff.contains("+modified"));
    }
}
