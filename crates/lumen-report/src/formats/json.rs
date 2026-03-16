//! JSON report generation

use lumen_core::ProjectInfo;
use lumen_score::{
    DimensionScores, Grade, ProjectScore, ScoreIssue, TrendAnalysis,
};
use serde_json::json;

/// Generate comprehensive JSON report
pub fn generate(info: &ProjectInfo, score: &ProjectScore) -> String {
    let report = json!({
        "version": "1.0",
        "lumen_version": score.metadata.scorer_version,
        "generated_at": score.timestamp,
        "project": {
            "name": info.name,
            "root": info.root.to_string_lossy(),
            "framework": framework_to_json(info.framework.clone()),
            "language": language_to_json(info.language.clone()),
            "test_runner": test_runner_to_json(info.test_runner.clone()),
            "package_manager": info.package_manager,
            "dependencies": info.dependencies.clone(),
            "dev_dependencies": info.dev_dependencies.clone(),
        },
        "overall_score": {
            "score": score.overall,
            "grade": grade_to_json(score.grade),
            "gpa": score.grade.gpa_value(),
            "is_passing": score.grade.is_passing(),
            "is_good": score.grade.is_good(),
            "is_excellent": score.grade.is_excellent(),
            "color": score.grade.css_color(),
            "label": score.grade.label(),
        },
        "dimensions": dimensions_to_json(&score.dimensions),
        "issues": collect_issues(&score.dimensions),
        "improvements": collect_improvements(&score.dimensions),
        "trend": score.trend.as_ref().map(trend_to_json),
        "metadata": metadata_to_json(&score.metadata),
        "metrics": extract_all_metrics(&score.dimensions),
        "summary": generate_summary(info, score),
    });

    serde_json::to_string_pretty(&report).unwrap_or_default()
}

fn framework_to_json(framework: lumen_core::Framework) -> serde_json::Value {
    json!({
        "id": serde_json::to_value(&framework).unwrap_or(json!("unknown")),
        "name": framework.display_name(),
        "is_rust": framework.is_rust(),
        "is_nodejs": framework.is_nodejs(),
    })
}

fn language_to_json(language: lumen_core::Language) -> serde_json::Value {
    json!({
        "id": serde_json::to_value(&language).unwrap_or(json!("unknown")),
        "name": language.display_name(),
    })
}

fn test_runner_to_json(test_runner: lumen_core::TestRunner) -> serde_json::Value {
    json!({
        "id": serde_json::to_value(&test_runner).unwrap_or(json!("unknown")),
        "name": test_runner.display_name(),
    })
}

fn grade_to_json(grade: Grade) -> serde_json::Value {
    json!({
        "letter": grade.as_letter(),
        "label": grade.label(),
        "gpa": grade.gpa_value(),
        "color": grade.css_color(),
        "is_passing": grade.is_passing(),
        "is_good": grade.is_good(),
        "is_excellent": grade.is_excellent(),
        "score_range": {
            "min": grade.score_range().0,
            "max": grade.score_range().1,
        },
    })
}

fn dimensions_to_json(dimensions: &DimensionScores) -> serde_json::Value {
    json!({
        "coverage": dimension_to_json(&dimensions.coverage),
        "quality": dimension_to_json(&dimensions.quality),
        "performance": dimension_to_json(&dimensions.performance),
        "security": dimension_to_json(&dimensions.security),
        "seo": dimension_to_json(&dimensions.seo),
        "docs": dimension_to_json(&dimensions.docs),
        "uiux": dimension_to_json(&dimensions.uiux),
        "summary": {
            "min": dimensions.min_score(),
            "max": dimensions.max_score(),
            "weighted_sum": dimensions.weighted_sum(),
        },
    })
}

fn dimension_to_json(dim: &lumen_score::DimensionScore) -> serde_json::Value {
    json!({
        "name": dim.name,
        "score": dim.score,
        "weight": dim.weight,
        "weighted": dim.weighted,
        "grade": grade_to_json(dim.grade),
        "issues": dim.issues.iter().map(issue_to_json).collect::<Vec<_>>(),
        "improvements": dim.improvements.iter().map(improvement_to_json).collect::<Vec<_>>(),
        "metrics": dim.metrics,
        "issue_count": dim.issues.len(),
        "issue_penalty": dim.calculate_issue_penalty(),
    })
}

