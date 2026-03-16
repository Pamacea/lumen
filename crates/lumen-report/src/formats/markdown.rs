//! Markdown report generation

use lumen_core::ProjectInfo;
use lumen_score::{
    DimensionScores, Grade, IssueSeverity, ProjectScore, ScoreIssue,
};
use std::fmt::Write;

/// Generate comprehensive markdown report
pub fn generate(info: &ProjectInfo, score: &ProjectScore) -> String {
    let mut md = String::new();

    // Header with title and grade badge
    write_header(&mut md, info, score);

    // Overall score visualization
    write_score_card(&mut md, score);

    // Dimension breakdown
    write_dimensions(&mut md, &score.dimensions);

    // Issues section
    write_issues(&mut md, &score.dimensions);

    // Trend analysis (if available)
    if let Some(trend) = &score.trend {
        write_trend(&mut md, trend);
    }

    // Metadata
    write_metadata(&mut md, info, score);

    // Recommendations
    write_recommendations(&mut md, &score.dimensions);

    md
}

/// Generate AI-friendly fixes report
pub fn generate_fixes(info: &ProjectInfo, score: &ProjectScore) -> String {
    let mut md = String::new();

    writeln!(md, "# Lumen Fixes Report").unwrap();
    writeln!(md).unwrap();
    writeln!(md, "**Project:** {}", info.name).unwrap();
    writeln!(md, "**Grade:** {} ({:.1}/100)", score.grade.as_letter(), score.overall).unwrap();
    writeln!(md, "**Generated:** {}", timestamp_to_readable(score.timestamp)).unwrap();
    writeln!(md).unwrap();
    writeln!(md, "---").unwrap();
    writeln!(md).unwrap();

    // Collect all issues across dimensions
    let mut all_issues = Vec::new();
    for (dim_name, dim_score) in score.dimensions.all() {
        for issue in &dim_score.issues {
            all_issues.push((dim_name.to_string(), issue.clone()));
        }
    }

    // Sort by severity (critical first)
    all_issues.sort_by(|a, b| b.1.severity.cmp(&a.1.severity));

    if all_issues.is_empty() {
        writeln!(md, "> No issues found! Your project is in great shape.").unwrap();
        return md;
    }

    // Group by severity
    let critical: Vec<_> = all_issues.iter().filter(|(_, i)| i.severity == IssueSeverity::Critical).collect();
    let high: Vec<_> = all_issues.iter().filter(|(_, i)| i.severity == IssueSeverity::High).collect();
    let medium: Vec<_> = all_issues.iter().filter(|(_, i)| i.severity == IssueSeverity::Medium).collect();
    let low: Vec<_> = all_issues.iter().filter(|(_, i)| i.severity == IssueSeverity::Low).collect();
    let info_issues: Vec<_> = all_issues.iter().filter(|(_, i)| i.severity == IssueSeverity::Info).collect();

    // Summary table
    writeln!(md,"## Summary").unwrap();
    writeln!(md).unwrap();
    writeln!(md,"| Severity | Count |").unwrap();
    writeln!(md,"|----------|-------|").unwrap();
    writeln!(md,"| CRITICAL | {} |", critical.len()).unwrap();
    writeln!(md,"| High | {} |", high.len()).unwrap();
    writeln!(md,"| Medium | {} |", medium.len()).unwrap();
    writeln!(md,"| Low | {} |", low.len()).unwrap();
    writeln!(md,"| Info | {} |", info_issues.len()).unwrap();
    writeln!(md).unwrap();

    // Critical issues
    if !critical.is_empty() {
        writeln!(md,"## CRITICAL Issues").unwrap();
        writeln!(md,"> These issues must be fixed immediately. They represent critical failures.").unwrap();
        writeln!(md).unwrap();
        for (dim, issue) in critical {
            write_fix_issue(&mut md, dim, issue);
        }
    }

    // High priority issues
    if !high.is_empty() {
        writeln!(md,"## High Priority Issues").unwrap();
        writeln!(md,"> These issues should be fixed soon. They significantly impact quality.").unwrap();
        writeln!(md).unwrap();
        for (dim, issue) in high {
            write_fix_issue(&mut md, dim, issue);
        }
    }

    // Medium priority issues
    if !medium.is_empty() {
        writeln!(md,"## Medium Priority Issues").unwrap();
        writeln!(md,"> These issues should be addressed for better code quality.").unwrap();
        writeln!(md).unwrap();
        for (dim, issue) in medium {
            write_fix_issue(&mut md, dim, issue);
        }
    }

    // Low priority issues
    if !low.is_empty() {
        writeln!(md,"## Low Priority Issues").unwrap();
        writeln!(md,"> Minor issues that can be cleaned up over time.").unwrap();
        writeln!(md).unwrap();
        for (dim, issue) in low {
            write_fix_issue(&mut md, dim, issue);
        }
    }

    // Quick fix commands section
    writeln!(md).unwrap();
    writeln!(md,"---").unwrap();
    writeln!(md,"## Quick Fix Commands").unwrap();
    writeln!(md).unwrap();
    writeln!(md,"```bash").unwrap();
    writeln!(md,"# Re-analyze after fixes").unwrap();
    writeln!(md,"lumen scan").unwrap();
    writeln!(md,"").unwrap();
    writeln!(md,"# Generate new report").unwrap();
    writeln!(md,"lumen report").unwrap();
    writeln!(md,"```").unwrap();

    md
}

