//! # Lumen Analysis
//!
//! Code analysis engine with multiple analyzers and AST parsing.

pub mod analyzers;
pub mod ast;
pub mod parsers;

use lumen_core::{LumenResult, Project};
// Re-export commonly used types from lumen_score
pub use lumen_score::{IssueSeverity, ScoreIssue};

/// Analysis result
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Static analysis findings
    pub static_findings: Vec<ScoreIssue>,
    /// Security findings
    pub security_findings: Vec<ScoreIssue>,
    /// Dependency findings
    pub dependency_findings: Vec<ScoreIssue>,
    /// Performance findings
    pub performance_findings: Vec<ScoreIssue>,
    /// SEO findings
    pub seo_findings: Vec<ScoreIssue>,
    /// UI/UX findings
    pub uiux_findings: Vec<ScoreIssue>,
    /// Documentation findings
    pub docs_findings: Vec<ScoreIssue>,
}

impl AnalysisResult {
    pub fn total_critical(&self) -> usize {
        self.static_findings.iter()
            .chain(self.security_findings.iter())
            .chain(self.dependency_findings.iter())
            .chain(self.performance_findings.iter())
            .chain(self.seo_findings.iter())
            .chain(self.uiux_findings.iter())
            .chain(self.docs_findings.iter())
            .filter(|i| i.severity == IssueSeverity::Critical)
            .count()
    }

    pub fn total_high(&self) -> usize {
        self.all_findings()
            .filter(|i| i.severity == IssueSeverity::High)
            .count()
    }

    pub fn total_medium(&self) -> usize {
        self.all_findings()
            .filter(|i| i.severity == IssueSeverity::Medium)
            .count()
    }

    pub fn total_low(&self) -> usize {
        self.all_findings()
            .filter(|i| i.severity == IssueSeverity::Low)
            .count()
    }

    pub fn total_fixable(&self) -> usize {
        self.all_findings().filter(|i| i.title.contains("fix") || i.title.contains("add")).count()
    }

    pub fn all_findings(&self) -> impl Iterator<Item = &ScoreIssue> {
        self.static_findings.iter()
            .chain(self.security_findings.iter())
            .chain(self.dependency_findings.iter())
            .chain(self.performance_findings.iter())
            .chain(self.seo_findings.iter())
            .chain(self.uiux_findings.iter())
            .chain(self.docs_findings.iter())
    }
}

/// Main analyzer
pub struct Analyzer {
    project: Project,
}

impl Analyzer {
    pub fn new(project: Project) -> Self {
        Self { project }
    }

    pub fn analyze(&self) -> LumenResult<AnalysisResult> {
        Ok(AnalysisResult {
            static_findings: analyzers::static_analyzer::analyze(&self.project),
            security_findings: analyzers::security_analyzer::analyze(&self.project),
            dependency_findings: analyzers::dependency_analyzer::analyze(&self.project),
            performance_findings: analyzers::performance_analyzer::analyze(&self.project),
            seo_findings: analyzers::seo_analyzer::analyze(&self.project),
            uiux_findings: analyzers::uiux_analyzer::analyze(&self.project),
            docs_findings: analyzers::docs_analyzer::analyze(&self.project),
        })
    }
}
