//! JUnit XML report generation for CI/CD integration

use lumenx_core::ProjectInfo;
use lumenx_score::{DimensionScores, IssueSeverity, ProjectScore};

/// Generate JUnit XML report
///
/// This format is widely used by CI/CD systems (Jenkins, GitLab, GitHub Actions, etc.)
/// to display test results and quality metrics.
pub fn generate(info: &ProjectInfo, score: &ProjectScore) -> String {
    let timestamp = format_timestamp_junit(score.timestamp);
    let duration = score.metadata.scan_duration_ms as f64 / 1000.0;

    let mut xml = String::new();

    // XML declaration
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

    // Main testsuite - one for overall project score
    xml.push_str(&format!(
        "<testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" skipped=\"0\" errors=\"{}\" time=\"{:.3}\" timestamp=\"{}\">\n",
        html_escape(&info.name),
        count_total_tests(&score.dimensions, score.overall),
        count_failures(&score.dimensions, score.overall),
        count_critical_failures(&score.dimensions, score.overall),
        duration,
        timestamp
    ));

    // Properties
    xml.push_str("  <properties>\n");
    xml.push_str(&format!("    <property name=\"lumen.version\" value=\"{}\"/>\n", score.metadata.scorer_version));
    xml.push_str(&format!("    <property name=\"lumen.score\" value=\"{:.1}\"/>\n", score.overall));
    xml.push_str(&format!("    <property name=\"lumen.grade\" value=\"{}\"/>\n", score.grade.as_letter()));
    xml.push_str(&format!("    <property name=\"lumen.gpa\" value=\"{:.2}\"/>\n", score.grade.gpa_value()));
    xml.push_str(&format!("    <property name=\"project.framework\" value=\"{}\"/>\n", info.framework.display_name()));
    xml.push_str(&format!("    <property name=\"project.language\" value=\"{}\"/>\n", info.language.display_name()));
    xml.push_str(&format!("    <property name=\"project.files_scanned\" value=\"{}\"/>\n", score.metadata.files_scanned));
    xml.push_str(&format!("    <property name=\"project.lines_of_code\" value=\"{}\"/>\n", score.metadata.lines_of_code));
    xml.push_str("  </properties>\n");

    // Overall score test case
    xml.push_str(&generate_overall_testcase(info, score));

    // Dimension test cases
    xml.push_str(&generate_dimension_testcases(&score.dimensions));

    // Issue test cases (grouped by severity)
    xml.push_str(&generate_issue_testcases(&score.dimensions));

    // Close testsuite
    xml.push_str("</testsuite>\n");

    xml
}

/// Generate overall score test case
fn generate_overall_testcase(info: &ProjectInfo, score: &ProjectScore) -> String {
    let mut xml = String::new();

    let is_passing = score.overall >= 70.0; // Threshold for passing
    let grade_letter = score.grade.as_letter();

    xml.push_str("  <testcase name=\"overall.score\" classname=\"lumen.quality\">\n");

    if !is_passing {
        xml.push_str(&format!(
            "    <failure message=\"Project score {:.1} (grade {}) is below passing threshold\" type=\"QualityGateFailure\">\n",
            score.overall, grade_letter
        ));
        xml.push_str(&format!("      Project: {}\n", info.name));
        xml.push_str(&format!("      Current Score: {:.1}/100\n", score.overall));
        xml.push_str(&format!("      Grade: {}\n", grade_letter));
        xml.push_str(&format!("      Threshold: 70.0\n"));
        xml.push_str("      Required actions:\n");

        // Add recommendations for failing score
        for (dim_name, dim) in score.dimensions.all() {
            if dim.score < 70.0 {
                xml.push_str(&format!("      - Improve {} (currently {:.1}/100)\n", dim_name, dim.score));
            }
        }

        xml.push_str("    </failure>\n");
    } else {
        xml.push_str(&format!(
            "    <system-out>Score: {:.1}/100 (Grade: {})</system-out>\n",
            score.overall, grade_letter
        ));
    }

    xml.push_str("  </testcase>\n");

    xml
}