/// Write a single fix issue entry
fn write_fix_issue(md: &mut String, dimension: &str, issue: &ScoreIssue) {
    writeln!(md,"### {}", issue.title).unwrap();
    writeln!(md).unwrap();

    // Metadata badges
    write!(md,"**Dimension:** {} | ", dimension).unwrap();
    writeln!(md,"**Severity:** {} | **Impact:** -{:.1} pts ", issue.severity.as_str(), issue.impact).unwrap();
    writeln!(md).unwrap();

    // Location if available
    if let Some(file) = &issue.file {
        writeln!(md,"**Location:** '{}'", file).unwrap();
        if let Some(line) = issue.line {
            writeln!(md,":{}'", line).unwrap();
        } else {
            writeln!(md,"'").unwrap();
        }
        writeln!(md).unwrap();
        writeln!(md).unwrap();
    }

    // Description
    writeln!(md,"**Description:**").unwrap();
    writeln!(md,"{}", issue.description).unwrap();
    writeln!(md).unwrap();

    // Suggested fix
    if let Some(suggestion) = &issue.suggestion {
        writeln!(md,"**Suggested Fix:**").unwrap();
        writeln!(md,"CODE_BLOCK_START").unwrap();
        writeln!(md,"{}", suggestion).unwrap();
        writeln!(md,"CODE_BLOCK_END").unwrap();
        writeln!(md).unwrap();
    }

    // AI Agent specific instructions
    writeln!(md,"**For AI Agents:**").unwrap();
    if let Some(file) = &issue.file {
        writeln!(md,"BASH_CODE_BLOCK_START").unwrap();
        write!(md,"# View the problematic code").unwrap();
        writeln!(md).unwrap();
        writeln!(md,"cat -n {} | head -{}", file, issue.line.unwrap_or(10) + 5).unwrap();
        writeln!(md,"CODE_BLOCK_END").unwrap();
    }
    writeln!(md).unwrap();
}

fn write_header(md: &mut String, _info: &ProjectInfo, score: &ProjectScore) {
    writeln!(md,"# Lumen Code Analysis Report").unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "> **{grade} Grade** - {label} - {score:.1}/100",
        grade = grade_badge(score.grade),
        label = score.grade.label(),
        score = score.overall
    )
    .unwrap();
    writeln!(md).unwrap();
}

fn write_score_card(md: &mut String, score: &ProjectScore) {
    writeln!(md,"## Score Overview").unwrap();
    writeln!(md).unwrap();

    // Score bar
    let filled = (score.overall / 100.0 * 20.0).round() as usize;
    let empty = 20 - filled;
    let _color = score_bar_color(score.overall);

    writeln!(md,"### Overall Score: **{:.1}** / 100", score.overall).unwrap();
    writeln!(md).unwrap();
    writeln!(
        md,
        "{}{}{}",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
        format!(" {:.0}%", score.overall)
    )
    .unwrap();
    writeln!(md).unwrap();

    // Quick stats
    writeln!(md,"| Metric | Value |").unwrap();
    writeln!(md,"|--------|-------|").unwrap();
    writeln!(md,"| **Grade** | {} |", score.grade.as_letter()).unwrap();
    writeln!(md,"| **GPA** | {:.2} / 4.0 |", score.grade.gpa_value()).unwrap();
    writeln!(
        md,
        "| **Status** | {} |",
        if score.overall >= 80.0 {
            "Excellent"
        } else if score.overall >= 70.0 {
            "Good"
        } else if score.overall >= 60.0 {
            "Fair"
        } else {
            "Needs Improvement"
        }
    )
    .unwrap();
    writeln!(md).unwrap();
}

