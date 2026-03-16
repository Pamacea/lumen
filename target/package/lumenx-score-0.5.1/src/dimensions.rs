//! Dimension-specific scoring modules

pub mod coverage;
pub mod quality;
pub mod performance;
pub mod security;
pub mod seo;
pub mod docs;
pub mod uiux;

pub use coverage::CoverageScorer;
pub use quality::QualityScorer;
pub use performance::PerformanceScorer;
pub use security::SecurityScorer;
pub use seo::SeoScorer;
pub use docs::DocsScorer;
pub use uiux::UiuxScorer;
