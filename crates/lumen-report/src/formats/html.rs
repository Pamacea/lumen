//! HTML report generation

use lumen_core::ProjectInfo;
use lumen_score::{DimensionScores, Grade, IssueSeverity, ProjectScore};

/// Generate comprehensive HTML report with embedded CSS and JavaScript
pub fn generate(info: &ProjectInfo, score: &ProjectScore) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Lumen Report - {name}</title>
    <style>{css}</style>
</head>
<body>
    <div class="container">
        <header>{header}</header>
        <main>{main_content}</main>
        <footer>{footer}</footer>
    </div>
    <script>{script}</script>
</body>
</html>"#,
        name = html_escape(&info.name),
        css = CSS_STYLES,
        header = render_header(info, score),
        main_content = render_main(info, score),
        footer = render_footer(score),
        script = JAVASCRIPT
    )
}

fn render_header(info: &ProjectInfo, score: &ProjectScore) -> String {
    format!(
        r#"
<div class="header">
    <div class="header-left">
        <h1 class="title">
            <span class="logo">Lumen</span>
            <span class="subtitle">Code Analysis Report</span>
        </h1>
        <p class="project-info">
            <span class="framework-badge">{framework}</span>
            <span class="language-badge">{language}</span>
        </p>
    </div>
    <div class="header-right">
        <div class="score-card grade-{grade_class}">
            <div class="score-value">{score:.0}</div>
            <div class="score-grade">{grade_letter}</div>
            <div class="score-label">{grade_label}</div>
        </div>
    </div>
</div>
<div class="timestamp">
    Generated: {timestamp}
</div>
"#,
        framework = html_escape(info.framework.display_name()),
        language = html_escape(info.language.display_name()),
        grade_class = grade_class(score.grade),
        score = score.overall,
        grade_letter = score.grade.as_letter(),
        grade_label = score.grade.label(),
        timestamp = format_timestamp(score.timestamp)
    )
}

fn render_main(_info: &ProjectInfo, score: &ProjectScore) -> String {
    format!(
        r#"
{score_overview}
{dimensions_section}
{issues_section}
{trend_section}
{metrics_section}
{recommendations_section}
"#,
        score_overview = render_score_overview(score),
        dimensions_section = render_dimensions(&score.dimensions),
        issues_section = render_issues(&score.dimensions),
        trend_section = render_trend(score),
        metrics_section = render_metrics(&score.dimensions),
        recommendations_section = render_recommendations(&score.dimensions)
    )
}

fn render_score_overview(score: &ProjectScore) -> String {
    let percentage = score.overall;
    let color = score_color(score.overall);

    format!(
        r#"
<section class="score-overview">
    <h2>Overall Score</h2>
    <div class="score-bar-container">
        <div class="score-bar" style="width: {percentage}%; background: {color};"></div>
    </div>
    <div class="score-stats">
        <div class="stat">
            <span class="stat-label">Grade</span>
            <span class="stat-value grade-{grade_class}">{grade_letter}</span>
        </div>
        <div class="stat">
            <span class="stat-label">GPA</span>
            <span class="stat-value">{gpa:.2}</span>
        </div>
        <div class="stat">
            <span class="stat-label">Files</span>
            <span class="stat-value">{files}</span>
        </div>
        <div class="stat">
            <span class="stat-label">Lines</span>
            <span class="stat-value">{lines}</span>
        </div>
        <div class="stat">
            <span class="stat-label">Duration</span>
            <span class="stat-value">{duration}ms</span>
        </div>
    </div>
</section>
"#,
        percentage = percentage,
        color = color,
        grade_class = grade_class(score.grade),
        grade_letter = score.grade.as_letter(),
        gpa = score.grade.gpa_value(),
        files = score.metadata.files_scanned,
        lines = score.metadata.lines_of_code,
        duration = score.metadata.scan_duration_ms
    )
}