fn write_dimensions(md: &mut String, dimensions: &DimensionScores) {
    writeln!(md,"## Dimension Breakdown").unwrap();
    writeln!(md).unwrap();

    let dim_data = [
        ("Coverage", &dimensions.coverage, 25, "\u{1F4A8}"),
        ("Quality", &dimensions.quality, 20, "\u{1F50D}"),
        ("Performance", &dimensions.performance, 15, "\u{26A1}"),
        ("Security", &dimensions.security, 15, "\u{1F512}"),
        ("SEO", &dimensions.seo, 10, "\u{1F50D}"),
        ("Documentation", &dimensions.docs, 5, "\u{1F4DA}"),
        ("UI/UX", &dimensions.uiux, 10, "\u{1F3A8}"),
    ];

    for (name, dim, weight, icon) in dim_data {
        writeln!(md,"### {} {}", icon, name).unwrap();
        writeln!(md).unwrap();

        // Score bar
        let filled = (dim.score / 100.0 * 20.0).round() as usize;
        let empty = 20 - filled;
        writeln!(
            md,
            "**{:.1}** / 100 (Weight: {}%)",
            dim.score, weight
        )
        .unwrap();
        writeln!(md).unwrap();
        writeln!(
            md,
            "{}{} {}",
            "\u{2588}".repeat(filled),
            "\u{2591}".repeat(empty),
            format!(" {:.0}%", dim.score)
        )
        .unwrap();
        writeln!(md).unwrap();

        // Grade badge
        writeln!(
            md,
            "*Grade: {} - {}*",
            dim.grade.as_letter(),
            dim.grade.label()
        )
        .unwrap();
        writeln!(md).unwrap();

        // Issues count
        if !dim.issues.is_empty() {
            writeln!(
                md,
                "<details><summary>{n} issue(s) found</summary>",
                n = dim.issues.len()
            )
            .unwrap();
            writeln!(md).unwrap();
            for issue in &dim.issues {
                writeln!(
                    md,
                    "- **{}** {} - *{}*",
                    severity_icon(issue.severity),
                    issue.title,
                    issue.severity.as_str()
                )
                .unwrap();
            }
            writeln!(md,"</details>").unwrap();
            writeln!(md).unwrap();
        }

        // Metrics preview
        if !dim.metrics.is_empty() {
            writeln!(md,"<details><summary>View metrics</summary>").unwrap();
            writeln!(md).unwrap();
            writeln!(md,"| Metric | Value |").unwrap();
            writeln!(md,"|--------|-------|").unwrap();
            for (key, value) in &dim.metrics {
                writeln!(md,"| {} | {} |", key, format_metric(value)).unwrap();
            }
            writeln!(md,"</details>").unwrap();
            writeln!(md).unwrap();
        }
    }
}

fn write_issues(md: &mut String, dimensions: &DimensionScores) {
    // Collect all issues
    let mut all_issues: Vec<_> = dimensions
        .all()
        .iter()
        .flat_map(|(name, dim)| {
            dim.issues.iter().map(move |issue| (name.to_string(), issue))
        })
        .collect();

    // Sort by severity
    all_issues.sort_by(|a, b| b.1.severity.cmp(&a.1.severity));

    if all_issues.is_empty() {
        writeln!(md,"## Issues").unwrap();
        writeln!(md).unwrap();
        writeln!(md,"> No issues found! Your project is in great shape.").unwrap();
        writeln!(md).unwrap();
        return;
    }

    writeln!(md,"## Issues ({})", all_issues.len()).unwrap();
    writeln!(md).unwrap();

    // Group by severity
    let by_severity = |sev: IssueSeverity| -> Vec<_> {
        all_issues
            .iter()
            .filter(|(_, i)| i.severity == sev)
            .collect()
    };

    for severity in [
        IssueSeverity::Critical,
        IssueSeverity::High,
        IssueSeverity::Medium,
        IssueSeverity::Low,
    ] {
        let issues = by_severity(severity);
        if !issues.is_empty() {
            writeln!(md,"### {}", severity_title(severity)).unwrap();
            writeln!(md).unwrap();

            for (dim, issue) in issues {
                writeln!(md,"#### {} {}", severity_icon(issue.severity), issue.title).unwrap();
                writeln!(md).unwrap();
                writeln!(md,"**{}** - *{}*", dim, issue.severity.as_str()).unwrap();
                writeln!(md,"> {}", issue.description).unwrap();
                writeln!(md).unwrap();

                if let Some(file) = &issue.file {
                    writeln!(md,"<details><summary>Location: {}</summary>", file).unwrap();
                    writeln!(md).unwrap();
                    writeln!(md,"CODE_BLOCK_START").unwrap();
                    if let Some(suggestion) = &issue.suggestion {
                        writeln!(md,"{}", suggestion).unwrap();
                    }
                    writeln!(md,"CODE_BLOCK_END").unwrap();
                    writeln!(md,"</details>").unwrap();
                    writeln!(md).unwrap();
                }
            }
        }
    }
}

