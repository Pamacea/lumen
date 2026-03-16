//! Auto-fixer for common code issues

use lumenx_score::ScoreIssue;
use lumenx_core::LumenResult;
use std::path::PathBuf;

/// Auto-fixer context
#[derive(Debug, Clone)]
pub struct FixContext {
    /// File to fix
    pub file: PathBuf,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Original content
    pub original: String,
    /// Fixed content
    pub fixed: String,
    /// Whether to apply the fix
    pub apply: bool,
}

/// Auto-fixer for common issues
pub struct AutoFixer {
    project_root: PathBuf,
}

impl AutoFixer {
    /// Create a new auto-fixer
    pub fn new(project_root: PathBuf) -> LumenResult<Self> {
        Ok(Self { project_root })
    }

    /// Fix a single issue
    pub fn fix_issue(&self, issue: &ScoreIssue, dry_run: bool) -> LumenResult<FixResult> {
        let file_path = issue.file.as_ref().map(PathBuf::from);
        let line = issue.line;

        match issue.category.as_str() {
            "unused-import" => self.fix_unused_import(issue, dry_run),
            "missing-export" => self.fix_missing_export(issue, dry_run),
            "formatting" => self.fix_formatting(issue, dry_run),
            _ => Ok(FixResult {
                issue_id: issue.id.clone(),
                severity: FixSeverity::from(issue.severity),
                success: false,
                message: "Issue type not auto-fixable".to_string(),
                file: file_path,
                line,
                patch: None,
            }),
        }
    }

    fn fix_unused_import(&self, issue: &ScoreIssue, dry_run: bool) -> LumenResult<FixResult> {
        let file = issue.file.as_ref().ok_or_else(|| {
            lumenx_core::LumenError::InvalidPath("No file specified".to_string())
        })?;

        let file_path = self.project_root.join(file);
        let content = std::fs::read_to_string(&file_path).map_err(|e| {
            lumenx_core::LumenError::FileNotFound(file_path.clone())
        })?;
        let line = issue.line.unwrap_or(0);

        // Remove the import line
        let new_content: String = content
            .lines()
            .enumerate()
            .filter(|(i, _)| *i != line)
            .map(|(_, l)| l)
            .collect::<Vec<_>>()
            .join("\n");

        if !dry_run {
            std::fs::write(&file_path, new_content.as_bytes()).map_err(|e| {
                lumenx_core::LumenError::FixFailed(format!("Failed to write file: {}", e))
            })?;
        }

        Ok(FixResult {
            issue_id: issue.id.clone(),
            severity: FixSeverity::from(issue.severity),
            success: true,
            message: format!("Removed unused import at line {}", line + 1),
            file: Some(file_path),
            line: issue.line,
            patch: Some(crate::patch::PatchDiff::simple(content, new_content)),
        })
    }

    fn fix_missing_export(&self, issue: &ScoreIssue, dry_run: bool) -> LumenResult<FixResult> {
        let file = issue.file.as_ref().ok_or_else(|| {
            lumenx_core::LumenError::InvalidPath("No file specified".to_string())
        })?;

        let file_path = self.project_root.join(file);
        let content = std::fs::read_to_string(&file_path).map_err(|e| {
            lumenx_core::LumenError::FileNotFound(file_path.clone())
        })?;

        // Add pub keyword to the item
        let new_content = content.replace(
            &format!("fn {}", issue.description),
            &format!("pub fn {}", issue.description),
        );

        if !dry_run {
            std::fs::write(&file_path, new_content.as_bytes()).map_err(|e| {
                lumenx_core::LumenError::FixFailed(format!("Failed to write file: {}", e))
            })?;
        }

        Ok(FixResult {
            issue_id: issue.id.clone(),
            severity: FixSeverity::from(issue.severity),
            success: true,
            message: format!("Added pub export to {}", issue.description),
            file: Some(file_path),
            line: issue.line,
            patch: Some(crate::patch::PatchDiff::simple(content, new_content)),
        })
    }

    fn fix_formatting(&self, issue: &ScoreIssue, dry_run: bool) -> LumenResult<FixResult> {
        Ok(FixResult {
            issue_id: issue.id.clone(),
            severity: FixSeverity::Low,
            success: false,
            message: "Formatting fixes require external tool (rustfmt/prettier)".to_string(),
            file: issue.file.as_ref().map(PathBuf::from),
            line: issue.line,
            patch: None,
        })
    }
}

/// Result of a fix operation
#[derive(Debug, Clone)]
pub struct FixResult {
    /// Issue ID that was fixed
    pub issue_id: String,
    /// Severity of the fix
    pub severity: FixSeverity,
    /// Whether the fix was successful
    pub success: bool,
    /// Human-readable message
    pub message: String,
    /// File that was modified
    pub file: Option<PathBuf>,
    /// Line number
    pub line: Option<usize>,
    /// Patch diff
    pub patch: Option<crate::patch::PatchDiff>,
}

/// Severity of a fix operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl From<lumenx_score::IssueSeverity> for FixSeverity {
    fn from(severity: lumenx_score::IssueSeverity) -> Self {
        match severity {
            lumenx_score::IssueSeverity::Critical => FixSeverity::Critical,
            lumenx_score::IssueSeverity::High => FixSeverity::High,
            lumenx_score::IssueSeverity::Medium => FixSeverity::Medium,
            lumenx_score::IssueSeverity::Low => FixSeverity::Low,
            lumenx_score::IssueSeverity::Info => FixSeverity::Info,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_conversion() {
        let severity = lumenx_score::IssueSeverity::High;
        let fix_severity = FixSeverity::from(severity);
        assert_eq!(fix_severity, FixSeverity::High);
    }
}