/// Generate test cases for each dimension
fn generate_dimension_testcases(dimensions: &DimensionScores) -> String {
    let mut xml = String::new();

    let dim_data = [
        ("coverage", &dimensions.coverage, "Test Coverage"),
        ("quality", &dimensions.quality, "Code Quality"),
        ("performance", &dimensions.performance, "Performance"),
        ("security", &dimensions.security, "Security"),
        ("seo", &dimensions.seo, "SEO"),
        ("docs", &dimensions.docs, "Documentation"),
        ("uiux", &dimensions.uiux, "UI/UX"),
    ];

    for (id, dim, label) in dim_data {
        let threshold = 70.0;
        let is_passing = dim.score >= threshold;

        xml.push_str(&format!(
            "  <testcase name=\"{}.{}\" classname=\"lumen.dimensions\">\n",
            id, label.to_lowercase().replace(' ', "_")
        ));

        if !is_passing {
            xml.push_str(&format!(
                "    <failure message=\"{} score {:.1} is below threshold {}\" type=\"DimensionFailure\">\n",
                label, dim.score, threshold
            ));
            xml.push_str(&format!("      Grade: {}\n", dim.grade.as_letter()));
            xml.push_str(&format!("      Weight: {:.0}%\n", dim.weight * 100.0));

            // Add issue summary
            if !dim.issues.is_empty() {
                xml.push_str("      Issues found:\n");
                for issue in dim.issues.iter().take(5) {
                    xml.push_str(&format!("      - [{}] {}\n", issue.severity.as_str(), issue.title));
                }
                if dim.issues.len() > 5 {
                    xml.push_str(&format!("      - ... and {} more\n", dim.issues.len() - 5));
                }
            }

            xml.push_str("    </failure>\n");
        } else {
            xml.push_str(&format!(
                "    <system-out>Score: {:.1}/100 (Grade: {})</system-out>\n",
                dim.score, dim.grade.as_letter()
            ));
        }

        xml.push_str("  </testcase>\n");
    }

    xml
}

/// Generate test cases for critical issues
fn generate_issue_testcases(dimensions: &DimensionScores) -> String {
    let mut xml = String::new();

    // Collect all critical and high severity issues
    let mut critical_issues = Vec::new();
    let mut high_issues = Vec::new();

    for (dim_name, dim) in dimensions.all() {
        for issue in &dim.issues {
            match issue.severity {
                IssueSeverity::Critical => critical_issues.push((dim_name, issue)),
                IssueSeverity::High => high_issues.push((dim_name, issue)),
                _ => {}
            }
        }
    }

    // Critical issues as errors
    for (dim, issue) in critical_issues {
        xml.push_str(&format!(
            "  <testcase name=\"issue.{}\" classname=\"lumen.issues.critical\">\n",
            sanitize_name(&issue.id)
        ));

        xml.push_str(&format!(
            "    <error message=\"[CRITICAL] {}\" type=\"CriticalIssue\">\n",
            html_escape(&issue.title)
        ));
        xml.push_str(&format!("      Dimension: {}\n", dim));
        xml.push_str(&format!("      Severity: Critical\n"));
        xml.push_str(&format!("      Impact: -{:.1} points\n", issue.impact));

        if let Some(file) = &issue.file {
            xml.push_str(&format!("      Location: {}\n", file));
        }

        xml.push_str(&format!("      Description: {}\n", html_escape(&issue.description)));

        if let Some(suggestion) = &issue.suggestion {
            xml.push_str(&format!("      Suggested fix: {}\n", html_escape(suggestion)));
        }

        xml.push_str("    </error>\n");
        xml.push_str("  </testcase>\n");
    }

    // High severity issues as failures
    for (dim, issue) in high_issues {
        xml.push_str(&format!(
            "  <testcase name=\"issue.{}\" classname=\"lumen.issues.high\">\n",
            sanitize_name(&issue.id)
        ));

        xml.push_str(&format!(
            "    <failure message=\"[HIGH] {}\" type=\"HighPriorityIssue\">\n",
            html_escape(&issue.title)
        ));
        xml.push_str(&format!("      Dimension: {}\n", dim));
        xml.push_str(&format!("      Severity: High\n"));
        xml.push_str(&format!("      Impact: -{:.1} points\n", issue.impact));

        if let Some(file) = &issue.file {
            xml.push_str(&format!("      Location: {}\n", file));
        }

        xml.push_str(&format!("      Description: {}\n", html_escape(&issue.description)));

        if let Some(suggestion) = &issue.suggestion {
            xml.push_str(&format!("      Suggested fix: {}\n", html_escape(suggestion)));
        }

        xml.push_str("    </failure>\n");
        xml.push_str("  </testcase>\n");
    }

    xml
}

