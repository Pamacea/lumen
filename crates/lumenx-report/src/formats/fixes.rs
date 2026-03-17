//! AI-Ready fixes.json generation
//!
//! This format is designed for AI agents (Claude Code, etc.) to automatically
//! apply fixes to a codebase. Each fix includes exact file locations, code snippets,
//! and verification commands.

use lumenx_core::ProjectInfo;
use lumenx_score::{DimensionScores, ScoreIssue};
use serde_json::json;
use std::collections::HashMap;

/// Generate AI-ready fixes JSON
pub fn generate_fixes_json(info: &ProjectInfo, dimensions: &DimensionScores) -> String {
    let fixes = collect_fixes(dimensions, info);

    let output = json!({
        "version": "1.0",
        "format": "lumen-fixes",
        "generated_at": chrono::Utc::now().to_rfc3339(),
        "project": {
            "name": info.name,
            "root": info.root.to_string_lossy().to_string(),
            "framework": info.framework.display_name(),
            "language": info.language.display_name(),
        },
        "summary": {
            "total_fixes": fixes.len(),
            "critical": fixes.iter().filter(|f| f["severity"]["level"] == 4).count(),
            "high": fixes.iter().filter(|f| f["severity"]["level"] == 3).count(),
            "medium": fixes.iter().filter(|f| f["severity"]["level"] == 2).count(),
            "low": fixes.iter().filter(|f| f["severity"]["level"] == 1).count(),
        },
        "fixes": fixes,
    });

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

/// Collect all fixes with full context
fn collect_fixes(dimensions: &DimensionScores, info: &ProjectInfo) -> Vec<serde_json::Value> {
    let mut all_fixes = Vec::new();

    for (dim_name, dim) in dimensions.all() {
        for issue in &dim.issues {
            if let Some(fix) = create_fix_entry(issue, dim_name, info) {
                all_fixes.push(fix);
            }
        }
    }

    // Sort by severity (critical first)
    all_fixes.sort_by(|a, b| {
        let sev_a = a["severity"]["level"].as_i64().unwrap_or(0);
        let sev_b = b["severity"]["level"].as_i64().unwrap_or(0);
        sev_b.cmp(&sev_a)
    });

    all_fixes
}

/// Create a detailed fix entry for an issue
fn create_fix_entry(issue: &ScoreIssue, dimension: &str, info: &ProjectInfo) -> Option<serde_json::Value> {
    let severity_level = match issue.severity {
        lumenx_score::IssueSeverity::Critical => 4,
        lumenx_score::IssueSeverity::High => 3,
        lumenx_score::IssueSeverity::Medium => 2,
        lumenx_score::IssueSeverity::Low => 1,
        lumenx_score::IssueSeverity::Info => 0,
    };

    let (fix_command, verify_command, code_snippet) = generate_fix_commands(issue, info);

    Some(json!({
        "id": issue.id,
        "severity": {
            "level": severity_level,
            "name": issue.severity.as_str(),
            "color": issue.severity.color(),
        },
        "category": issue.category,
        "dimension": dimension,
        "title": issue.title,
        "description": issue.description,
        "location": {
            "file": issue.file,
            "line": issue.line,
            "column": issue.column,
            "relative_path": issue.file.as_ref().and_then(|f| {
                f.strip_prefix(&info.root.to_string_lossy().to_string())
                    .map(|s| s.trim_start_matches('\\').trim_start_matches('/').to_string())
                    .or_else(|| f.strip_prefix("/").map(|s| s.to_string()))
            }),
        },
        "code": {
            "snippet": code_snippet,
            "language": info.language.display_name(),
        },
        "fix": {
            "command": fix_command,
            "description": issue.suggestion.clone().unwrap_or_else(|| generate_default_suggestion(issue)),
            "automated": can_be_automated(issue),
            "risk": estimate_fix_risk(issue),
        },
        "verification": {
            "command": verify_command,
            "expected_outcome": verify_outcome(issue),
        },
        "impact": {
            "score_delta": estimate_score_impact(issue),
            "dimension": dimension,
        },
        "metadata": {
            "effort": estimate_effort(issue),
            "priority": calculate_priority(issue, severity_level),
        },
    }))
}

/// Generate fix and verification commands for an issue
fn generate_fix_commands(issue: &ScoreIssue, _info: &ProjectInfo) -> (Option<String>, Option<String>, Option<String>) {
    match issue.category.as_str() {
        "security" => {
            if issue.title.contains(".env") || issue.title.contains("Environment file") {
                let verify_cmd = Some("!git ls-files | grep -E '^\\.env$' || echo 'PASS: No .env in git'".to_string());
                let fix_cmd = if let Some(ref file) = issue.file {
                    Some(format!("git rm --cached {} && echo '{}' >> .gitignore",
                        file,
                        "# Environment files\n.env\n.env.local\n.env.*.local"
                    ))
                } else {
                    Some("echo '*.env' >> .gitignore && git rm --cached .env 2>/dev/null || true".to_string())
                };
                (fix_cmd, verify_cmd, None)
            } else {
                (None, None, None)
            }
        }
        "seo" => {
            if issue.title.contains("title") {
                let snippet = Some("<head>\n  <meta charset=\"utf-8\" />\n  <!-- ADD TITLE BELOW -->\n  <title>Your Page Title</title>\n</head>".to_string());
                (Some("Add <title> tag to <head> section".to_string()), None, snippet)
            } else if issue.title.contains("meta description") {
                let snippet = Some("<head>\n  <meta name=\"description\" content=\"Your page description here\" />\n</head>".to_string());
                (Some("Add <meta name=\"description\"> tag".to_string()), None, snippet)
            } else if issue.title.contains("Open Graph") {
                let snippet = Some("<head>\n  <meta property=\"og:title\" content=\"Title\" />\n  <meta property=\"og:description\" content=\"Description\" />\n  <meta property=\"og:image\" content=\"/og-image.jpg\" />\n  <meta property=\"og:url\" content=\"https://example.com/page\" />\n  <meta property=\"og:type\" content=\"website\" />\n</head>".to_string());
                (Some("Add Open Graph meta tags".to_string()), None, snippet)
            } else if issue.title.contains("viewport") {
                let snippet = Some("<head>\n  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n</head>".to_string());
                (Some("Add viewport meta tag".to_string()), None, snippet)
            } else {
                (None, None, None)
            }
        }
        "performance" => {
            if issue.title.contains("Nested loops") {
                if let Some(ref file) = issue.file {
                    let cmd = format!("echo 'Review nested loops in {} and consider refactoring'", file);
                    (Some(cmd), None, None)
                } else {
                    (None, None, None)
                }
            } else {
                (None, None, None)
            }
        }
        "quality" => {
            (None, None, None)
        }
        "docs" => {
            if issue.title.contains("CONTRIBUTING") {
                (Some("Create CONTRIBUTING.md with guidelines".to_string()), Some("test -f CONTRIBUTING.md && echo 'EXISTS' || echo 'MISSING'".to_string()), None)
            } else if issue.title.contains("documentation") {
                (Some("Add function documentation comments".to_string()), None, None)
            } else {
                (None, None, None)
            }
        }
        "uiux" => {
            (None, None, None)
        }
        _ => (None, None, None)
    }
}

/// Generate default suggestion when none exists
fn generate_default_suggestion(issue: &ScoreIssue) -> String {
    format!("Fix the {}: {}", issue.category, issue.title)
}

/// Check if a fix can be fully automated
fn can_be_automated(issue: &ScoreIssue) -> bool {
    matches!(
        issue.category.as_str(),
        "security" | "seo"
    )
}

/// Estimate fix risk level
fn estimate_fix_risk(issue: &ScoreIssue) -> String {
    match issue.severity {
        lumenx_score::IssueSeverity::Critical => "low".to_string(), // Critical security issues should be fixed
        lumenx_score::IssueSeverity::High => "low".to_string(),
        lumenx_score::IssueSeverity::Medium => "low".to_string(),
        lumenx_score::IssueSeverity::Low => "very-low".to_string(),
        lumenx_score::IssueSeverity::Info => "none".to_string(),
    }
}

/// Verify expected outcome after fix
fn verify_outcome(issue: &ScoreIssue) -> String {
    match issue.category.as_str() {
        "security" => "Issue should be resolved after applying the fix".to_string(),
        "seo" => "Meta tags should be present in <head> section".to_string(),
        "performance" => "Performance metrics should improve".to_string(),
        "quality" => "Code quality score should increase".to_string(),
        "docs" => "Documentation should be present and accessible".to_string(),
        "uiux" => "UX issue should be resolved".to_string(),
        _ => "Issue should be resolved".to_string(),
    }
}

/// Estimate score impact of fixing this issue
fn estimate_score_impact(issue: &ScoreIssue) -> f64 {
    match issue.severity {
        lumenx_score::IssueSeverity::Critical => 5.0,
        lumenx_score::IssueSeverity::High => 3.0,
        lumenx_score::IssueSeverity::Medium => 1.5,
        lumenx_score::IssueSeverity::Low => 0.5,
        lumenx_score::IssueSeverity::Info => 0.0,
    }
}

/// Estimate effort required to fix
fn estimate_effort(issue: &ScoreIssue) -> String {
    match issue.category.as_str() {
        "security" => match issue.severity {
            lumenx_score::IssueSeverity::Critical => "5 minutes",
            _ => "10 minutes",
        },
        "seo" => "5 minutes",
        "docs" => "30 minutes",
        "performance" => "1 hour",
        "quality" => "2 hours",
        "uiux" => "2 hours",
        _ => "15 minutes",
    }.to_string()
}

/// Calculate priority based on severity and effort
fn calculate_priority(issue: &ScoreIssue, severity_level: i64) -> i64 {
    let base_priority = severity_level * 10;
    let effort_penalty = match estimate_effort(issue).as_str() {
        e if e.contains("5 minutes") => 0,
        e if e.contains("10 minutes") => 1,
        e if e.contains("30 minutes") => 2,
        e if e.contains("1 hour") => 3,
        _ => 4,
    };
    base_priority - effort_penalty
}

/// Generate a summary of fixes
pub fn generate_fixes_summary(info: &ProjectInfo, dimensions: &DimensionScores) -> String {
    let fixes = collect_fixes(dimensions, info);

    let total = fixes.len();
    let critical = fixes.iter().filter(|f| f["severity"]["level"] == 4).count();
    let high = fixes.iter().filter(|f| f["severity"]["level"] == 3).count();

    format!(
        "## 📋 Fixes Available\n\n\
         Total fixes: **{}**\n\
         - 🚨 Critical: {}\n\
         - ⚠️ High: {}\n\
         - ⚡ Medium+Low: {}\n\n\
         Run `lumenx fix` to apply automatically.",
        total,
        critical,
        high,
        total - critical - high
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumenx_score::{DimensionScore, DimensionScores, IssueSeverity};
    use lumenx_core::{Framework, Language, TestRunner};
    use std::path::PathBuf;

    fn mock_info() -> ProjectInfo {
        ProjectInfo {
            name: "test".to_string(),
            root: PathBuf::from("/test"),
            framework: Framework::NextJs,
            language: Language::TypeScript,
            test_runner: TestRunner::Vitest,
            package_manager: Some("npm".to_string()),
            dependencies: vec![],
            dev_dependencies: vec![],
            database: None,
            package_json: None,
            cargo_dependencies: None,
        }
    }

    fn mock_dimensions() -> DimensionScores {
        DimensionScores {
            coverage: DimensionScore::new("coverage".to_string(), 50.0, 0.25),
            quality: DimensionScore::new("quality".to_string(), 60.0, 0.20),
            performance: DimensionScore::new("performance".to_string(), 70.0, 0.15),
            security: DimensionScore::new("security".to_string(), 80.0, 0.15),
            seo: DimensionScore::new("seo".to_string(), 40.0, 0.10),
            docs: DimensionScore::new("docs".to_string(), 30.0, 0.05),
            uiux: DimensionScore::new("uiux".to_string(), 60.0, 0.10),
        }
    }

    #[test]
    fn test_fixes_json_valid() {
        let info = mock_info();
        let dimensions = mock_dimensions();
        let json = generate_fixes_json(&info, &dimensions);

        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
    }

    #[test]
    fn test_fixes_summary() {
        let info = mock_info();
        let dimensions = mock_dimensions();
        let summary = generate_fixes_summary(&info, &dimensions);

        assert!(summary.contains("Fixes Available"));
    }
}
