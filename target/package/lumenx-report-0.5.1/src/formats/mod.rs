//! Report formats

pub mod markdown;
pub mod json;
pub mod html;
pub mod junit;

// Re-export generate functions
pub use markdown::{generate as generate_markdown, generate_fixes as generate_fixes_markdown};

use lumenx_core::ProjectInfo;
use lumenx_score::ProjectScore;

pub fn generate(info: &ProjectInfo, score: &ProjectScore) -> String {
    markdown::generate(info, score)
}

pub fn generate_fixes(info: &ProjectInfo, score: &ProjectScore) -> String {
    markdown::generate_fixes(info, score)
}