fn write_trend(md: &mut String, trend: &lumen_score::TrendAnalysis) {
    writeln!(md, "## Trend Analysis").unwrap();
    writeln!(md).unwrap();

    // Direction indicator
    let (icon, description) = match trend.direction {
        lumen_score::TrendDirection::Improving => ("📈", "Score is improving"),
        lumen_score::TrendDirection::Stable => ("➡️", "Score is stable"),
        lumen_score::TrendDirection::Declining => ("📉", "Score is declining"),
    };

    writeln!(md, "### {} {}", icon, description).unwrap();
    writeln!(md).unwrap();

    if let Some(prev) = trend.previous {
        let delta = trend.current - prev;
        writeln!(
            md,
            "- **Previous:** {:.1}", prev
        )
        .unwrap();
        writeln!(
            md,
            "- **Current:** {:.1}", trend.current
        )
        .unwrap();
        writeln!(
            md,
            "- **Change:** {:+.1} points", delta
        )
        .unwrap();
    } else {
        writeln!(md, "No historical data available for comparison.").unwrap();
    }
    writeln!(md).unwrap();

    // Predictions
    if let Some(pred) = &trend.prediction {
        writeln!(md, "<details><summary>Score Predictions</summary>").unwrap();
        writeln!(md).unwrap();
        writeln!(
            md,
            "- **7-day prediction:** {:.1} ({})", pred.score_7d, pred.grade_7d.as_letter()
        )
        .unwrap();
        writeln!(
            md,
            "- **30-day prediction:** {:.1} ({})", pred.score_30d, pred.grade_30d.as_letter()
        )
        .unwrap();
        writeln!(
            md,
            "- **Confidence:** {:.0}%", pred.confidence * 100.0
        )
        .unwrap();
        writeln!(md, "</details>").unwrap();
        writeln!(md).unwrap();
    }
}

fn write_metadata(md: &mut String, info: &ProjectInfo, score: &ProjectScore) {
    writeln!(md, "## Analysis Metadata").unwrap();
    writeln!(md).unwrap();

    writeln!(md, "| Property | Value |").unwrap();
    writeln!(md, "|----------|-------|").unwrap();
    writeln!(md, "| **Project** | {} |", info.name).unwrap();
    writeln!(md, "| **Framework** | {} |", info.framework.display_name()).unwrap();
    writeln!(md, "| **Language** | {} |", info.language.display_name()).unwrap();
    writeln!(md, "| **Test Runner** | {} |", info.test_runner.display_name()).unwrap();
    writeln!(md, "| **Lumen Version** | {} |", score.metadata.scorer_version).unwrap();
    writeln!(md, "| **Scan Duration** | {} ms |", score.metadata.scan_duration_ms).unwrap();
    writeln!(
        md,
        "| **Files Scanned** | {} |",
        score.metadata.files_scanned
    )
    .unwrap();
    writeln!(
        md,
        "| **Lines of Code** | {} |",
        score.metadata.lines_of_code
    )
    .unwrap();
    writeln!(
        md,
        "| **Generated** | {} |",
        timestamp_to_readable(score.timestamp)
    )
    .unwrap();
    writeln!(md).unwrap();
}

