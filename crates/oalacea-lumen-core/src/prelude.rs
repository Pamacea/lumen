//! Prelude module for common imports

// Core types
pub use crate::core::{
    Config, LumenError as Error, LumenResult as Result, Project, ProjectInfo,
};

pub use crate::core::config::*;
pub use crate::core::project::*;

// Detection
pub use crate::detect::{
    FrameworkDetector, DetectionResult,
};

pub use crate::detect::detectors::*;

// Scoring
pub use crate::scoring::{
    ScoreCalculator,
    ProjectScore, DimensionScores, DimensionScore,
    ScoreIssue, IssueSeverity, Improvement,
    Effort, Impact, ScoreMetadata, Grade, GradeSystem,
};

pub use crate::scoring::calculator::*;
pub use crate::scoring::metrics::*;
pub use crate::scoring::trend::*;

// Re-export commonly used external crates
pub use serde::{Deserialize, Serialize};
pub use std::path::{Path, PathBuf};
pub use std::collections::HashMap;