fn render_dimensions(dimensions: &DimensionScores) -> String {
    let dim_html = [
        ("coverage", &dimensions.coverage, 25, "Test Coverage", "📊"),
        ("quality", &dimensions.quality, 20, "Code Quality", "🔍"),
        ("performance", &dimensions.performance, 15, "Performance", "⚡"),
        ("security", &dimensions.security, 15, "Security", "🔒"),
        ("seo", &dimensions.seo, 10, "SEO", "🔍"),
        ("docs", &dimensions.docs, 5, "Documentation", "📚"),
        ("uiux", &dimensions.uiux, 10, "UI/UX", "🎨"),
    ]
    .iter()
    .map(|(id, dim, weight, label, icon)| {
        format!(
            r#"
<div class="dimension-card" data-dimension="{id}">
    <div class="dimension-header">
        <span class="dimension-icon">{icon}</span>
        <span class="dimension-name">{label}</span>
        <span class="dimension-weight">{weight}%</span>
    </div>
    <div class="dimension-score">
        <span class="dimension-score-value grade-{grade_class}">{score:.0}</span>
        <span class="dimension-score-max">/100</span>
    </div>
    <div class="dimension-bar-container">
        <div class="dimension-bar grade-{grade_class}" style="width: {score}%;"></div>
    </div>
    <div class="dimension-grade">{grade_letter} - {grade_label}</div>
    <div class="dimension-issues" data-dimension="{id}">
        <button class="expand-btn" onclick="toggleDimensionIssues('{id}')">
            <span class="issue-count">{issue_count}</span> issues found
            <span class="expand-icon">▼</span>
        </button>
        <div class="dimension-issues-list" id="issues-{id}">
            {issues_html}
        </div>
    </div>
</div>
"#,
            id = id,
            label = label,
            icon = icon,
            weight = weight,
            score = dim.score,
            grade_class = grade_class(dim.grade),
            grade_letter = dim.grade.as_letter(),
            grade_label = dim.grade.label(),
            issue_count = dim.issues.len(),
            issues_html = dim.issues.iter().map(|issue| render_issue(issue)).collect::<Vec<_>>().join("")
        )
    })
    .collect::<Vec<_>>()
    .join("");

    format!(
        r#"
<section class="dimensions">
    <h2>Dimension Breakdown</h2>
    <div class="dimensions-grid">
        {dim_html}
    </div>
</section>
"#
    )
}

fn render_issue(issue: &lumen_score::ScoreIssue) -> String {
    format!(
        r#"
<div class="issue severity-{sev_class}" onclick="toggleIssueDetails('{id}')">
    <div class="issue-header">
        <span class="issue-sev-icon">{sev_icon}</span>
        <span class="issue-title">{title}</span>
        <span class="issue-impact">-{impact:.0} pts</span>
    </div>
    <div class="issue-details" id="issue-{id}">
        <p class="issue-description">{description}</p>
        {location_html}
        {suggestion_html}
    </div>
</div>
"#,
        id = html_escape(&issue.id),
        sev_class = issue.severity.as_str(),
        sev_icon = severity_icon(issue.severity),
        title = html_escape(&issue.title),
        impact = issue.impact,
        description = html_escape(&issue.description),
        location_html = if let Some(file) = &issue.file {
            format!(
                r#"<div class="issue-location"><strong>Location:</strong> <code>{}</code></div>"#,
                html_escape(file)
            )
        } else {
            String::new()
        },
        suggestion_html = if let Some(suggestion) = &issue.suggestion {
            format!(
                r#"<div class="issue-suggestion"><strong>Suggested fix:</strong><pre><code>{}</code></pre></div>"#,
                html_escape(suggestion)
            )
        } else {
            String::new()
        }
    )
}

