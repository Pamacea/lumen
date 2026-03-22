//! # CLI Module
//!
//! Command-line interface for Oalacea Lumen.

use anyhow::Result;
use clap::Parser;

/// CLI entry point
pub struct Cli;

impl Cli {
    pub async fn run() -> Result<()> {
        println!("Oalacea Lumen v{}", env!("CARGO_PKG_VERSION"));
        Ok(())
    }
}

/// CLI configuration
#[derive(Debug, Parser)]
pub struct CliConfig {
    /// Project root directory
    #[arg(short, long, default_value = ".")]
    pub root: String,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Output format
    #[arg(short, long, default_value = "text")]
    pub format: String,
}
