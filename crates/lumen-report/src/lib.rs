//! # Lumen Report Generation
//!
//! Generate reports in multiple formats.

pub mod formats;

use lumen_core::{LumenResult, ProjectInfo};
use lumen_score::ProjectScore;
use std::path::PathBuf;

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    Markdown,
    Json,
    Html,
    JUnit,
}

/// Report generator
pub struct ReportGenerator {
    project_info: ProjectInfo,
    score: ProjectScore,
    output_dir: PathBuf,
}

impl ReportGenerator {
    pub fn new(project_info: ProjectInfo, score: ProjectScore, output_dir: PathBuf) -> Self {
        Self {
            project_info,
            score,
            output_dir,
        }
    }

    pub fn generate(&self, format: ReportFormat) -> LumenResult<Vec<ReportOutput>> {
        // Create output directory
        std::fs::create_dir_all(&self.output_dir)?;

        match format {
            ReportFormat::Markdown => {
                let report = formats::markdown::generate(&self.project_info, &self.score);
                let path = self.output_dir.join("report.md");
                std::fs::write(&path, &report)?;
                Ok(vec![ReportOutput {
                    format: ReportFormat::Markdown,
                    path,
                    content: report,
                }])
            }
            ReportFormat::Json => {
                let report = formats::json::generate(&self.project_info, &self.score);
                let path = self.output_dir.join("report.json");
                std::fs::write(&path, &report)?;
                Ok(vec![ReportOutput {
                    format: ReportFormat::Json,
                    path,
                    content: report,
                }])
            }
            ReportFormat::Html => {
                let report = formats::html::generate(&self.project_info, &self.score);
                let path = self.output_dir.join("report.html");
                std::fs::write(&path, &report)?;
                Ok(vec![ReportOutput {
                    format: ReportFormat::Html,
                    path,
                    content: report,
                }])
            }
            ReportFormat::JUnit => {
                let report = formats::junit::generate(&self.project_info, &self.score);
                let path = self.output_dir.join("junit.xml");
                std::fs::write(&path, &report)?;
                Ok(vec![ReportOutput {
                    format: ReportFormat::JUnit,
                    path,
                    content: report,
                }])
            }
        }
    }

    /// Generate AI-friendly fixes report
    pub fn generate_fixes_report(&self) -> LumenResult<ReportOutput> {
        std::fs::create_dir_all(&self.output_dir)?;

        let report = formats::markdown::generate_fixes(&self.project_info, &self.score);
        let path = self.output_dir.join("fixes.md");
        std::fs::write(&path, &report)?;

        Ok(ReportOutput {
            format: ReportFormat::Markdown,
            path,
            content: report.clone(),
        })
    }

    /// Generate all reports
    pub fn generate_all(&self) -> LumenResult<Vec<ReportOutput>> {
        let mut outputs = Vec::new();
        
        for format in [ReportFormat::Markdown, ReportFormat::Json] {
            outputs.extend(self.generate(format)?);
        }
        
        // Also generate fixes.md
        outputs.push(self.generate_fixes_report()?);
        
        Ok(outputs)
    }
}

/// Generated report output
#[derive(Debug, Clone)]
pub struct ReportOutput {
    pub format: ReportFormat,
    pub path: PathBuf,
    pub content: String,
}