fn render_issues(dimensions: &DimensionScores) -> String {
    let mut all_issues: Vec<_> = dimensions
        .all()
        .iter()
        .flat_map(|(name, dim)| {
            dim.issues.iter().map(move |issue| (name.to_string(), issue.clone()))
        })
        .collect();

    all_issues.sort_by(|a, b| b.1.severity.cmp(&a.1.severity));

    if all_issues.is_empty() {
        return r#"
<section class="issues">
    <h2>Issues Found</h2>
    <div class="no-issues">
        <div class="no-issues-icon">🎉</div>
        <p>No issues found! Your project is in great shape.</p>
    </div>
</section>
"#.to_string();
    }

    let issues_html = all_issues
        .iter()
        .filter(|(_, issue)| {
            matches!(issue.severity, IssueSeverity::Critical | IssueSeverity::High | IssueSeverity::Medium)
        })
        .map(|(dim, issue)| {
            format!(
                r#"
<tr class="issue-row severity-{}" onclick="openIssueModal('{}')">
    <td><span class="severity-badge severity-{}">{}</span></td>
    <td>{}</td>
    <td>{}</td>
    <td>{} pts</td>
    <td>{}</td>
</tr>
"#,
                issue.severity.as_str(),
                html_escape(&issue.id),
                issue.severity.as_str(),
                severity_icon(issue.severity),
                html_escape(&issue.title),
                dim,
                issue.impact,
                if issue.file.is_some() {
                    format!(r#"<code>{}</code>"#, html_escape(issue.file.as_deref().unwrap_or("")))
                } else {
                    String::from("-")
                }
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"
<section class="issues">
    <h2>Issues Found <span class="count">({})</span></h2>
    <div class="issues-filters">
        <button class="filter-btn active" data-filter="all">All</button>
        <button class="filter-btn" data-filter="critical">Critical</button>
        <button class="filter-btn" data-filter="high">High</button>
        <button class="filter-btn" data-filter="medium">Medium</button>
        <button class="filter-btn" data-filter="low">Low</button>
    </div>
    <table class="issues-table">
        <thead>
            <tr>
                <th>Severity</th>
                <th>Issue</th>
                <th>Dimension</th>
                <th>Impact</th>
                <th>Location</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>
</section>
"#,
        all_issues.len(),
        issues_html
    )
}

fn render_trend(score: &ProjectScore) -> String {
    if let Some(trend) = &score.trend {
        let (icon, class, text) = match trend.direction {
            lumen_score::TrendDirection::Improving => ("📈", "improving", "Score is improving"),
            lumen_score::TrendDirection::Stable => ("➡️", "stable", "Score is stable"),
            lumen_score::TrendDirection::Declining => ("📉", "declining", "Score is declining"),
        };

        format!(
            r#"
<section class="trend">
    <h2>Trend Analysis</h2>
    <div class="trend-card trend-{class}">
        <div class="trend-icon">{icon}</div>
        <div class="trend-info">
            <div class="trend-title">{text}</div>
            <div class="trend-values">
                <span class="trend-current">{current:.1}</span>
                {prev_html}
            </div>
        </div>
    </div>
    {prediction_html}
</section>
"#,
            class = class,
            icon = icon,
            text = text,
            current = trend.current,
            prev_html = if let Some(prev) = trend.previous {
                format!(
                    r#"<span class="trend-delta {delta_class}">{delta:+.1}</span>"#,
                    delta = trend.current - prev,
                    delta_class = if (trend.current - prev) >= 0.0 { "positive" } else { "negative" }
                )
            } else {
                String::new()
            },
            prediction_html = if let Some(pred) = &trend.prediction {
                format!(
                    r#"<div class="prediction">
    <div class="prediction-title">Predictions</div>
    <div class="prediction-grid">
        <div class="prediction-item">
            <span class="prediction-label">7 days</span>
            <span class="prediction-value">{score_7d:.0} ({grade_7d})</span>
        </div>
        <div class="prediction-item">
            <span class="prediction-label">30 days</span>
            <span class="prediction-value">{score_30d:.0} ({grade_30d})</span>
        </div>
        <div class="prediction-item">
            <span class="prediction-label">Confidence</span>
            <span class="prediction-value">{conf:.0}%</span>
        </div>
    </div>
</div>"#,
                    score_7d = pred.score_7d,
                    grade_7d = pred.grade_7d.as_letter(),
                    score_30d = pred.score_30d,
                    grade_30d = pred.grade_30d.as_letter(),
                    conf = pred.confidence * 100.0
                )
            } else {
                String::new()
            }
        )
    } else {
        String::new()
    }
}

fn render_metrics(dimensions: &DimensionScores) -> String {
    let metrics_html = dimensions
        .all()
        .iter()
        .filter(|(_, dim)| !dim.metrics.is_empty())
        .map(|(name, dim)| {
            let metrics = dim
                .metrics
                .iter()
                .map(|(key, value)| {
                    format!(
                        r#"<tr><td>{}</td><td>{}</td></tr>"#,
                        html_escape(key),
                        format_metric_value(value)
                    )
                })
                .collect::<Vec<_>>()
                .join("");

            format!(
                r#"
<div class="metrics-dimension">
    <h3>{}</h3>
    <table class="metrics-table"><tbody>{}</tbody></table>
</div>
"#,
                name.to_uppercase(),
                metrics
            )
        })
        .collect::<Vec<_>>()
        .join("");

    if metrics_html.is_empty() {
        return String::new();
    }

    format!(
        r#"
<section class="metrics">
    <h2>Metrics</h2>
    <div class="metrics-grid">
        {}
    </div>
</section>
"#,
        metrics_html
    )
}

fn render_recommendations(dimensions: &DimensionScores) -> String {
    let mut all_improvements: Vec<_> = dimensions
        .all()
        .iter()
        .flat_map(|(name, dim)| {
            dim.improvements
                .iter()
                .map(move |imp| (name.to_string(), imp))
        })
        .collect();

    all_improvements.sort_by(|a, b| {
        // Sort by impact priority (higher impact first)
        let impact_value = |imp: &lumen_score::Impact| -> u8 {
            match imp {
                lumen_score::Impact::VeryHigh => 5,
                lumen_score::Impact::High => 4,
                lumen_score::Impact::Medium => 3,
                lumen_score::Impact::Low => 2,
                lumen_score::Impact::Minimal => 1,
            }
        };
        impact_value(&b.1.impact).cmp(&impact_value(&a.1.impact))
    });

    let quick_wins: Vec<_> = all_improvements
        .iter()
        .filter(|(_, imp)| {
            matches!(
                imp.effort,
                lumen_score::Effort::Trivial | lumen_score::Effort::Low
            ) && matches!(imp.impact, lumen_score::Impact::Medium | lumen_score::Impact::High)
        })
        .take(5)
        .collect();

    if quick_wins.is_empty() {
        return r#"
<section class="recommendations">
    <h2>Recommendations</h2>
    <p class="no-recommendations">Your project is well-maintained! No specific recommendations at this time.</p>
</section>
"#
        .to_string();
    }

    let recommendations_html = quick_wins
        .iter()
        .map(|(dim, imp)| {
            format!(
                r#"
<div class="recommendation-card">
    <div class="rec-header">
        <h3>{title}</h3>
        <div class="rec-badges">
            <span class="rec-badge effort-{effort}">{effort}</span>
            <span class="rec-badge impact-{impact}">{impact}</span>
        </div>
    </div>
    <p class="rec-description">{description}</p>
    <span class="rec-dimension">{dimension}</span>
</div>
"#,
                title = html_escape(&imp.title),
                effort = imp.effort.as_str(),
                impact = imp.impact.as_str(),
                description = html_escape(&imp.description),
                dimension = dim
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        r#"
<section class="recommendations">
    <h2>Quick Wins</h2>
    <p class="rec-subtitle">High impact, low effort improvements</p>
    <div class="recommendations-grid">
        {}
    </div>
</section>
"#,
        recommendations_html
    )
}

fn render_footer(score: &ProjectScore) -> String {
    format!(
        r#"
<div class="footer">
    <p>Generated by <strong>Lumen v{}</strong></p>
    <p>Analysis completed in {}ms across {} files ({} lines of code)</p>
</div>
"#,
        score.metadata.scorer_version,
        score.metadata.scan_duration_ms,
        score.metadata.files_scanned,
        score.metadata.lines_of_code
    )
}

// Helper functions

fn grade_class(grade: Grade) -> &'static str {
    match grade {
        Grade::APlus | Grade::A | Grade::AMinus => "a",
        Grade::BPlus | Grade::B | Grade::BMinus => "b",
        Grade::CPlus | Grade::C | Grade::CMinus => "c",
        Grade::DPlus | Grade::D => "d",
        Grade::F => "f",
    }
}

fn score_color(score: f64) -> &'static str {
    if score >= 90.0 {
        return "#10b981";
    } else if score >= 80.0 {
        return "#3b82f6";
    } else if score >= 70.0 {
        return "#f59e0b";
    } else if score >= 60.0 {
        return "#f97316";
    } else {
        return "#ef4444";
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

fn format_metric_value(value: &lumen_score::MetricValue) -> String {
    match value {
        lumen_score::MetricValue::Float(v) => format!("{:.2}", v),
        lumen_score::MetricValue::Integer(v) => format!("{}", v),
        lumen_score::MetricValue::Percentage(v) => format!("{:.1}%", v),
        lumen_score::MetricValue::Duration(v) => format!("{}ms", v),
        lumen_score::MetricValue::Count(v) => format!("{}", v),
        lumen_score::MetricValue::Boolean(v) => (if *v { "Yes" } else { "No" }).to_string(),
        lumen_score::MetricValue::String(v) => v.clone(),
    }
}

fn format_timestamp(timestamp: i64) -> String {
    if let Some(dt) = std::time::SystemTime::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(timestamp as u64)) {
        let dt: chrono::DateTime<chrono::Utc> = dt.into();
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else {
        format!("{}", timestamp)
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

// Embedded CSS
const CSS_STYLES: &str = r#"
:root {
    --color-a: #10b981;
    --color-b: #3b82f6;
    --color-c: #f59e0b;
    --color-d: #f97316;
    --color-f: #ef4444;
    --bg-primary: #0f172a;
    --bg-secondary: #1e293b;
    --bg-tertiary: #334155;
    --text-primary: #f1f5f9;
    --text-secondary: #94a3b8;
    --border: #334155;
}

* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
    background: var(--bg-primary);
    color: var(--text-primary);
    line-height: 1.6;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
}

/* Header */
.header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 2rem 0;
    border-bottom: 1px solid var(--border);
    margin-bottom: 2rem;
    flex-wrap: wrap;
    gap: 1rem;
}

.title {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
}

.logo {
    font-size: 2rem;
    font-weight: 700;
    background: linear-gradient(135deg, #fbbf24, #f59e0b);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
}

.subtitle {
    font-size: 1rem;
    color: var(--text-secondary);
    font-weight: 400;
}

.project-info {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
}

.framework-badge, .language-badge {
    padding: 0.25rem 0.75rem;
    background: var(--bg-secondary);
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 500;
}

.score-card {
    text-align: center;
    padding: 1.5rem 2rem;
    border-radius: 1rem;
    min-width: 150px;
}

.score-card.grade-a { background: linear-gradient(135deg, rgba(16, 185, 129, 0.2), rgba(16, 185, 129, 0.1)); border: 1px solid var(--color-a); }
.score-card.grade-b { background: linear-gradient(135deg, rgba(59, 130, 246, 0.2), rgba(59, 130, 246, 0.1)); border: 1px solid var(--color-b); }
.score-card.grade-c { background: linear-gradient(135deg, rgba(245, 158, 11, 0.2), rgba(245, 158, 11, 0.1)); border: 1px solid var(--color-c); }
.score-card.grade-d, .score-card.grade-f { background: linear-gradient(135deg, rgba(239, 68, 68, 0.2), rgba(239, 68, 68, 0.1)); border: 1px solid var(--color-f); }

.score-value { font-size: 3rem; font-weight: 700; line-height: 1; }
.score-grade { font-size: 1.5rem; font-weight: 600; margin-top: 0.25rem; }
.score-label { font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.25rem; }

.timestamp {
    text-align: center;
    color: var(--text-secondary);
    font-size: 0.875rem;
    margin-bottom: 2rem;
}

/* Sections */
section {
    background: var(--bg-secondary);
    border-radius: 1rem;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
}

section h2 {
    font-size: 1.5rem;
    margin-bottom: 1.5rem;
}

/* Score Overview */
.score-bar-container {
    height: 12px;
    background: var(--bg-tertiary);
    border-radius: 9999px;
    overflow: hidden;
    margin-bottom: 1.5rem;
}

.score-bar {
    height: 100%;
    transition: width 0.5s ease;
    border-radius: 9999px;
}

.score-stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
    gap: 1rem;
}

.stat {
    text-align: center;
}

.stat-label { display: block; font-size: 0.75rem; color: var(--text-secondary); }
.stat-value { display: block; font-size: 1.25rem; font-weight: 600; }
.stat-value.grade-a, .stat-value.grade-b, .stat-value.grade-c,
.stat-value.grade-d, .stat-value.grade-f { font-weight: 700; }

.stat-value.grade-a { color: var(--color-a); }
.stat-value.grade-b { color: var(--color-b); }
.stat-value.grade-c { color: var(--color-c); }
.stat-value.grade-d { color: var(--color-d); }
.stat-value.grade-f { color: var(--color-f); }

/* Dimensions */
.dimensions-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 1rem;
}

.dimension-card {
    background: var(--bg-tertiary);
    border-radius: 0.75rem;
    padding: 1.25rem;
    transition: transform 0.2s ease;
}

.dimension-card:hover {
    transform: translateY(-2px);
}

.dimension-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 1rem;
}

.dimension-icon { font-size: 1.25rem; }
.dimension-name { flex: 1; font-weight: 600; }
.dimension-weight { font-size: 0.75rem; color: var(--text-secondary); }

.dimension-score {
    display: flex;
    align-items: baseline;
    gap: 0.25rem;
    margin-bottom: 0.75rem;
}

.dimension-score-value { font-size: 2rem; font-weight: 700; }
.dimension-score-max { font-size: 1rem; color: var(--text-secondary); }

.dimension-bar-container {
    height: 8px;
    background: var(--bg-secondary);
    border-radius: 9999px;
    overflow: hidden;
    margin-bottom: 0.75rem;
}

.dimension-bar {
    height: 100%;
    border-radius: 9999px;
    transition: width 0.5s ease;
}

.dimension-bar.grade-a { background: var(--color-a); }
.dimension-bar.grade-b { background: var(--color-b); }
.dimension-bar.grade-c { background: var(--color-c); }
.dimension-bar.grade-d, .dimension-bar.grade-f { background: var(--color-f); }

.dimension-grade {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
}

.dimension-issues button {
    width: 100%;
    padding: 0.5rem;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    color: var(--text-secondary);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    transition: all 0.2s ease;
}

.dimension-issues button:hover {
    background: var(--bg-secondary);
    color: var(--text-primary);
}

.expand-icon {
    transition: transform 0.2s ease;
}

.expand-btn.active .expand-icon {
    transform: rotate(180deg);
}

.dimension-issues-list {
    max-height: 0;
    overflow: hidden;
    transition: max-height 0.3s ease;
}

.dimension-issues-list.expanded {
    max-height: 500px;
    overflow-y: auto;
}

/* Issues */
.no-issues {
    text-align: center;
    padding: 3rem 1rem;
}

.no-issues-icon {
    font-size: 4rem;
    margin-bottom: 1rem;
}

.issues-filters {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
}

.filter-btn {
    padding: 0.5rem 1rem;
    background: var(--bg-tertiary);
    border: 1px solid var(--border);
    border-radius: 0.5rem;
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s ease;
}

.filter-btn:hover, .filter-btn.active {
    background: var(--bg-primary);
    color: var(--text-primary);
    border-color: var(--color-b);
}

.issues-table {
    width: 100%;
    border-collapse: collapse;
}

.issues-table th {
    text-align: left;
    padding: 0.75rem;
    border-bottom: 1px solid var(--border);
    color: var(--text-secondary);
    font-weight: 500;
}

.issues-table td {
    padding: 0.75rem;
    border-bottom: 1px solid var(--border);
}

.issue-row {
    cursor: pointer;
    transition: background 0.2s ease;
}

.issue-row:hover {
    background: var(--bg-tertiary);
}

.severity-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
    font-weight: 500;
}

.severity-badge.severity-critical { background: rgba(239, 68, 68, 0.2); color: var(--color-f); }
.severity-badge.severity-high { background: rgba(245, 158, 11, 0.2); color: var(--color-c); }
.severity-badge.severity-medium { background: rgba(59, 130, 246, 0.2); color: var(--color-b); }
.severity-badge.severity-low { background: rgba(16, 185, 129, 0.2); color: var(--color-a); }
.severity-badge.severity-info { background: rgba(148, 163, 184, 0.2); color: var(--text-secondary); }

/* Issue cards in dimension view */
.issue {
    background: var(--bg-secondary);
    border-radius: 0.5rem;
    margin-bottom: 0.5rem;
    overflow: hidden;
}

.issue-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.75rem;
    cursor: pointer;
}

.issue-sev-icon { font-size: 1rem; }
.issue-title { flex: 1; font-weight: 500; }
.issue-impact {
    padding: 0.125rem 0.5rem;
    background: rgba(239, 68, 68, 0.1);
    color: var(--color-f);
    border-radius: 0.25rem;
    font-size: 0.75rem;
}

.issue-details {
    max-height: 0;
    overflow: hidden;
    transition: max-height 0.3s ease;
    padding: 0 0.75rem;
}

.issue-details.expanded {
    max-height: 300px;
    overflow-y: auto;
    padding-bottom: 0.75rem;
}

.issue-description {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 0.5rem;
}

.issue-location, .issue-suggestion {
    font-size: 0.75rem;
    margin-bottom: 0.5rem;
}

.issue-suggestion code {
    display: block;
    background: var(--bg-primary);
    padding: 0.5rem;
    border-radius: 0.25rem;
    margin-top: 0.25rem;
    white-space: pre-wrap;
}

/* Trend */
.trend-card {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 1.5rem;
    border-radius: 0.75rem;
}

.trend-card.trend-improving { background: rgba(16, 185, 129, 0.1); border: 1px solid var(--color-a); }
.trend-card.trend-stable { background: rgba(59, 130, 246, 0.1); border: 1px solid var(--color-b); }
.trend-card.trend-declining { background: rgba(239, 68, 68, 0.1); border: 1px solid var(--color-f); }

.trend-icon { font-size: 2.5rem; }

.trend-info { flex: 1; }
.trend-title { font-size: 0.875rem; color: var(--text-secondary); margin-bottom: 0.25rem; }
.trend-values { display: flex; align-items: baseline; gap: 0.75rem; }
.trend-current { font-size: 2rem; font-weight: 700; }
.trend-delta { font-size: 1.25rem; font-weight: 600; }
.trend-delta.positive { color: var(--color-a); }
.trend-delta.negative { color: var(--color-f); }

.prediction {
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
}

.prediction-title {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
}

.prediction-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1rem;
}

