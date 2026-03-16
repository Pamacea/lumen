//! Git operations for the fix engine

use lumenx_core::{LumenError, LumenResult};
use std::path::Path;
use std::process::Command;

/// Get the current git branch name
pub fn get_current_branch(project_root: &Path) -> LumenResult<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(project_root)
        .output()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(branch)
    } else {
        Err(LumenError::FixFailed(format!(
            "Failed to get current branch: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Get the current HEAD commit SHA
pub fn get_head_sha(project_root: &Path) -> LumenResult<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(project_root)
        .output()?;

    if output.status.success() {
        let sha = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(sha)
    } else {
        Err(LumenError::FixFailed(format!(
            "Failed to get HEAD SHA: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Get git status (returns list of modified files)
pub fn get_status(project_root: &Path) -> LumenResult<Vec<String>> {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(project_root)
        .output()?;

    if output.status.success() {
        let status = String::from_utf8_lossy(&output.stdout);
        let files: Vec<String> = status
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect();
        Ok(files)
    } else {
        Err(LumenError::FixFailed(format!(
            "Failed to get git status: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Create a new git branch and checkout to it
pub fn create_branch(project_root: &Path, branch_name: &str) -> LumenResult<()> {
    let output = Command::new("git")
        .args(["checkout", "-b", branch_name])
        .current_dir(project_root)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(LumenError::FixFailed(format!(
            "Failed to create branch '{}': {}",
            branch_name,
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Checkout to an existing branch
pub fn checkout_branch(project_root: &Path, branch_name: &str) -> LumenResult<()> {
    let output = Command::new("git")
        .args(["checkout", branch_name])
        .current_dir(project_root)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(LumenError::FixFailed(format!(
            "Failed to checkout branch '{}': {}",
            branch_name,
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Delete a git branch
pub fn delete_branch(project_root: &Path, branch_name: &str) -> LumenResult<()> {
    let output = Command::new("git")
        .args(["branch", "-D", branch_name])
        .current_dir(project_root)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(LumenError::FixFailed(format!(
            "Failed to delete branch '{}': {}",
            branch_name,
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Commit changes with a message
pub fn commit(project_root: &Path, message: &str) -> LumenResult<String> {
    // Add all changes
    let add_output = Command::new("git")
        .args(["add", "-A"])
        .current_dir(project_root)
        .output()?;

    if !add_output.status.success() {
        return Err(LumenError::FixFailed(format!(
            "Failed to stage changes: {}",
            String::from_utf8_lossy(&add_output.stderr)
        )));
    }

    // Commit
    let commit_output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(project_root)
        .output()?;

    if commit_output.status.success() {
        get_head_sha(project_root)
    } else {
        Err(LumenError::FixFailed(format!(
            "Failed to commit: {}",
            String::from_utf8_lossy(&commit_output.stderr)
        )))
    }
}

/// Stash current changes
pub fn stash(project_root: &Path) -> LumenResult<()> {
    let output = Command::new("git")
        .args(["stash"])
        .current_dir(project_root)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(LumenError::FixFailed(format!(
            "Failed to stash changes: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

/// Pop stashed changes
pub fn stash_pop(project_root: &Path) -> LumenResult<()> {
    let output = Command::new("git")
        .args(["stash", "pop"])
        .current_dir(project_root)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(LumenError::FixFailed(format!(
            "Failed to pop stash: {}",
            String::from_utf8_lossy(&output.stderr)
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_git_operations() {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .args(["init"])
            .current_dir(project_root)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(project_root)
            .output()
            .unwrap();

        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(project_root)
            .output()
            .unwrap();

        // Test get_current_branch
        let branch = get_current_branch(project_root).unwrap();
        assert_eq!(branch, "main");

        // Test get_status (should be empty)
        let status = get_status(project_root).unwrap();
        assert!(status.is_empty());

        // Test create_branch
        create_branch(project_root, "test-branch").unwrap();
        let branch = get_current_branch(project_root).unwrap();
        assert_eq!(branch, "test-branch");

        // Test checkout_branch
        checkout_branch(project_root, "main").unwrap();
        let branch = get_current_branch(project_root).unwrap();
        assert_eq!(branch, "main");

        // Test delete_branch
        delete_branch(project_root, "test-branch").unwrap();
    }
}
