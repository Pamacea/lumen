//! Report formats

pub mod markdown;
pub mod json;
pub mod html;
pub mod junit;
pub mod fixes;

// Re-export generate functions
pub use markdown::{generate as generate_markdown, generate_fixes as generate_fixes_markdown};
pub use fixes::{generate_fixes_json, generate_fixes_summary};

use lumenx_core::ProjectInfo;
use lumenx_score::{ProjectScore, DimensionScores};

pub fn generate(info: &ProjectInfo, score: &ProjectScore) -> String {
    markdown::generate(info, score)
}

pub fn generate_fixes(info: &ProjectInfo, score: &ProjectScore) -> String {
    markdown::generate_fixes(info, score)
}

/// Generate AI-ready fixes JSON
pub fn generate_fixes_for_ai(info: &ProjectInfo, dimensions: &DimensionScores) -> String {
    fixes::generate_fixes_json(info, dimensions)
}