.prediction-item {
    text-align: center;
}

.prediction-label {
    display: block;
    font-size: 0.75rem;
    color: var(--text-secondary);
}

.prediction-value {
    display: block;
    font-size: 1.25rem;
    font-weight: 600;
}

/* Metrics */
.metrics-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 1rem;
}

.metrics-dimension h3 {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
}

.metrics-table {
    width: 100%;
}

.metrics-table td {
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border);
    font-size: 0.875rem;
}

.metrics-table td:last-child {
    text-align: right;
    font-weight: 600;
}

/* Recommendations */
.rec-subtitle {
    color: var(--text-secondary);
    font-size: 0.875rem;
    margin-bottom: 1rem;
}

.recommendations-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: 1rem;
}

.recommendation-card {
    background: var(--bg-tertiary);
    border-radius: 0.75rem;
    padding: 1rem;
}

.rec-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.5rem;
    margin-bottom: 0.75rem;
}

.rec-header h3 {
    font-size: 1rem;
    font-weight: 600;
}

.rec-badges {
    display: flex;
    gap: 0.25rem;
}

.rec-badge {
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.625rem;
    font-weight: 500;
    text-transform: uppercase;
}

.rec-badge.effort-trivial { background: rgba(16, 185, 129, 0.2); color: var(--color-a); }
.rec-badge.effort-low { background: rgba(59, 130, 246, 0.2); color: var(--color-b); }
.rec-badge.effort-medium { background: rgba(245, 158, 11, 0.2); color: var(--color-c); }
.rec-badge.impact-medium { background: rgba(59, 130, 246, 0.2); color: var(--color-b); }
.rec-badge.impact-high { background: rgba(16, 185, 129, 0.2); color: var(--color-a); }