fn write_recommendations(md: &mut String, dimensions: &DimensionScores) {
    writeln!(md, "## Recommendations").unwrap();
    writeln!(md).unwrap();

    let mut all_improvements: Vec<_> = dimensions
        .all()
        .iter()
        .flat_map(|(name, dim)| {
            dim.improvements
                .iter()
                .map(move |imp| (name.to_string(), imp))
        })
        .collect();

    // Sort by impact and effort
    all_improvements.sort_by(|a, b| {
        b.1.impact
            .points()
            .0
            .partial_cmp(&a.1.impact.points().0)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if all_improvements.is_empty() {
        writeln!(
            md,
            "> Your project is well-maintained! No specific recommendations at this time."
        )
        .unwrap();
        return;
    }

    // Top 5 high-impact improvements
    writeln!(md, "### High Impact, Low Effort").unwrap();
    writeln!(md).unwrap();

    let quick_wins: Vec<_> = all_improvements
        .iter()
        .filter(|(_, imp)| {
            matches!(imp.effort, lumen_score::Effort::Trivial | lumen_score::Effort::Low)
                && matches!(
                    imp.impact,
                    lumen_score::Impact::Medium | lumen_score::Impact::High
                )
        })
        .take(5)
        .collect();

    if quick_wins.is_empty() {
        writeln!(md, "No quick wins identified.").unwrap();
    } else {
        for (dim, imp) in quick_wins {
            writeln!(md, "#### {}", imp.title).unwrap();
            writeln!(md, "- **Dimension:** {}", dim).unwrap();
            writeln!(md, "- **Effort:** {}", imp.effort.as_str()).unwrap();
            writeln!(md, "- **Impact:** {}", imp.impact.as_str()).unwrap();
            writeln!(md, "> {}", imp.description).unwrap();
            writeln!(md).unwrap();
        }
    }
}

// Helper functions

fn grade_badge(grade: Grade) -> String {
    match grade {
        Grade::APlus => "🟢 A+".to_string(),
        Grade::A => "🟢 A".to_string(),
        Grade::AMinus => "🟢 A-".to_string(),
        Grade::BPlus => "🔵 B+".to_string(),
        Grade::B => "🔵 B".to_string(),
        Grade::BMinus => "🔵 B-".to_string(),
        Grade::CPlus => "🟡 C+".to_string(),
        Grade::C => "🟡 C".to_string(),
        Grade::CMinus => "🟡 C-".to_string(),
        Grade::DPlus => "🟠 D+".to_string(),
        Grade::D => "🟠 D".to_string(),
        Grade::F => "🔴 F".to_string(),
    }
}

fn score_bar_color(score: f64) -> &'static str {
    if score >= 90.0 {
        "🟢"
    } else if score >= 80.0 {
        "🔵"
    } else if score >= 70.0 {
        "🟡"
    } else if score >= 60.0 {
        "🟠"
    } else {
        "🔴"
    }
}

fn severity_icon(severity: IssueSeverity) -> &'static str {
    match severity {
        IssueSeverity::Critical => "🚨",
        IssueSeverity::High => "⚠️",
        IssueSeverity::Medium => "⚡",
        IssueSeverity::Low => "💡",
        IssueSeverity::Info => "ℹ️",
    }
}

fn severity_title(severity: IssueSeverity) -> &'static str {
    match severity {
        IssueSeverity::Critical => "CRITICAL",
        IssueSeverity::High => "High Priority",
        IssueSeverity::Medium => "Medium Priority",
        IssueSeverity::Low => "Low Priority",
        IssueSeverity::Info => "Info",
    }
}

fn format_metric(value: &lumen_score::MetricValue) -> String {
    match value {
        lumen_score::MetricValue::Float(v) => format!("{:.2}", v),
        lumen_score::MetricValue::Integer(v) => format!("{}", v),
        lumen_score::MetricValue::Percentage(v) => format!("{:.1}%", v),
        lumen_score::MetricValue::Duration(v) => format!("{}ms", v),
        lumen_score::MetricValue::Count(v) => format!("{}", v),
        lumen_score::MetricValue::Boolean(v) => format!("{}", if *v { "Yes" } else { "No" }),
        lumen_score::MetricValue::String(v) => v.clone(),
    }
}

fn timestamp_to_readable(timestamp: i64) -> String {
    use std::time::SystemTime;

    if let Some(datetime) = SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(timestamp as u64)) {
        let datetime: chrono::DateTime<chrono::Utc> = datetime.into();
        datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else {
        format!("{} (timestamp)", timestamp)
    }
}