fn issue_to_json(issue: &ScoreIssue) -> serde_json::Value {
    json!({
        "id": issue.id,
        "severity": {
            "level": issue.severity as i32,
            "name": issue.severity.as_str(),
            "color": issue.severity.color(),
        },
        "category": issue.category,
        "title": issue.title,
        "description": issue.description,
        "location": {
            "file": issue.file,
            "line": issue.line,
            "column": issue.column,
            "display": format_location(issue.file.as_deref(), issue.line, issue.column),
        },
        "impact": issue.impact,
        "suggestion": issue.suggestion,
        "fix_available": issue.suggestion.is_some(),
    })
}

fn improvement_to_json(imp: &lumen_score::Improvement) -> serde_json::Value {
    json!({
        "id": imp.id,
        "title": imp.title,
        "description": imp.description,
        "effort": {
            "level": serde_json::to_value(imp.effort).unwrap_or(json!("unknown")),
            "name": imp.effort.as_str(),
            "hours": imp.effort.hours(),
        },
        "impact": {
            "level": serde_json::to_value(imp.impact).unwrap_or(json!("unknown")),
            "name": imp.impact.as_str(),
            "points": imp.impact.points(),
        },
        "dimension": imp.dimension,
        "roi": calculate_roi(imp.effort, imp.impact),
    })
}

fn trend_to_json(trend: &TrendAnalysis) -> serde_json::Value {
    json!({
        "current": trend.current,
        "previous": trend.previous,
        "direction": format!("{:?}", trend.direction),
        "delta": {
            "overall": trend.delta.overall,
            "coverage": trend.delta.coverage,
            "quality": trend.delta.quality,
            "performance": trend.delta.performance,
            "security": trend.delta.security,
            "seo": trend.delta.seo,
            "docs": trend.delta.docs,
            "uiux": trend.delta.uiux,
            "is_improving": trend.delta.is_improving(),
            "is_declining": trend.delta.is_declining(),
            "is_significant": trend.delta.is_significant(),
            "best_improvement": trend.delta.best_improvement(),
            "worst_decline": trend.delta.worst_decline(),
        },
        "moving_average": {
            "ma7": trend.moving_avg.ma7,
            "ma30": trend.moving_avg.ma30,
            "ma90": trend.moving_avg.ma90,
            "is_accelerating": trend.moving_avg.is_accelerating(),
            "is_decelerating": trend.moving_avg.is_decelerating(),
        },
        "prediction": trend.prediction.as_ref().map(|p| json!({
            "score_7d": p.score_7d,
            "grade_7d": grade_to_json(p.grade_7d),
            "score_30d": p.score_30d,
            "grade_30d": grade_to_json(p.grade_30d),
            "confidence": p.confidence,
            "velocity": p.velocity,
            "is_optimistic": p.is_optimistic(),
            "is_pessimistic": p.is_pessimistic(),
        })),
        "summary": trend.summary(),
        "has_critical_decline": trend.has_critical_decline(),
    })
}

fn metadata_to_json(metadata: &lumen_score::ScoreMetadata) -> serde_json::Value {
    json!({
        "scorer_version": metadata.scorer_version,
        "scan_duration_ms": metadata.scan_duration_ms,
        "scan_duration_seconds": metadata.scan_duration_ms as f64 / 1000.0,
        "files_scanned": metadata.files_scanned,
        "lines_of_code": metadata.lines_of_code,
        "language_breakdown": metadata.language_breakdown,
        "profile": metadata.profile,
        "average_file_size": if metadata.files_scanned > 0 {
            Some(metadata.lines_of_code / metadata.files_scanned)
        } else {
            None
        },
    })
}

fn collect_issues(dimensions: &DimensionScores) -> serde_json::Value {
    let mut all_issues = Vec::new();

    for (dim_name, dim) in dimensions.all() {
        for issue in &dim.issues {
            let mut issue_json = issue_to_json(issue);
            issue_json["dimension"] = json!(dim_name);
            all_issues.push(issue_json);
        }
    }

    // Sort by severity
    all_issues.sort_by(|a, b| {
        let sev_a: i32 = a["severity"]["level"].as_i64().unwrap_or(0) as i32;
        let sev_b: i32 = b["severity"]["level"].as_i64().unwrap_or(0) as i32;
        sev_b.cmp(&sev_a)
    });

    json!({
        "total": all_issues.len(),
        "by_severity": {
            "critical": all_issues.iter().filter(|i| i["severity"]["name"] == "critical").count(),
            "high": all_issues.iter().filter(|i| i["severity"]["name"] == "high").count(),
            "medium": all_issues.iter().filter(|i| i["severity"]["name"] == "medium").count(),
            "low": all_issues.iter().filter(|i| i["severity"]["name"] == "low").count(),
            "info": all_issues.iter().filter(|i| i["severity"]["name"] == "info").count(),
        },
        "items": all_issues,
    })
}

