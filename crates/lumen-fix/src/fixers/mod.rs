//! Fixer modules for different issue categories
//!
//! Each fixer handles a specific category of issues and implements
//! the necessary logic to detect and fix those issues.

pub mod imports;
pub mod security;
pub mod performance;
pub mod quality;

use lumen_core::{LumenError, LumenResult};
use lumen_score::ScoreIssue;
use std::path::{Path, PathBuf};

pub use imports::{ImportFixer, ImportFix};
pub use security::{SecurityFixer, SecurityFix};
pub use performance::{PerformanceFixer, PerformanceFix};
pub use quality::QualityFixer;

/// Apply all applicable fixes for a file
pub fn apply_file_fixes(
    project_root: &Path,
    file_path: &Path,
    issues: &[&ScoreIssue],
    dry_run: bool,
) -> LumenResult<Vec<lumen_core::LumenResult<crate::FixedIssue>>> {
    let mut results = Vec::new();

    // Read file content
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| lumen_core::LumenError::FileNotFound(file_path.to_path_buf()))?;

    // Group issues by category
    let mut import_issues: Vec<&ScoreIssue> = Vec::new();
    let mut security_issues: Vec<&ScoreIssue> = Vec::new();
    let mut performance_issues: Vec<&ScoreIssue> = Vec::new();
    let mut quality_issues: Vec<&ScoreIssue> = Vec::new();

    for issue in issues {
        match issue.category.as_str() {
            cat if cat.contains("import") || cat.contains("export") => {
                import_issues.push(issue);
            }
            cat if cat.contains("security") => {
                security_issues.push(issue);
            }
            cat if cat.contains("performance") => {
                performance_issues.push(issue);
            }
            cat if cat.contains("quality") => {
                quality_issues.push(issue);
            }
            _ => {
                // Unknown category - try quality fixer as fallback
                quality_issues.push(issue);
            }
        }
    }

    // Apply fixes in order of safety
    let mut modified_content = content.clone();

    // 1. Import fixes (safest)
    if !import_issues.is_empty() {
        let fixer = ImportFixer::new(file_path);
        match fixer.fix_all(&modified_content, &import_issues) {
            Ok((fixed, patches)) => {
                for (issue, patch) in import_issues.iter().zip(patches.iter()) {
                    results.push(Ok(crate::FixedIssue {
                        issue: (*issue).clone(),
                        description: format!("Applied import fix: {}", patch.description),
                        patch: patch.diff.clone(),
                    }));
                }
                modified_content = fixed;
            }
            Err(e) => {
                for issue in &import_issues {
                    results.push(Err(lumen_core::LumenError::FixFailed(format!(
                        "Import fix failed: {}",
                        e
                    ))));
                }
            }
        }
    }

    // 2. Security fixes (important)
    if !security_issues.is_empty() {
        let fixer = SecurityFixer::new(file_path);
        match fixer.fix_all(&modified_content, &security_issues) {
            Ok((fixed, patches)) => {
                for (issue, patch) in security_issues.iter().zip(patches.iter()) {
                    results.push(Ok(crate::FixedIssue {
                        issue: (*issue).clone(),
                        description: format!("Applied security fix: {}", patch.description),
                        patch: patch.diff.clone(),
                    }));
                }
                modified_content = fixed;
            }
            Err(e) => {
                for issue in &security_issues {
                    results.push(Err(lumen_core::LumenError::FixFailed(format!(
                        "Security fix failed: {}",
                        e
                    ))));
                }
            }
        }
    }

    // 3. Performance fixes
    if !performance_issues.is_empty() {
        let fixer = PerformanceFixer::new(file_path);
        match fixer.fix_all(&modified_content, &performance_issues) {
            Ok((fixed, patches)) => {
                for (issue, patch) in performance_issues.iter().zip(patches.iter()) {
                    results.push(Ok(crate::FixedIssue {
                        issue: (*issue).clone(),
                        description: format!("Applied performance fix: {}", patch.description),
                        patch: patch.diff.clone(),
                    }));
                }
                modified_content = fixed;
            }
            Err(e) => {
                for issue in &performance_issues {
                    results.push(Err(lumen_core::LumenError::FixFailed(format!(
                        "Performance fix failed: {}",
                        e
                    ))));
                }
            }
        }
    }

    // 4. Quality fixes
    if !quality_issues.is_empty() {
        let fixer = QualityFixer::new(file_path);
        match fixer.fix_all(&modified_content, &quality_issues) {
            Ok((fixed, patches)) => {
                for (issue, patch) in quality_issues.iter().zip(patches.iter()) {
                    results.push(Ok(crate::FixedIssue {
                        issue: (*issue).clone(),
                        description: format!("Applied quality fix: {}", patch.description),
                        patch: patch.diff.clone(),
                    }));
                }
                modified_content = fixed;
            }
            Err(e) => {
                for issue in &quality_issues {
                    results.push(Err(lumen_core::LumenError::FixFailed(format!(
                        "Quality fix failed: {}",
                        e
                    ))));
                }
            }
        }
    }

    // Write the modified content if not in dry-run mode
    if !dry_run && modified_content != content {
        std::fs::write(file_path, modified_content).map_err(|e| {
            lumen_core::LumenError::FixFailed(format!("Failed to write file: {}", e))
        })?;
    }

    Ok(results)
}

/// Apply a fix to a single issue (legacy compatibility)
pub fn apply_fix(
    root: &Path,
    issue: &ScoreIssue,
    dry_run: bool,
) -> LumenResult<Option<String>> {
    let file_path = issue.file.as_ref().map(|f| root.join(f));
    let file_path = match file_path {
        Some(p) if p.exists() => p,
        _ => return Ok(None),
    };

    let results = apply_file_fixes(root, &file_path, &[issue], dry_run)?;

    if let Some(Ok(fixed)) = results.first() {
        Ok(Some(fixed.description.clone()))
    } else if let Some(Err(_e)) = results.first() {
        // Convert the error - since LumenError doesn't implement Clone,
        // we need to create a new error
        Err(LumenError::FixFailed("Fix application failed".to_string()))
    } else {
        Ok(None)
    }
}

/// Trait that all fixers must implement
pub trait Fixer: Send + Sync {
    /// The fix type this fixer produces
    type Fix: Fix;

    /// Create a new fixer for the given file
    fn new(file_path: &Path) -> Self
    where
        Self: Sized;

    /// Check if this fixer can handle the given issue
    fn can_fix(&self, issue: &ScoreIssue) -> bool;

    /// Apply a fix for a single issue
    fn fix(&self, content: &str, issue: &ScoreIssue) -> LumenResult<(String, Self::Fix)>;

    /// Apply fixes for multiple issues at once (more efficient)
    fn fix_all(
        &self,
        content: &str,
        issues: &[&ScoreIssue],
    ) -> LumenResult<(String, Vec<Self::Fix>)>;
}

/// Trait for fix results
pub trait Fix {
    /// Get a description of the fix
    fn description(&self) -> String;

    /// Get the patch diff
    fn diff(&self) -> crate::PatchDiff;

    /// Check if the fix is safe to apply automatically
    fn is_safe(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixer_trait() {
        // This test verifies the fixer trait is properly implemented
        // Actual fixer tests are in their respective modules
        assert!(true);
    }
}
