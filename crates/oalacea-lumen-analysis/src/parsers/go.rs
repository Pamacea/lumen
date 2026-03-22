//! Go analyzer

pub struct GoAnalyzer;

impl GoAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, _code: &str) -> AnalysisResult {
        AnalysisResult
    }
}

pub struct AnalysisResult;