.rec-description {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-bottom: 0.5rem;
}

.rec-dimension {
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-transform: uppercase;
}

/* Footer */
.footer {
    text-align: center;
    padding: 2rem 0;
    color: var(--text-secondary);
    font-size: 0.875rem;
}

.footer strong {
    color: var(--text-primary);
}

/* Responsive */
@media (max-width: 768px) {
    .container {
        padding: 1rem;
    }

    .header {
        flex-direction: column;
        text-align: center;
    }

    .project-info {
        justify-content: center;
    }

    .score-stats {
        grid-template-columns: repeat(2, 1fr);
    }

    .dimensions-grid {
        grid-template-columns: 1fr;
    }

    .prediction-grid {
        grid-template-columns: 1fr;
    }
}
"#;

// Embedded JavaScript
const JAVASCRIPT: &str = r#"
function toggleDimensionIssues(id) {
    const btn = document.querySelector(`[onclick="toggleDimensionIssues('${id}')"]`);
    const list = document.getElementById(`issues-${id}`);

    if (list) {
        list.classList.toggle('expanded');
        btn.classList.toggle('active');
    }
}

function toggleIssueDetails(id) {
    const details = document.getElementById(`issue-${id}`);

    if (details) {
        details.classList.toggle('expanded');
    }
}

function openIssueModal(id) {
    // Modal functionality - expandable for future use
    console.log('Open issue:', id);
}

// Issue filtering
document.addEventListener('DOMContentLoaded', function() {
    const filterBtns = document.querySelectorAll('.filter-btn');

    filterBtns.forEach(btn => {
        btn.addEventListener('click', function() {
            // Remove active class from all buttons
            filterBtns.forEach(b => b.classList.remove('active'));
            // Add active class to clicked button
            this.classList.add('active');

            const filter = this.dataset.filter;
            const rows = document.querySelectorAll('.issue-row');

            rows.forEach(row => {
                if (filter === 'all' || row.classList.contains(`severity-${filter}`)) {
                    row.style.display = '';
                } else {
                    row.style.display = 'none';
                }
            });
        });
    });
});
"#;
