// Oalacea Lumen Core
//
// Core foundations for Oalacea Lumen toolkit:
// - Core types, errors, and configuration
// - Framework and language detection
// - 7-dimension quality scoring system

pub mod core;
pub mod detect;
pub mod scoring;
pub mod prelude;

// Re-exports for convenience - Core types
pub use core::{
    Config,
    LumenError,
    LumenResult,
    Project,
    ProjectInfo,
    Framework,
    Language,
    TestRunner,
    PackageJson,
};

pub use detect::{
    FrameworkDetector,
};

pub use scoring::{
    ScoreCalculator,
    ProjectScore,
    DimensionScores,
    DimensionScore,
    ScoreIssue,
    IssueSeverity,
    Grade,
};