fn collect_improvements(dimensions: &DimensionScores) -> serde_json::Value {
    let mut all_improvements = Vec::new();

    for (dim_name, dim) in dimensions.all() {
        for imp in &dim.improvements {
            let mut imp_json = improvement_to_json(imp);
            imp_json["dimension"] = json!(dim_name);
            all_improvements.push(imp_json);
        }
    }

    // Sort by ROI (impact/effort ratio)
    all_improvements.sort_by(|a, b| {
        let roi_a = a["roi"].as_f64().unwrap_or(0.0);
        let roi_b = b["roi"].as_f64().unwrap_or(0.0);
        roi_b.partial_cmp(&roi_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    json!({
        "total": all_improvements.len(),
        "quick_wins": all_improvements.iter()
            .filter(|i| i["effort"]["name"] == "trivial" || i["effort"]["name"] == "low")
            .filter(|i| i["impact"]["name"] == "medium" || i["impact"]["name"] == "high")
            .count(),
        "items": all_improvements,
    })
}

fn extract_all_metrics(dimensions: &DimensionScores) -> serde_json::Value {
    let mut all_metrics = serde_json::Map::new();

    for (dim_name, dim) in dimensions.all() {
        let dim_metrics: serde_json::Map<String, serde_json::Value> = dim
            .metrics
            .iter()
            .map(|(k, v)| {
                let json_value = match v {
                    lumen_score::MetricValue::Float(val) => json!({ "type": "float", "value": val }),
                    lumen_score::MetricValue::Integer(val) => json!({ "type": "integer", "value": val }),
                    lumen_score::MetricValue::Percentage(val) => json!({ "type": "percentage", "value": val }),
                    lumen_score::MetricValue::Duration(val) => json!({ "type": "duration_ms", "value": val }),
                    lumen_score::MetricValue::Count(val) => json!({ "type": "count", "value": val }),
                    lumen_score::MetricValue::Boolean(val) => json!({ "type": "boolean", "value": val }),
                    lumen_score::MetricValue::String(val) => json!({ "type": "string", "value": val }),
                };
                (k.clone(), json_value)
            })
            .collect();

        all_metrics.insert(dim_name.to_string(), json!(dim_metrics));
    }

    json!(all_metrics)
}

fn generate_summary(info: &ProjectInfo, score: &ProjectScore) -> serde_json::Value {
    let issues = collect_issues(&score.dimensions);
    let improvements = collect_improvements(&score.dimensions);

    // Determine overall status
    let status = if score.overall >= 90.0 {
        "excellent"
    } else if score.overall >= 80.0 {
        "good"
    } else if score.overall >= 70.0 {
        "fair"
    } else if score.overall >= 60.0 {
        "poor"
    } else {
        "critical"
    };

    json!({
        "status": status,
        "action_required": score.overall < 80.0,
        "critical_issues": issues["by_severity"]["critical"],
        "high_issues": issues["by_severity"]["high"],
        "total_issues": issues["total"],
        "quick_wins_available": improvements["quick_wins"],
        "framework": info.framework.display_name(),
        "language": info.language.display_name(),
    })
}

fn format_location(file: Option<&str>, line: Option<usize>, column: Option<usize>) -> String {
    let mut loc = String::new();
    if let Some(f) = file {
        loc.push_str(f);
        if let Some(l) = line {
            loc.push_str(&format!(":{}", l));
            if let Some(c) = column {
                loc.push_str(&format!(":{}", c));
            }
        }
    }
    loc
}

fn calculate_roi(
    effort: lumen_score::Effort,
    impact: lumen_score::Impact,
) -> f64 {
    let effort_hours = effort.hours();
    let (impact_min, impact_max) = impact.points();
    let impact_avg = (impact_min + impact_max) / 2.0;

    if effort_hours > 0.0 {
        impact_avg / effort_hours
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumen_score::{DimensionScore, DimensionScores, Grade};
    use std::collections::HashMap;

    #[test]
    fn test_json_output_valid() {
        let info = mock_project_info();
        let score = mock_project_score();

        let json = generate(&info, &score);

        // Verify valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
    }

    #[test]
    fn test_json_structure() {
        let info = mock_project_info();
        let score = mock_project_score();

        let json = generate(&info, &score);
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(value["overall_score"].is_object());
        assert!(value["dimensions"].is_object());
        assert!(value["issues"].is_object());
        assert!(value["metadata"].is_object());
        assert!(value["summary"].is_object());
    }

    fn mock_project_info() -> ProjectInfo {
        use lumen_core::{Framework, Language, ProjectInfo, TestRunner};
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
        use lumen_score::{DimensionScore, ScoreMetadata, ScoreIssue, IssueSeverity};

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