/// Count total tests (dimensions + overall)
fn count_total_tests(dimensions: &DimensionScores, _overall: f64) -> i32 {
    let mut count = 8; // 7 dimensions + 1 overall

    // Count critical and high issues as tests
    for (_, dim) in dimensions.all() {
        count += dim.issues.iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Critical | IssueSeverity::High))
            .count() as i32;
    }

    count
}

/// Count failures (dimensions below threshold + critical/high issues)
fn count_failures(dimensions: &DimensionScores, overall: f64) -> i32 {
    let mut count = 0;

    // Overall failure
    if overall < 70.0 {
        count += 1;
    }

    // Dimension failures
    for (_, dim) in dimensions.all() {
        if dim.score < 70.0 {
            count += 1;
        }
    }

    // Critical and high issues
    for (_, dim) in dimensions.all() {
        count += dim.issues.iter()
            .filter(|i| matches!(i.severity, IssueSeverity::Critical | IssueSeverity::High))
            .count() as i32;
    }

    count
}

/// Count critical failures (for errors attribute)
fn count_critical_failures(dimensions: &DimensionScores, _overall: f64) -> i32 {
    let mut count = 0;

    for (_, dim) in dimensions.all() {
        count += dim.issues.iter()
            .filter(|i| i.severity == IssueSeverity::Critical)
            .count() as i32;
    }

    count
}

/// Format timestamp for JUnit XML
fn format_timestamp_junit(timestamp: i64) -> String {
    if let Some(dt) = std::time::SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(timestamp as u64)) {
        let dt: chrono::DateTime<chrono::Utc> = dt.into();
        dt.format("%Y-%m-%dT%H:%M:%S").to_string()
    } else {
        String::new()
    }
}

/// Sanitize name for use in XML attributes
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' { c } else { '_' })
        .collect()
}

/// Escape HTML entities
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumenx_score::{DimensionScore, ScoreMetadata};
    use std::collections::HashMap;

    #[test]
    fn test_junit_output_valid_xml() {
        let info = mock_project_info();
        let score = mock_project_score();

        let xml = generate(&info, &score);

        // Basic XML validation
        assert!(xml.starts_with("<?xml version=\"1.0\""));
        assert!(xml.contains("<testsuite"));
        assert!(xml.contains("</testsuite>"));
    }

    #[test]
    fn test_junit_contains_testcases() {
        let info = mock_project_info();
        let score = mock_project_score();

        let xml = generate(&info, &score);

        assert!(xml.contains("<testcase"));
        assert!(xml.contains("classname=\"lumen"));
    }

    #[test]
    fn test_junit_escaping() {
        assert_eq!(html_escape("test & quote"), "test &amp; quote");
        assert_eq!(html_escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(html_escape("\"quoted\""), "&quot;quoted&quot;");
    }

    fn mock_project_info() -> ProjectInfo {
        use lumenx_core::{Framework, Language, ProjectInfo, TestRunner};
        use std::path::PathBuf;

        ProjectInfo {
            name: "test-project".to_string(),
            root: PathBuf::from("/test"),
            framework: Framework::NextJs,
            language: Language::TypeScript,
            test_runner: TestRunner::Vitest,
            package_manager: Some("npm".to_string()),
            dependencies: vec!["react".to_string()],
            dev_dependencies: vec!["vitest".to_string()],
            database: None,
            package_json: None,
            cargo_dependencies: None,
        }
    }

    fn mock_project_score() -> ProjectScore {
        use lumenx_score::{ScoreIssue, IssueSeverity};

        ProjectScore {
            project_name: "test-project".to_string(),
            commit_sha: "abc123".to_string(),
            timestamp: 1234567890,
            overall: 75.0,
            grade: Grade::B,
            dimensions: DimensionScores {
                coverage: DimensionScore::new("coverage".to_string(), 80.0, 0.25),
                quality: DimensionScore::new("quality".to_string(), 70.0, 0.20),
                performance: DimensionScore::new("performance".to_string(), 85.0, 0.15),
                security: DimensionScore::new("security".to_string(), 75.0, 0.15),
                seo: DimensionScore::new("seo".to_string(), 60.0, 0.10),
                docs: DimensionScore::new("docs".to_string(), 50.0, 0.05),
                uiux: DimensionScore::new("uiux".to_string(), 80.0, 0.10),
            },
            trend: None,
            metadata: ScoreMetadata {
                scorer_version: "0.5.0".to_string(),
                scan_duration_ms: 1500,
                files_scanned: 100,
                lines_of_code: 10000,
                language_breakdown: HashMap::new(),
                profile: "standard".to_string(),
            },
        }
    }
}
