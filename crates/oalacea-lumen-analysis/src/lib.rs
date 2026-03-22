// Oalacea Lumen Analysis
//
// Analysis engine for Oalacea Lumen:
// - Static code analysis (AST-based)
// - Git diff analysis
// - Score history tracking with SQLite

pub mod analyze;
pub mod diff;
pub mod history;
pub mod ast;
pub mod parsers;

// Re-exports for convenience
pub use analyze::{
    AnalysisResult,
    AnalyzerConfig,
};

pub use diff::{
    DiffAnalyzer,
    DiffResult,
    ChangedFile,
};

pub use history::{
    ScoreHistory,
    HistoryEntry,
    TrendAnalysis,
};
