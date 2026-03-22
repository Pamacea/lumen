// Oalacea Lumen
//
// Oalacea Lumen CLI - AI-powered code analysis and test generation toolkit
//
// This crate contains:
// - CLI interface
// - Test generation
// - Auto-fix capabilities
// - Multi-format report generation

pub mod cli;
pub mod testgen;
pub mod fix;
pub mod report;
pub mod watch;

// Re-exports for convenience
pub use cli::{Cli, CliConfig};
pub use testgen::{TestGenerator, TestConfig};
pub use fix::{AutoFixer, FixResult, FixStrategy};
pub use report::{ReportGenerator, ReportFormat, ReportOutput};

// Re-export scoring as lumen_score for backward compatibility
pub use oalacea_lumen_core::scoring as lumen_score;
