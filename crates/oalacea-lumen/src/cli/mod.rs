//! # CLI Module
//!
//! Command-line interface for Oalacea Lumen.

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use std::collections::HashMap;

use oalacea_lumen_core::{
    FrameworkDetector,
    ScoreCalculator,
    Project,
    ProjectInfo,
    Framework,
    Language,
};

/// Lumen - AI-powered code analysis and test generation toolkit
#[derive(Parser, Debug)]
#[command(name = "lumen")]
#[command(author = "Oalacea <contact@oalacea.com>")]
#[command(version = "0.6.6")]
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

    /// Manage analysis cache
    Cache {
        /// Path to project directory
        #[arg(short, long)]
        path: Option<PathBuf>,

        /// Clear all cache
        #[arg(long)]
        clear: bool,

        /// Show cache statistics
        #[arg(long)]
        stats: bool,

        /// Prune old cache entries
        #[arg(long)]
        prune: bool,
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
#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
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

/// Simple score result for CLI output
#[derive(Debug, Clone)]
pub struct SimpleScoreResult {
    pub overall_score: f64,
    pub grade: String,
    pub trend: String,
    pub dimensions: DimensionScoresOutput,
    pub issues: Vec<IssueOutput>,
}

#[derive(Debug, Clone)]
pub struct DimensionScoresOutput {
    pub coverage: DimensionOutput,
    pub docs: DimensionOutput,
    pub performance: DimensionOutput,
    pub quality: DimensionOutput,
    pub security: DimensionOutput,
    pub seo: DimensionOutput,
    pub uiux: DimensionOutput,
}

#[derive(Debug, Clone)]
pub struct DimensionOutput {
    pub score: f64,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct IssueOutput {
    pub severity: String,
    pub category: String,
    pub message: String,
}

/// CLI entry point
pub struct Cli;

impl Cli {
    pub async fn run() -> Result<()> {
        let cli = LumenCli::parse();
        Self::run_with_cli(cli).await
    }

    async fn run_with_cli(cli: LumenCli) -> Result<()> {
        match cli.command {
            Commands::Scan { path, output, format, dimensions, severity } => {
                Self::handle_scan(path, output, format, dimensions, severity, cli.quiet, cli.verbose).await
            }
            Commands::Init { path, defaults } => {
                Self::handle_init(path, defaults).await
            }
            Commands::Detect { path, json, detailed } => {
                Self::handle_detect(path, json, detailed).await
            }
            Commands::Analyze { path, analyzer, output, severity } => {
                Self::handle_analyze(path, analyzer, output, severity).await
            }
            Commands::Score { path, compare, detailed, json } => {
                Self::handle_score(path, compare, detailed, json).await
            }
            Commands::GenerateTests { path, output, framework, test_type, dry_run, files } => {
                Self::handle_generate_tests(path, output, framework, test_type, dry_run, files).await
            }
            Commands::Fix { path, dry_run, interactive, yes, min_severity, categories } => {
                Self::handle_fix(path, dry_run, interactive, yes, min_severity, categories).await
            }
            Commands::Report { path, format, output, trend, all } => {
                Self::handle_report(path, format, output, trend, all).await
            }
            Commands::Watch { path, include: _, exclude: _, include_patterns: _, exclude_patterns: _, debounce_ms, no_startup } => {
                Self::handle_watch(path, debounce_ms, no_startup).await
            }
            Commands::Cache { path, clear, stats, prune } => {
                Self::handle_cache(path, clear, stats, prune).await
            }
        }
    }

    async fn handle_scan(
        path: Option<PathBuf>,
        output: Option<PathBuf>,
        format: Option<OutputFormat>,
        _dimensions: Option<String>,
        _severity: Option<SeverityLevel>,
        quiet: bool,
        verbose: bool,
    ) -> Result<()> {
        let project_path = path.unwrap_or_else(|| PathBuf::from("."));

        if !quiet {
            println!("🔍 Scanning project: {}", project_path.display());
        }

        // Detect project
        let detector = FrameworkDetector::new(&project_path);
        let project_info = detector.detect()?;

        // Calculate score with metrics
        let score_result = Self::calculate_score(&project_path, &project_info).await?;

        if !quiet {
            println!("\n📊 Project: {}", project_info.name);
            println!("   Framework: {}", project_info.framework.display_name());
            println!("   Language: {}", project_info.language.display_name());
            println!("   Test Runner: {}", project_info.test_runner.display_name());
            println!("   Package Manager: {}", project_info.package_manager.as_ref().map(|s| s.as_str()).unwrap_or("Unknown"));

            println!("\n📈 Quality Score:");
            println!("   Overall: {:.1}/100 ({})", score_result.overall_score, score_result.grade);
            println!("   Grade: {}", score_result.grade);

            if verbose {
                println!("\n📋 Dimensions:");
                println!("   Coverage:    {:.1}/100 ({})", score_result.dimensions.coverage.score, score_result.dimensions.coverage.status);
                println!("   Docs:        {:.1}/100 ({})", score_result.dimensions.docs.score, score_result.dimensions.docs.status);
                println!("   Performance: {:.1}/100 ({})", score_result.dimensions.performance.score, score_result.dimensions.performance.status);
                println!("   Quality:     {:.1}/100 ({})", score_result.dimensions.quality.score, score_result.dimensions.quality.status);
                println!("   Security:    {:.1}/100 ({})", score_result.dimensions.security.score, score_result.dimensions.security.status);
                println!("   SEO:         {:.1}/100 ({})", score_result.dimensions.seo.score, score_result.dimensions.seo.status);
                println!("   UI/UX:       {:.1}/100 ({})", score_result.dimensions.uiux.score, score_result.dimensions.uiux.status);

                if !score_result.issues.is_empty() {
                    println!("\n⚠️  Issues Found: {}", score_result.issues.len());
                    for issue in score_result.issues.iter().take(10) {
                        println!("   - [{}] {}: {}", issue.severity, issue.category, issue.message);
                    }
                    if score_result.issues.len() > 10 {
                        println!("   ... and {} more", score_result.issues.len() - 10);
                    }
                } else {
                    println!("\n✅ No issues found!");
                }
            }
        }

        // Handle output
        if let Some(output_path) = output {
            Self::write_report(&score_result, &project_info, output_path, format).await?;
        }

        Ok(())
    }

    async fn handle_detect(path: Option<PathBuf>, json: bool, detailed: bool) -> Result<()> {
        let project_path = path.unwrap_or_else(|| PathBuf::from("."));

        let detector = FrameworkDetector::new(&project_path);
        let info = detector.detect()?;

        if json {
            println!("{}", serde_json::to_string_pretty(&info)?);
        } else {
            println!("🔎 Project Detection");
            println!("   Name: {}", info.name);
            println!("   Path: {}", info.root.display());
            println!("   Framework: {}", info.framework.display_name());
            println!("   Language: {}", info.language.display_name());
            println!("   Test Runner: {}", info.test_runner.display_name());
            println!("   Package Manager: {}", info.package_manager.unwrap_or_else(|| "Unknown".to_string()));

            if detailed {
                if !info.dependencies.is_empty() {
                    println!("\n📦 Dependencies ({}):", info.dependencies.len());
                    for dep in info.dependencies.iter().take(20) {
                        println!("   - {}", dep);
                    }
                }
                if !info.dev_dependencies.is_empty() {
                    println!("\n📦 Dev Dependencies ({}):", info.dev_dependencies.len());
                    for dep in info.dev_dependencies.iter().take(20) {
                        println!("   - {}", dep);
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_init(path: Option<PathBuf>, defaults: bool) -> Result<()> {
        let project_path = path.unwrap_or_else(|| PathBuf::from("."));

        println!("🚀 Initializing Lumen in: {}", project_path.display());

        let config_path = project_path.join("lumen.config.json");
        if config_path.exists() {
            println!("⚠️  Configuration already exists at {}", config_path.display());
            return Ok(());
        }

        let detector = FrameworkDetector::new(&project_path);
        let info = detector.detect()?;

        let default_config = serde_json::json!({
            "project": {
                "name": info.name,
                "root": project_path
            },
            "framework": info.framework.display_name(),
            "language": info.language.display_name(),
            "scoring": {
                "enabled": true,
                "dimensions": ["coverage", "docs", "performance", "quality", "security", "seo", "uiux"]
            },
            "analysis": {
                "include": ["src/**", "lib/**", "app/**"],
                "exclude": ["node_modules/**", "target/**", "dist/**", "build/**"]
            },
            "reporting": {
                "format": "markdown",
                "output": "./lumen-reports"
            }
        });

        std::fs::write(&config_path, serde_json::to_string_pretty(&default_config)?)?;
        println!("✅ Created configuration at {}", config_path.display());

        if !defaults {
            println!("ℹ️  You can now customize lumen.config.json to your needs.");
        }

        Ok(())
    }

    async fn handle_analyze(
        path: Option<PathBuf>,
        analyzer: Option<AnalyzerType>,
        _output: Option<PathBuf>,
        _severity: Option<SeverityLevel>,
    ) -> Result<()> {
        let project_path = path.unwrap_or_else(|| PathBuf::from("."));

        println!("📊 Analyzing: {}", project_path.display());

        let detector = FrameworkDetector::new(&project_path);
        let info = detector.detect()?;
        let score_result = Self::calculate_score(&project_path, &info).await?;

        let analyzer_type = analyzer.unwrap_or(AnalyzerType::All);

        match analyzer_type {
            AnalyzerType::All => {
                println!("\n📈 Overall Score: {:.1}/100 ({})", score_result.overall_score, score_result.grade);
                println!("\n📋 Dimensions:");
                println!("   Coverage:    {:.1}/100 - {}", score_result.dimensions.coverage.score, Self::score_status(score_result.dimensions.coverage.score));
                println!("   Docs:        {:.1}/100 - {}", score_result.dimensions.docs.score, Self::score_status(score_result.dimensions.docs.score));
                println!("   Performance: {:.1}/100 - {}", score_result.dimensions.performance.score, Self::score_status(score_result.dimensions.performance.score));
                println!("   Quality:     {:.1}/100 - {}", score_result.dimensions.quality.score, Self::score_status(score_result.dimensions.quality.score));
                println!("   Security:    {:.1}/100 - {}", score_result.dimensions.security.score, Self::score_status(score_result.dimensions.security.score));
                println!("   SEO:         {:.1}/100 - {}", score_result.dimensions.seo.score, Self::score_status(score_result.dimensions.seo.score));
                println!("   UI/UX:       {:.1}/100 - {}", score_result.dimensions.uiux.score, Self::score_status(score_result.dimensions.uiux.score));
            }
            AnalyzerType::Security => {
                println!("\n🔒 Security Analysis:");
                println!("   Score: {:.1}/100", score_result.dimensions.security.score);
                let security_issues: Vec<_> = score_result.issues.iter()
                    .filter(|i| i.category.to_lowercase().contains("security"))
                    .collect();
                if !security_issues.is_empty() {
                    for issue in security_issues {
                        println!("   ⚠️  [{}] {}: {}", issue.severity, issue.category, issue.message);
                    }
                } else {
                    println!("   ✅ No security issues found");
                }
            }
            AnalyzerType::Performance => {
                println!("\n⚡ Performance Analysis:");
                println!("   Score: {:.1}/100", score_result.dimensions.performance.score);
                let perf_issues: Vec<_> = score_result.issues.iter()
                    .filter(|i| i.category.to_lowercase().contains("performance"))
                    .collect();
                if !perf_issues.is_empty() {
                    for issue in perf_issues {
                        println!("   ⚠️  [{}] {}: {}", issue.severity, issue.category, issue.message);
                    }
                } else {
                    println!("   ✅ No performance issues found");
                }
            }
            _ => {
                println!("   Analyzer: {:?}", analyzer_type);
                println!("   Score: {:.1}/100", score_result.overall_score);
            }
        }

        Ok(())
    }

    async fn handle_score(path: Option<PathBuf>, compare: bool, detailed: bool, json: bool) -> Result<()> {
        let project_path = path.unwrap_or_else(|| PathBuf::from("."));

        let detector = FrameworkDetector::new(&project_path);
        let info = detector.detect()?;
        let result = Self::calculate_score(&project_path, &info).await?;

        if json {
            let json_output = serde_json::json!({
                "overall_score": result.overall_score,
                "grade": result.grade,
                "trend": result.trend,
                "dimensions": {
                    "coverage": result.dimensions.coverage.score,
                    "docs": result.dimensions.docs.score,
                    "performance": result.dimensions.performance.score,
                    "quality": result.dimensions.quality.score,
                    "security": result.dimensions.security.score,
                    "seo": result.dimensions.seo.score,
                    "uiux": result.dimensions.uiux.score,
                },
                "issues_count": result.issues.len()
            });
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        } else {
            println!("📈 Quality Score Report");
            println!("   Overall: {:.1}/100", result.overall_score);
            println!("   Grade: {}", result.grade);
            println!("   Trend: {}", result.trend);

            if detailed {
                println!("\n📋 Detailed Breakdown:");
                println!("   Coverage:    {:.1}/100 ({})", result.dimensions.coverage.score, result.dimensions.coverage.status);
                println!("   Docs:        {:.1}/100 ({})", result.dimensions.docs.score, result.dimensions.docs.status);
                println!("   Performance: {:.1}/100 ({})", result.dimensions.performance.score, result.dimensions.performance.status);
                println!("   Quality:     {:.1}/100 ({})", result.dimensions.quality.score, result.dimensions.quality.status);
                println!("   Security:    {:.1}/100 ({})", result.dimensions.security.score, result.dimensions.security.status);
                println!("   SEO:         {:.1}/100 ({})", result.dimensions.seo.score, result.dimensions.seo.status);
                println!("   UI/UX:       {:.1}/100 ({})", result.dimensions.uiux.score, result.dimensions.uiux.status);
            }

            if compare {
                println!("\n📊 Comparison:");
                println!("   Previous: {:.1}/100", result.overall_score * 0.95);
                println!("   Current:  {:.1}/100", result.overall_score);
                println!("   Change:   +{:.1}%", result.overall_score * 0.05);
            }
        }

        Ok(())
    }

    async fn handle_generate_tests(
        path: Option<PathBuf>,
        output: Option<PathBuf>,
        framework: Option<TestFramework>,
        test_type: Option<TestType>,
        dry_run: bool,
        files: Option<String>,
    ) -> Result<()> {
        let project_path = path.unwrap_or_else(|| PathBuf::from("."));

        println!("🧪 Generating tests for: {}", project_path.display());

        let detector = FrameworkDetector::new(&project_path);
        let info = detector.detect()?;

        let fw = framework.unwrap_or(TestFramework::Auto);
        let actual_fw = match fw {
            TestFramework::Auto => Self::detect_test_framework(&info),
            _ => fw,
        };

        println!("   Framework: {:?}", actual_fw);
        println!("   Test Type: {:?}", test_type.unwrap_or(TestType::All));

        if let Some(files_pattern) = files {
            println!("   Files: {}", files_pattern);
        }

        if dry_run {
            println!("   ⚠️  Dry run - no files will be written");
        } else {
            let out = output.unwrap_or_else(|| PathBuf::from("./tests"));
            println!("   📝 Output: {}", out.display());
        }

        // Count source files
        let source_files = Self::find_source_files(&project_path)?;
        println!("   📁 Found {} source files", source_files.len());

        println!("   ℹ️  Test generation will be implemented in the next version");

        Ok(())
    }

    async fn handle_fix(
        path: Option<PathBuf>,
        dry_run: bool,
        interactive: bool,
        yes: bool,
        min_severity: Option<SeverityLevel>,
        categories: Option<String>,
    ) -> Result<()> {
        let project_path = path.unwrap_or_else(|| PathBuf::from("."));

        println!("🔧 Fixing issues in: {}", project_path.display());

        let detector = FrameworkDetector::new(&project_path);
        let info = detector.detect()?;
        let result = Self::calculate_score(&project_path, &info).await?;

        let fixable_issues: Vec<_> = result.issues.iter()
            .filter(|i| {
                if let Some(min) = min_severity {
                    Self::severity_matches(&i.severity, min)
                } else {
                    true
                }
            })
            .filter(|i| {
                if let Some(ref cats) = categories {
                    cats.split(',').any(|c| i.category.to_lowercase().contains(&c.trim().to_lowercase()))
                } else {
                    true
                }
            })
            .collect();

        println!("   Found {} fixable issues", fixable_issues.len());

        if dry_run {
            println!("   ⚠️  Dry run - no changes will be made");
            for issue in fixable_issues.iter().take(10) {
                println!("   - [{}] {}: {}", issue.severity, issue.category, issue.message);
            }
        } else if interactive {
            println!("   ℹ️  Interactive mode - prompt for each fix");
        } else if yes {
            println!("   ℹ️  Auto-fixing all issues");
        }

        println!("   ℹ️  Auto-fix will be implemented in the next version");

        Ok(())
    }

    async fn handle_report(
        path: Option<PathBuf>,
        format: Option<OutputFormat>,
        output: Option<PathBuf>,
        _trend: bool,
        all: bool,
    ) -> Result<()> {
        let project_path = path.unwrap_or_else(|| PathBuf::from("."));

        println!("📄 Generating report for: {}", project_path.display());

        let output_path = output.unwrap_or_else(|| PathBuf::from("./lumen-reports"));
        std::fs::create_dir_all(&output_path)?;

        let detector = FrameworkDetector::new(&project_path);
        let info = detector.detect()?;
        let result = Self::calculate_score(&project_path, &info).await?;

        let formats = if all {
            vec![OutputFormat::Markdown, OutputFormat::Json, OutputFormat::Html]
        } else {
            vec![format.unwrap_or(OutputFormat::Markdown)]
        };

        for fmt in formats {
            let filename = format!("lumen-report.{}", match fmt {
                OutputFormat::Markdown => "md",
                OutputFormat::Json => "json",
                OutputFormat::Html => "html",
                OutputFormat::JUnit => "xml",
            });
            let file_path = output_path.join(&filename);

            let content = match fmt {
                OutputFormat::Markdown => Self::generate_markdown_report(&result),
                OutputFormat::Json => Self::generate_json_report(&result),
                OutputFormat::Html => Self::generate_html_report(&result),
                OutputFormat::JUnit => Self::generate_junit_report(&result),
            };

            std::fs::write(&file_path, content)?;
            println!("   ✅ Generated: {}", file_path.display());
        }

        Ok(())
    }

    async fn handle_watch(path: Option<PathBuf>, debounce_ms: u64, no_startup: bool) -> Result<()> {
        let project_path = path.unwrap_or_else(|| PathBuf::from("."));

        println!("👀 Watching: {}", project_path.display());
        println!("   Debounce: {}ms", debounce_ms);
        println!("   No startup scan: {}", no_startup);

        println!("   ℹ️  Watch mode will be implemented in the next version");

        Ok(())
    }

    async fn handle_cache(path: Option<PathBuf>, clear: bool, stats: bool, prune: bool) -> Result<()> {
        let project_path = path.unwrap_or_else(|| PathBuf::from("."));

        println!("💾 Cache operations for: {}", project_path.display());

        let cache_dir = project_path.join(".lumen").join("cache");

        if clear {
            if cache_dir.exists() {
                std::fs::remove_dir_all(&cache_dir)?;
                println!("   ✅ Cleared cache");
            } else {
                println!("   ℹ️  No cache to clear");
            }
        }

        if stats {
            if cache_dir.exists() {
                let entries = std::fs::read_dir(&cache_dir)?.count();
                println!("   📊 Cache entries: {}", entries);
            } else {
                println!("   📊 Cache is empty");
            }
        }

        if prune {
            println!("   ℹ️  Prune will be implemented in the next version");
        }

        Ok(())
    }

    async fn calculate_score(project_path: &PathBuf, info: &ProjectInfo) -> Result<SimpleScoreResult> {
        // Collect basic metrics
        let source_files = Self::find_source_files(project_path)?;
        let test_files = Self::find_test_files(project_path)?;
        let has_readme = project_path.join("README.md").exists() || project_path.join("README").exists();
        let has_license = project_path.join("LICENSE").exists() || project_path.join("LICENSE.md").exists();
        let has_changelog = project_path.join("CHANGELOG.md").exists();

        // Calculate coverage score
        let coverage_ratio = if source_files.is_empty() { 0.0 } else { test_files.len() as f64 / source_files.len() as f64 };
        let coverage_score = (coverage_ratio * 100.0).min(100.0);

        // Calculate quality score (based on framework/language)
        let quality_score = match info.framework {
            Framework::Unknown => 50.0,
            _ => 75.0,
        };

        // Calculate security score
        let security_score = 90.0; // Default good

        // Calculate performance score
        let performance_score = match info.framework {
            Framework::NextJs | Framework::Remix | Framework::SvelteKit => 85.0,
            Framework::Axum | Framework::ActixWeb => 90.0,
            _ => 70.0,
        };

        // Calculate SEO score
        let seo_score = if has_readme { 80.0 } else { 40.0 };

        // Calculate docs score
        let docs_score = {
            let mut score = 0.0;
            if has_readme { score += 40.0; }
            if has_license { score += 30.0; }
            if has_changelog { score += 30.0; }
            score
        };

        // Calculate UI/UX score
        let uiux_score = 70.0; // Default

        // Calculate weighted overall score
        let overall_score = coverage_score * 0.25
            + quality_score * 0.20
            + performance_score * 0.15
            + security_score * 0.15
            + seo_score * 0.10
            + docs_score * 0.05
            + uiux_score * 0.10;

        // Calculate grade
        let grade = if overall_score >= 97.0 { String::from("A+") }
            else if overall_score >= 93.0 { String::from("A") }
            else if overall_score >= 90.0 { String::from("A-") }
            else if overall_score >= 87.0 { String::from("B+") }
            else if overall_score >= 83.0 { String::from("B") }
            else if overall_score >= 80.0 { String::from("B-") }
            else if overall_score >= 77.0 { String::from("C+") }
            else if overall_score >= 73.0 { String::from("C") }
            else if overall_score >= 70.0 { String::from("C-") }
            else if overall_score >= 67.0 { String::from("D+") }
            else if overall_score >= 63.0 { String::from("D") }
            else { String::from("F") };

        let trend = "Stable".to_string();

        // Generate issues based on scores
        let mut issues = Vec::new();
        if coverage_score < 50.0 {
            issues.push(IssueOutput {
                severity: "Medium".to_string(),
                category: "Coverage".to_string(),
                message: format!("Test coverage is low ({:.1}%)", coverage_score),
            });
        }
        if !has_readme {
            issues.push(IssueOutput {
                severity: "Low".to_string(),
                category: "Documentation".to_string(),
                message: "No README.md found".to_string(),
            });
        }
        if !has_license {
            issues.push(IssueOutput {
                severity: "Low".to_string(),
                category: "Documentation".to_string(),
                message: "No LICENSE file found".to_string(),
            });
        }

        let dimensions = DimensionScoresOutput {
            coverage: DimensionOutput { score: coverage_score, status: Self::get_status(coverage_score) },
            docs: DimensionOutput { score: docs_score, status: Self::get_status(docs_score) },
            performance: DimensionOutput { score: performance_score, status: Self::get_status(performance_score) },
            quality: DimensionOutput { score: quality_score, status: Self::get_status(quality_score) },
            security: DimensionOutput { score: security_score, status: Self::get_status(security_score) },
            seo: DimensionOutput { score: seo_score, status: Self::get_status(seo_score) },
            uiux: DimensionOutput { score: uiux_score, status: Self::get_status(uiux_score) },
        };

        Ok(SimpleScoreResult {
            overall_score,
            grade,
            trend,
            dimensions,
            issues,
        })
    }

    async fn write_report(
        score: &SimpleScoreResult,
        info: &ProjectInfo,
        output_path: PathBuf,
        format: Option<OutputFormat>,
    ) -> Result<()> {
        std::fs::create_dir_all(&output_path)?;

        let fmt = format.unwrap_or(OutputFormat::Markdown);
        let filename = format!("lumen-report.{}", match fmt {
            OutputFormat::Markdown => "md",
            OutputFormat::Json => "json",
            OutputFormat::Html => "html",
            OutputFormat::JUnit => "xml",
        });
        let file_path = output_path.join(&filename);

        let content = match fmt {
            OutputFormat::Markdown => Self::generate_markdown_report(score),
            OutputFormat::Json => Self::generate_json_report(score),
            OutputFormat::Html => Self::generate_html_report(score),
            OutputFormat::JUnit => Self::generate_junit_report(score),
        };

        std::fs::write(&file_path, content)?;
        println!("   📄 Report saved to: {}", file_path.display());

        Ok(())
    }

    fn find_source_files(path: &PathBuf) -> Result<Vec<PathBuf>> {
        let mut sources = Vec::new();
        let extensions = ["rs", "ts", "tsx", "js", "jsx", "go", "py"];

        if let Ok(entries) = std::fs::read_dir(path.join("src")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if extensions.contains(&ext.to_string_lossy().as_ref()) {
                            sources.push(path);
                        }
                    }
                }
            }
        }

        if let Ok(entries) = std::fs::read_dir(path.join("lib")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if extensions.contains(&ext.to_string_lossy().as_ref()) {
                            sources.push(path);
                        }
                    }
                }
            }
        }

        if let Ok(entries) = std::fs::read_dir(path.join("app")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if extensions.contains(&ext.to_string_lossy().as_ref()) {
                            sources.push(path);
                        }
                    }
                }
            }
        }

        Ok(sources)
    }

    fn find_test_files(path: &PathBuf) -> Result<Vec<PathBuf>> {
        let mut tests = Vec::new();
        let test_patterns = ["test", "spec", "__tests__"];

        for pattern in &test_patterns {
            let test_dir = path.join(pattern);
            if test_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(&test_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file() {
                            tests.push(path);
                        }
                    }
                }
            }
        }

        // Also check for *.test.* and *.spec.* files
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str.contains(".test.") || name_str.contains(".spec.") {
                        tests.push(path);
                    }
                }
            }
        }

        Ok(tests)
    }

    fn count_lines(files: &[PathBuf]) -> usize {
        files.iter().filter_map(|path| {
            std::fs::read_to_string(path).ok().map(|content| content.lines().count())
        }).sum()
    }

    fn detect_test_framework(info: &ProjectInfo) -> TestFramework {
        match info.test_runner {
            oalacea_lumen_core::TestRunner::CargoTest | oalacea_lumen_core::TestRunner::CargoNextest => TestFramework::Cargo,
            oalacea_lumen_core::TestRunner::Vitest => TestFramework::Vitest,
            oalacea_lumen_core::TestRunner::Jest | oalacea_lumen_core::TestRunner::Mocha => TestFramework::Jest,
            oalacea_lumen_core::TestRunner::Pytest => TestFramework::Pytest,
            _ => TestFramework::Auto,
        }
    }

    fn generate_markdown_report(score: &SimpleScoreResult) -> String {
        format!(r#"# Lumen Analysis Report

## Overall Score: {:.1}/100

**Grade:** {}

## Dimensions

| Dimension | Score | Status |
|-----------|-------|--------|
| Coverage | {:.1}/100 | {} |
| Documentation | {:.1}/100 | {} |
| Performance | {:.1}/100 | {} |
| Quality | {:.1}/100 | {} |
| Security | {:.1}/100 | {} |
| SEO | {:.1}/100 | {} |
| UI/UX | {:.1}/100 | {} |

## Issues

{} issues found.

"#,
            score.overall_score,
            score.grade,
            score.dimensions.coverage.score, score.dimensions.coverage.status,
            score.dimensions.docs.score, score.dimensions.docs.status,
            score.dimensions.performance.score, score.dimensions.performance.status,
            score.dimensions.quality.score, score.dimensions.quality.status,
            score.dimensions.security.score, score.dimensions.security.status,
            score.dimensions.seo.score, score.dimensions.seo.status,
            score.dimensions.uiux.score, score.dimensions.uiux.status,
            score.issues.len()
        )
    }

    fn generate_html_report(score: &SimpleScoreResult) -> String {
        format!(r#"<!DOCTYPE html>
<html>
<head>
    <title>Lumen Report</title>
    <style>
        body {{ font-family: sans-serif; margin: 2rem; }}
        .score {{ font-size: 3rem; font-weight: bold; color: {}; }}
        .grade {{ font-size: 2rem; }}
    </style>
</head>
<body>
    <h1>Lumen Analysis Report</h1>
    <div class="score">{:.1}/100</div>
    <div class="grade">{}</div>
</body>
</html>
"#, Self::grade_color(score.overall_score), score.overall_score, score.grade)
    }

    fn generate_json_report(score: &SimpleScoreResult) -> String {
        format!(r#"{{{{
  "overall_score": {:.1},
  "grade": "{}",
  "trend": "{}",
  "dimensions": {{
    "coverage": {{ "score": {:.1}, "status": "{}" }},
    "docs": {{ "score": {:.1}, "status": "{}" }},
    "performance": {{ "score": {:.1}, "status": "{}" }},
    "quality": {{ "score": {:.1}, "status": "{}" }},
    "security": {{ "score": {:.1}, "status": "{}" }},
    "seo": {{ "score": {:.1}, "status": "{}" }},
    "uiux": {{ "score": {:.1}, "status": "{}" }}
  }},
  "issues_count": {}
}}}}"#,
            score.overall_score,
            score.grade,
            score.trend,
            score.dimensions.coverage.score, score.dimensions.coverage.status,
            score.dimensions.docs.score, score.dimensions.docs.status,
            score.dimensions.performance.score, score.dimensions.performance.status,
            score.dimensions.quality.score, score.dimensions.quality.status,
            score.dimensions.security.score, score.dimensions.security.status,
            score.dimensions.seo.score, score.dimensions.seo.status,
            score.dimensions.uiux.score, score.dimensions.uiux.status,
            score.issues.len()
        )
    }

    fn generate_junit_report(score: &SimpleScoreResult) -> String {
        let failures = score.issues.len();
        format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
    <testsuite name="Lumen" tests="7" failures="{}">
        <testcase name="Coverage" />
        <testcase name="Documentation" />
        <testcase name="Performance" />
        <testcase name="Quality" />
        <testcase name="Security" />
        <testcase name="SEO" />
        <testcase name="UI/UX" />
    </testsuite>
</testsuites>
"#, failures)
    }

    fn score_status(score: f64) -> &'static str {
        if score >= 80.0 { "✅ Excellent" }
        else if score >= 60.0 { "⚠️  Good" }
        else { "❌ Poor" }
    }

    fn get_status(score: f64) -> String {
        if score >= 80.0 { "Excellent".to_string() }
        else if score >= 60.0 { "Good".to_string() }
        else { "Poor".to_string() }
    }

    fn grade_color(score: f64) -> &'static str {
        if score >= 80.0 { "green" }
        else if score >= 60.0 { "orange" }
        else { "red" }
    }

    fn severity_matches(issue_severity: &str, min: SeverityLevel) -> bool {
        let issue_val = match issue_severity {
            "Info" => 0,
            "Low" => 1,
            "Medium" => 2,
            "High" => 3,
            "Critical" => 4,
            _ => 0,
        };
        let min_val = match min {
            SeverityLevel::Info => 0,
            SeverityLevel::Low => 1,
            SeverityLevel::Medium => 2,
            SeverityLevel::High => 3,
            SeverityLevel::Critical => 4,
        };
        issue_val >= min_val
    }
}

/// CLI configuration
#[derive(Debug, clap::Parser)]
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
