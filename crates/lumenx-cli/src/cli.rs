//! # Lumen CLI module
//!
//! Command-line interface definitions for Lumen.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Lumen - AI-powered code analysis and test generation toolkit
#[derive(Parser, Debug)]
#[command(name = "lumen")]
#[command(author = "Oalacea <contact@oalacea.com>")]
#[command(version = "0.6.0")]
#[command(about = "AI-powered code analysis and test generation toolkit", long_about = None)]
#[command(after_help = "Examples:
  lumen scan
  lumen scan --format json --output ./reports
  lumen init
  lumen detect --json
  lumen fix --dry-run --interactive
  lumen generate-tests --framework vitest")]
pub struct LumenCli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Full analysis with AI-ready fixes (default command)
    Scan {
        /// Path to project directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Output directory for reports
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Report format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Only scan specific dimensions (comma-separated)
        #[arg(long)]
        dimensions: Option<String>,

        /// Minimum severity level to report
        #[arg(long, value_enum)]
        severity: Option<SeverityLevel>,
    },

    /// Initialize Lumen configuration in the project
    Init {
        /// Path to project directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Use default configuration without prompts
        #[arg(long)]
        defaults: bool,
    },

    /// Detect framework, language, and tools
    Detect {
        /// Path to project directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Show detailed detection information
        #[arg(long)]
        detailed: bool,
    },

    /// Run specific analyzers on the codebase
    Analyze {
        /// Path to project directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Specific analyzer to run
        #[arg(long, value_enum)]
        analyzer: Option<AnalyzerType>,

        /// Output file for results
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Minimum severity level to report
        #[arg(long, value_enum)]
        severity: Option<SeverityLevel>,
    },

    /// Calculate and display quality scores
    Score {
        /// Path to project directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Compare with previous score (if available)
        #[arg(long)]
        compare: bool,

        /// Show detailed breakdown by dimension
        #[arg(long)]
        detailed: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate test templates for untested code
    GenerateTests {
        /// Path to project directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Output directory for tests
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Test framework to use
        #[arg(long, value_enum)]
        framework: Option<TestFramework>,

        /// Type of tests to generate
        #[arg(long, value_enum)]
        test_type: Option<TestType>,

        /// Show what would be generated without writing
        #[arg(long)]
        dry_run: bool,

        /// Specific files or patterns to generate tests for
        #[arg(long)]
        files: Option<String>,
    },

    /// Apply automatic fixes to identified issues
    Fix {
        /// Path to project directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Dry run (show what would be fixed without making changes)
        #[arg(long)]
        dry_run: bool,

        /// Interactive mode (prompt for each fix)
        #[arg(long, short)]
        interactive: bool,

        /// Automatically apply all fixes without prompting
        #[arg(long)]
        yes: bool,

        /// Only fix issues with specific severity or higher
        #[arg(long, value_enum)]
        min_severity: Option<SeverityLevel>,

        /// Specific categories to fix (comma-separated)
        #[arg(long)]
        categories: Option<String>,
    },

    /// Generate comprehensive reports
    Report {
        /// Path to project directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Report format
        #[arg(short, long, value_enum)]
        format: Option<OutputFormat>,

        /// Output directory for reports
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include historical trend data
        #[arg(long)]
        trend: bool,

        /// Generate all available formats
        #[arg(long)]
        all: bool,
    },

    /// Watch for changes and re-run analysis automatically
    Watch {
        /// Path to project directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Paths to include (comma-separated)
        #[arg(long)]
        include: Option<String>,

        /// Paths to exclude (comma-separated)
        #[arg(long)]
        exclude: Option<String>,

        /// File patterns to include (comma-separated, e.g., "*.ts,*.rs")
        #[arg(long)]
        include_patterns: Option<String>,

        /// File patterns to exclude (comma-separated)
        #[arg(long)]
        exclude_patterns: Option<String>,

        /// Debounce delay in milliseconds
        #[arg(long, default_value = "300")]
        debounce_ms: u64,

        /// Don't run analysis on startup
        #[arg(long)]
        no_startup: bool,
    },
}

/// Report output format
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum OutputFormat {
    /// Markdown format
    Markdown,
    /// JSON format
    Json,
    /// HTML format
    Html,
    /// JUnit XML format
    JUnit,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Markdown => write!(f, "markdown"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Html => write!(f, "html"),
            OutputFormat::JUnit => write!(f, "junit"),
        }
    }
}

impl OutputFormat {
    pub fn extension(&self) -> &str {
        match self {
            OutputFormat::Markdown => "md",
            OutputFormat::Json => "json",
            OutputFormat::Html => "html",
            OutputFormat::JUnit => "xml",
        }
    }
}

/// Analyzer types
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum AnalyzerType {
    /// Static code analysis
    Static,
    /// Security vulnerability scanning
    Security,
    /// Dependency analysis
    Dependency,
    /// Performance analysis
    Performance,
    /// SEO analysis
    Seo,
    /// UI/UX analysis
    Uiux,
    /// Documentation analysis
    Docs,
    /// Run all analyzers
    All,
}

/// Severity levels
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityLevel {
    /// Info only
    Info,
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
    Critical,
}

impl SeverityLevel {
    pub fn from_lumen_score(severity: lumenx_score::IssueSeverity) -> Self {
        match severity {
            lumenx_score::IssueSeverity::Info => SeverityLevel::Info,
            lumenx_score::IssueSeverity::Low => SeverityLevel::Low,
            lumenx_score::IssueSeverity::Medium => SeverityLevel::Medium,
            lumenx_score::IssueSeverity::High => SeverityLevel::High,
            lumenx_score::IssueSeverity::Critical => SeverityLevel::Critical,
        }
    }

    pub fn to_lumen_score(&self) -> lumenx_score::IssueSeverity {
        match self {
            SeverityLevel::Info => lumenx_score::IssueSeverity::Info,
            SeverityLevel::Low => lumenx_score::IssueSeverity::Low,
            SeverityLevel::Medium => lumenx_score::IssueSeverity::Medium,
            SeverityLevel::High => lumenx_score::IssueSeverity::High,
            SeverityLevel::Critical => lumenx_score::IssueSeverity::Critical,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SeverityLevel::Info => "info",
            SeverityLevel::Low => "low",
            SeverityLevel::Medium => "medium",
            SeverityLevel::High => "high",
            SeverityLevel::Critical => "critical",
        }
    }
}

/// Test frameworks
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum TestFramework {
    /// Vitest (recommended for Vite projects)
    Vitest,
    /// Jest
    Jest,
    /// Cargo test (Rust)
    Cargo,
    /// Cargo nextest (Rust)
    Nextest,
    /// Pytest (Python)
    Pytest,
    /// Detect automatically
    Auto,
}

/// Test types
#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum TestType {
    /// Unit tests only
    Unit,
    /// Integration tests only
    Integration,
    /// E2E tests only
    E2E,
    /// All test types
    All,
}
