//! Lumen - AI-powered code analysis and test generation toolkit

#![warn(missing_docs)]
#![warn(clippy::all)]

use clap::Parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use glob::glob;

mod cli;
mod watch;

use cli::{
    AnalyzerType, Commands, LumenCli, OutputFormat, SeverityLevel, TestFramework,
    TestType,
};

/// Lumen version constant
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = LumenCli::parse();

    // Setup colors
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Setup logging
    let log_level = if cli.verbose {
        tracing::Level::DEBUG
    } else if cli.quiet {
        tracing::Level::WARN
    } else {
        tracing::Level::INFO
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Print banner if not quiet
    if !cli.quiet {
        print_banner();
    }

    // Run the appropriate command
    let result = run(cli).await;

    // Print final status
    if result.is_err() && !matches!(result, Err(ref e) if e.to_string().contains("exit code")) {
        eprintln!("\n{}", "Scan completed with errors.".red());
        std::process::exit(1);
    }

    result
}

async fn run(cli: LumenCli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Scan {
            path,
            output,
            format,
            dimensions,
            severity,
        } => {
            cmd_scan(
                path.unwrap_or_else(|| ".".into()),
                output,
                format,
                dimensions,
                severity,
            )
            .await?
        }
        Commands::Init { path, defaults } => {
            cmd_init(path.unwrap_or_else(|| ".".into()), defaults)?
        }
        Commands::Detect {
            path,
            json,
            detailed,
        } => cmd_detect(path.unwrap_or_else(|| ".".into()), json, detailed)?,
        Commands::Analyze {
            path,
            analyzer,
            output,
            severity,
        } => {
            cmd_analyze(
                path.unwrap_or_else(|| ".".into()),
                analyzer,
                output,
                severity,
            )?
        }
        Commands::Score {
            path,
            compare,
            detailed,
            json,
        } => {
            cmd_score(
                path.unwrap_or_else(|| ".".into()),
                compare,
                detailed,
                json,
            )?
        }
        Commands::GenerateTests {
            path,
            output,
            framework,
            test_type,
            dry_run,
            files,
        } => {
            cmd_generate_tests(
                path.unwrap_or_else(|| ".".into()),
                output,
                framework,
                test_type,
                dry_run,
                files,
            )?
        }
        Commands::Fix {
            path,
            dry_run,
            interactive,
            yes,
            min_severity,
            categories,
        } => {
            cmd_fix(
                path.unwrap_or_else(|| ".".into()),
                dry_run,
                interactive,
                yes,
                min_severity,
                categories,
            )?
        }
        Commands::Report {
            path,
            format,
            output,
            trend,
            all,
        } => {
            cmd_report(
                path.unwrap_or_else(|| ".".into()),
                format,
                output,
                trend,
                all,
            )?
        }
        Commands::Watch {
            path,
            include,
            exclude,
            include_patterns,
            exclude_patterns,
            debounce_ms,
            no_startup,
        } => {
            cmd_watch(
                path.unwrap_or_else(|| ".".into()),
                include,
                exclude,
                include_patterns,
                exclude_patterns,
                debounce_ms,
                no_startup,
            )?
        }
        Commands::Cache {
            path,
            clear,
            stats,
            prune,
        } => {
            cmd_cache(
                path.unwrap_or_else(|| ".".into()),
                clear,
                stats,
                prune,
            )?
        }
    }

    Ok(())
}

/// Scan command - Full project analysis
async fn cmd_scan(
    path: PathBuf,
    output: Option<PathBuf>,
    format: Option<OutputFormat>,
    _dimensions: Option<String>,
    _severity: Option<SeverityLevel>,
) -> anyhow::Result<()> {
    let start = std::time::Instant::now();

    // Validate path
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }

    let steps = 6;
    let pb = ProgressBar::new(steps);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );

    // Step 1: Initialize
    pb.set_message("Initializing Lumen...");
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    pb.inc(1);

    // Step 2: Detect framework
    pb.set_message("Detecting framework and tools...");
    let detector = lumenx_detect::FrameworkDetector::new(&path);
    let project_info = detector.detect()?;
    pb.inc(1);

    // Step 3: Scan files
    pb.set_message("Scanning source files...");
    let source_files = scan_source_files(&path)?;
    let test_files = scan_test_files(&path)?;
    let config_files = scan_config_files(&path)?;
    let total_lines = count_lines(&source_files)?;
    pb.inc(1);

    // Step 4: Analyze code
    pb.set_message("Analyzing code quality...");
    let project = lumenx_core::Project {
        info: project_info.clone(),
        source_files: source_files.clone(),
        test_files: test_files.clone(),
        config_files: config_files.clone(),
        total_lines,
        coverage: None,
    };

    let analyzer = lumenx_analyze::Analyzer::new(project.clone());
    let analysis_result = analyzer.analyze()?;
    pb.inc(1);

    // Step 5: Calculate scores
    pb.set_message("Calculating quality scores...");
    let dimension_scores = calculate_dimension_scores(
        &analysis_result,
        &source_files,
        total_lines,
        &test_files,
    );

    let overall = dimension_scores.weighted_sum();
    let grade = lumenx_score::Grade::from_score(overall);
    let commit_sha = get_current_commit_sha(&path);

    let mut score = lumenx_score::ProjectScore {
        project_name: project_info.name.clone(),
        commit_sha,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        overall,
        grade,
        dimensions: dimension_scores,
        trend: None,
        metadata: lumenx_score::ScoreMetadata {
            scorer_version: VERSION.to_string(),
            scan_duration_ms: start.elapsed().as_millis() as u64,
            files_scanned: source_files.len(),
            lines_of_code: total_lines,
            language_breakdown: calculate_language_breakdown(&source_files),
            profile: "standard".to_string(),
        },
    };
    pb.inc(1);

    // Step 6: Generate reports
    pb.set_message("Generating reports...");
    let output_dir = output.unwrap_or_else(|| {
        path.join("lumen-reports")
    });

    let report_format = format.unwrap_or(OutputFormat::Markdown);
    let report_gen = lumenx_report::ReportGenerator::new(
        project_info.clone(),
        score.clone(),
        output_dir.clone(),
    );

    let format_match = match report_format {
        OutputFormat::Markdown => lumenx_report::ReportFormat::Markdown,
        OutputFormat::Json => lumenx_report::ReportFormat::Json,
        OutputFormat::Html => lumenx_report::ReportFormat::Html,
        OutputFormat::JUnit => lumenx_report::ReportFormat::JUnit,
    };

    let reports = report_gen.generate(format_match)?;

    // Generate AI-ready fixes.json
    let fixes_json = lumenx_report::formats::generate_fixes_for_ai(&project_info, &score.dimensions);
    let fixes_path = output_dir.join("fixes.json");
    fs::create_dir_all(&output_dir)?;
    fs::write(&fixes_path, fixes_json)?;

    pb.finish_with_message("Scan complete!");

    // Print summary
    println!();
    print_scan_summary(&score, &analysis_result, &source_files, start.elapsed());

    // Print report location
    println!("\n{}", "Reports generated:".cyan().bold());
    for report in &reports {
        println!("  - {}", report.path.display());
    }
    println!("  - {} (AI-ready fixes)", "fixes.json".green());

    // Print fixes summary
    let fixable_count = analysis_result.all_findings().filter(|i| i.suggestion.is_some()).count();
    if fixable_count > 0 {
        println!("\n{}", "Fixable issues found:".yellow().bold());
        println!("  {} issues can be automatically fixed", fixable_count);
        println!("  Run '{} {}' to apply fixes", "lumen fix".green(), "--interactive".white());
    }

    Ok(())
}

/// Initialize command - Create Lumen config
fn cmd_init(path: PathBuf, defaults: bool) -> anyhow::Result<()> {
    println!(
        "{} {}",
        "Initializing Lumen in".cyan(),
        path.display().to_string().white()
    );

    // Validate path
    if !path.exists() {
        fs::create_dir_all(&path)?;
        println!("{} Created directory {}", "+".green(), path.display());
    }

    // Check for existing config
    let config_path = path.join("lumen.toml");
    if config_path.exists() {
        println!(
            "\n{} {}",
            "!".yellow(),
            "lumen.toml already exists".yellow()
        );

        if !defaults {
            print!("Overwrite? [y/N] ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            if !input.trim().to_lowercase().starts_with('y') {
                println!("Skipping initialization.");
                return Ok(());
            }
        }
    }

    // Create .lumen directory
    let lumen_dir = path.join(".lumen");
    fs::create_dir_all(&lumen_dir)?;

    // Create default config
    let config_content = get_default_config();
    fs::write(&config_path, config_content)?;
    println!("{} {}", "✓".green(), config_path.display());

    // Create .gitignore entry
    let gitignore_path = path.join(".gitignore");
    let lumen_ignore = "\n# Lumen\n.lumen/\nlumen-reports/\n";
    if gitignore_path.exists() {
        let existing = fs::read_to_string(&gitignore_path)?;
        if !existing.contains(".lumen/") {
            fs::write(&gitignore_path, existing + lumen_ignore)?;
            println!("{} Updated .gitignore", "+".green());
        }
    }

    println!("\n{}", "Lumen initialized successfully!".green().bold());
    println!("Run '{} {}' to analyze your project.", "lumen scan".cyan(), "--format markdown".white());

    Ok(())
}

/// Cache command - Manage analysis cache
fn cmd_cache(path: PathBuf, clear: bool, stats: bool, prune: bool) -> anyhow::Result<()> {
    use lumenx_testgen::{TestGenCache, CacheConfig, compute_file_hash};

    let cache_dir = path.join(".lumen").join("cache");
    let config = CacheConfig {
        cache_dir: cache_dir.clone(),
        ..Default::default()
    };

    if clear {
        println!("{} Clearing cache...", "🗑️".red());
        let cache = TestGenCache::new(config)?;
        cache.clear()?;
        println!("{} Cache cleared!", "✓".green());
        return Ok(());
    }

    if stats {
        println!("{} Cache statistics:", "📊".cyan());
        let cache = TestGenCache::new(config)?;
        match cache.stats() {
            Ok(stats) => {
                println!("  Total entries: {}", stats.total_entries);
                println!("  Memory entries: {}", stats.memory_entries);
                println!("  Total size: {:.2} MB", stats.total_size_mb);
                if let Some(oldest) = stats.oldest_entry {
                    let elapsed = oldest.elapsed().unwrap_or_default().as_secs();
                    println!("  Oldest entry: {} seconds ago", elapsed);
                }
                if let Some(newest) = stats.newest_entry {
                    let elapsed = newest.elapsed().unwrap_or_default().as_secs();
                    println!("  Newest entry: {} seconds ago", elapsed);
                }
            }
            Err(e) => {
                println!("  No cache found or error: {}", e);
            }
        }
        return Ok(());
    }

    if prune {
        println!("{} Pruning old cache entries...", "✂️".yellow());
        let cache = TestGenCache::new(config)?;
        let pruned = cache.prune()?;
        println!("{} Pruned {} entries", "✓".green(), pruned);
        return Ok(());
    }

    // Show cache info by default
    println!("{} Cache information:", "ℹ️".cyan());
    println!("  Location: {}", cache_dir.display());
    let cache = TestGenCache::new(config)?;
    match cache.stats() {
        Ok(stats) => {
            if stats.total_entries > 0 {
                println!("  Status: {}Active ({} entries)", "🟢".green(), stats.total_entries);
            } else {
                println!("  Status: {}Empty (no cached analysis)", "⚪".white());
            }
        }
        Err(_) => {
            println!("  Status: {}Not initialized", "⚪".white());
        }
    }

    println!("\n{} Usage:", "💡".yellow());
    println!("  lumen cache --clear     Clear all cache");
    println!("  lumen cache --stats     Show cache statistics");
    println!("  lumen cache --prune     Remove old entries");

    Ok(())
}

/// Detect command - Detect framework and tools
fn cmd_detect(path: PathBuf, json: bool, detailed: bool) -> anyhow::Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }

    let detector = lumenx_detect::FrameworkDetector::new(&path);
    let info = detector.detect()?;

    if json {
        let json_output = serde_json::to_string_pretty(&info)?;
        println!("{}", json_output);
        return Ok(());
    }

    // Pretty print
    println!("\n{}", "Project Detection Results".cyan().bold());
    println!("{}", "═".repeat(60).cyan());

    print_detection_row("Project Name", &info.name);
    print_detection_row("Framework", info.framework.display_name());
    print_detection_row("Language", info.language.display_name());
    print_detection_row("Test Runner", info.test_runner.display_name());
    print_detection_row(
        "Package Manager",
        info.package_manager.as_deref().unwrap_or("Unknown"),
    );

    if detailed {
        println!("\n{}", "Details".cyan().bold());
        println!("{}", "─".repeat(60).cyan());

        // Count source files by type
        let source_counts = count_files_by_extension(&path)?;
        for (ext, count) in source_counts.iter().take(10) {
            println!("  {} files: {}", ext, count);
        }

        // Dependencies
        if !info.dependencies.is_empty() {
            println!("\n{}", "Dependencies".cyan().bold());
            println!("{}", "─".repeat(60).cyan());
            for dep in info.dependencies.iter().take(10) {
                println!("  - {}", dep);
            }
            if info.dependencies.len() > 10 {
                println!("  ... and {} more", info.dependencies.len() - 10);
            }
        }
    }

    println!();

    Ok(())
}

/// Analyze command - Run specific analyzers
fn cmd_analyze(
    path: PathBuf,
    analyzer: Option<AnalyzerType>,
    output: Option<PathBuf>,
    severity: Option<SeverityLevel>,
) -> anyhow::Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }

    let detector = lumenx_detect::FrameworkDetector::new(&path);
    let project_info = detector.detect()?;

    let source_files = scan_source_files(&path)?;
    let test_files = scan_test_files(&path)?;
    let config_files = scan_config_files(&path)?;
    let total_lines = count_lines(&source_files)?;

    let project = lumenx_core::Project {
        info: project_info,
        source_files: source_files.clone(),
        test_files,
        config_files,
        total_lines,
        coverage: None,
    };

    let analyzer_engine = lumenx_analyze::Analyzer::new(project);
    let analysis_result = analyzer_engine.analyze()?;

    let min_severity = severity.unwrap_or(SeverityLevel::Low);

    // Filter by analyzer type
    let findings = match analyzer.unwrap_or(AnalyzerType::All) {
        AnalyzerType::Static => analysis_result.static_findings,
        AnalyzerType::Security => analysis_result.security_findings,
        AnalyzerType::Dependency => analysis_result.dependency_findings,
        AnalyzerType::Performance => analysis_result.performance_findings,
        AnalyzerType::Seo => analysis_result.seo_findings,
        AnalyzerType::Uiux => analysis_result.uiux_findings,
        AnalyzerType::Docs => analysis_result.docs_findings,
        AnalyzerType::All => {
            let mut all = Vec::new();
            all.extend(analysis_result.static_findings);
            all.extend(analysis_result.security_findings);
            all.extend(analysis_result.dependency_findings);
            all.extend(analysis_result.performance_findings);
            all.extend(analysis_result.seo_findings);
            all.extend(analysis_result.uiux_findings);
            all.extend(analysis_result.docs_findings);
            all
        }
    };

    // Filter by severity
    let findings: Vec<_> = findings
        .into_iter()
        .filter(|f| {
            SeverityLevel::from_lumen_score(f.severity) >= min_severity
        })
        .collect();

    // Print findings
    print_findings_table(&findings);

    // Write to output file if specified
    if let Some(output_path) = output {
        let json_output = serde_json::to_string_pretty(&findings)?;
        fs::write(&output_path, json_output)?;
        println!("\n{}", format!("Results written to {}", output_path.display()).cyan());
    }

    Ok(())
}

/// Score command - Calculate and display scores
fn cmd_score(
    path: PathBuf,
    compare: bool,
    detailed: bool,
    json: bool,
) -> anyhow::Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }

    let detector = lumenx_detect::FrameworkDetector::new(&path);
    let project_info = detector.detect()?;

    // Run analysis
    let source_files = scan_source_files(&path)?;
    let test_files = scan_test_files(&path)?;
    let config_files = scan_config_files(&path)?;
    let total_lines = count_lines(&source_files)?;

    let project = lumenx_core::Project {
        info: project_info.clone(),
        source_files: source_files.clone(),
        test_files: test_files.clone(),
        config_files,
        total_lines,
        coverage: None,
    };

    let analyzer = lumenx_analyze::Analyzer::new(project.clone());
    let analysis_result = analyzer.analyze()?;

    // Calculate scores
    let dimension_scores = calculate_dimension_scores(
        &analysis_result,
        &source_files,
        total_lines,
        &test_files,
    );

    let overall = dimension_scores.weighted_sum();
    let grade = lumenx_score::Grade::from_score(overall);

    let score = lumenx_score::ProjectScore {
        project_name: project_info.name.clone(),
        commit_sha: get_current_commit_sha(&path),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        overall,
        grade,
        dimensions: dimension_scores,
        trend: None,
        metadata: lumenx_score::ScoreMetadata::default(),
    };

    if json {
        let json_output = serde_json::to_string_pretty(&score)?;
        println!("{}", json_output);
        return Ok(());
    }

    // Print score
    if detailed {
        print_detailed_score(&score);
    } else {
        print_score_summary(&score);
    }

    // Compare with history if requested
    if compare {
        print_history_comparison(&path, &score)?;
    }

    Ok(())
}

/// Generate tests command
fn cmd_generate_tests(
    path: PathBuf,
    output: Option<PathBuf>,
    framework: Option<TestFramework>,
    _test_type: Option<TestType>,
    dry_run: bool,
    _files: Option<String>,
) -> anyhow::Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }

    let detector = lumenx_detect::FrameworkDetector::new(&path);
    let project_info = detector.detect()?;

    println!("\n{}", "Test Generation".cyan().bold());
    println!("Framework: {}", project_info.framework.display_name());

    // Map CLI framework to internal framework
    let test_framework = match framework.unwrap_or(TestFramework::Auto) {
        TestFramework::Auto => match project_info.framework {
            lumenx_core::Framework::NextJs
            | lumenx_core::Framework::Remix
            | lumenx_core::Framework::ViteReact => lumenx_testgen::TestFramework::Vitest,
            lumenx_core::Framework::Axum
            | lumenx_core::Framework::ActixWeb
            | lumenx_core::Framework::Rocket => lumenx_testgen::TestFramework::RustBuiltIn,
            _ => lumenx_testgen::TestFramework::Vitest,
        },
        TestFramework::Vitest => lumenx_testgen::TestFramework::Vitest,
        TestFramework::Jest => lumenx_testgen::TestFramework::Jest,
        TestFramework::Cargo => lumenx_testgen::TestFramework::RustBuiltIn,
        TestFramework::Nextest => lumenx_testgen::TestFramework::RustBuiltIn,
        TestFramework::Pytest => lumenx_testgen::TestFramework::Pytest,
    };

    // Create a simple test generation result
    let source_files = scan_source_files(&path)?;
    let test_dir = output.unwrap_or_else(|| path.join("tests"));

    println!("\n{}", "Analysis Summary".cyan().bold());
    println!("  Source files found: {}", source_files.len());
    println!("  Test output: {}", test_dir.display());

    if dry_run {
        println!("\n{}", "Dry run mode - no files written".yellow());
    } else {
        // Create a simple test file
        fs::create_dir_all(&test_dir)?;

        let test_content = match test_framework {
            lumenx_testgen::TestFramework::Vitest => generate_vitest_sample(&project_info),
            lumenx_testgen::TestFramework::Jest => generate_jest_sample(&project_info),
            lumenx_testgen::TestFramework::RustBuiltIn => generate_rust_test_sample(&project_info),
            _ => "# Placeholder test file\n".to_string(),
        };

        let test_file = test_dir.join("sample.test.");
        let ext = match test_framework {
            lumenx_testgen::TestFramework::Vitest | lumenx_testgen::TestFramework::Jest => "ts",
            lumenx_testgen::TestFramework::RustBuiltIn => "rs",
            _ => "txt",
        };

        let test_path = format!("{}.{}", test_file.display(), ext);
        fs::write(&test_path, test_content)?;

        println!("\n{}", "Test generation complete!".green());
        println!("  Created: {}", test_path);
    }

    println!("\n{}", "Note:".yellow());
    println!("  Full test generation requires AST parsing.");
    println!("  This is a placeholder implementation.");

    Ok(())
}

/// Fix command - Apply automatic fixes
fn cmd_fix(
    path: PathBuf,
    dry_run: bool,
    interactive: bool,
    yes: bool,
    min_severity: Option<SeverityLevel>,
    categories: Option<String>,
) -> anyhow::Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }

    println!("\n{}", "Auto-Fix Engine".cyan().bold());

    if dry_run {
        println!("Mode: {}", "Dry Run".yellow());
    } else if interactive {
        println!("Mode: {}", "Interactive".cyan());
    } else if yes {
        println!("Mode: {}", "Auto-Apply".green());
    }

    // Run analysis first
    let detector = lumenx_detect::FrameworkDetector::new(&path);
    let project_info = detector.detect()?;

    let source_files = scan_source_files(&path)?;
    let test_files = scan_test_files(&path)?;
    let config_files = scan_config_files(&path)?;
    let total_lines = count_lines(&source_files)?;

    let project = lumenx_core::Project {
        info: project_info,
        source_files,
        test_files,
        config_files,
        total_lines,
        coverage: None,
    };

    let analyzer = lumenx_analyze::Analyzer::new(project);
    let analysis_result = analyzer.analyze()?;

    // Collect all fixable issues
    let mut fixable_issues: Vec<_> = analysis_result
        .all_findings()
        .filter(|i| i.suggestion.is_some())
        .cloned()
        .collect();

    // Filter by severity
    if let Some(min_sv) = min_severity {
        fixable_issues.retain(|i| SeverityLevel::from_lumen_score(i.severity) >= min_sv);
    }

    // Filter by category
    if let Some(cats) = categories {
        let cat_list: Vec<&str> = cats.split(',').map(|s| s.trim()).collect();
        fixable_issues.retain(|i| {
            cat_list.iter().any(|cat| i.category.to_lowercase().contains(cat))
        });
    }

    if fixable_issues.is_empty() {
        println!("\n{}", "No fixable issues found.".green());
        return Ok(());
    }

    println!("\nFound {} fixable issues:", fixable_issues.len());

    // Group by severity
    let critical = fixable_issues.iter().filter(|i| i.severity == lumenx_score::IssueSeverity::Critical).count();
    let high = fixable_issues.iter().filter(|i| i.severity == lumenx_score::IssueSeverity::High).count();
    let medium = fixable_issues.iter().filter(|i| i.severity == lumenx_score::IssueSeverity::Medium).count();
    let low = fixable_issues.iter().filter(|i| i.severity == lumenx_score::IssueSeverity::Low).count();

    println!("  {} Critical", critical.to_string().red().bold());
    println!("  {} High", high.to_string().red());
    println!("  {} Medium", medium.to_string().yellow());
    println!("  {} Low", low.to_string().white());

    // Apply fixes
    let fix_engine = if dry_run {
        lumenx_fix::FixEngine::new(path.clone()).with_dry_run()
    } else {
        lumenx_fix::FixEngine::new(path.clone())
    };

    if interactive && !yes {
        println!("\n{}", "Interactive Fix Mode".cyan());
        for (i, issue) in fixable_issues.iter().enumerate() {
            println!(
                "\n[{}] {} {}",
                i + 1,
                format!("{:?}", issue.severity).color(severity_color(issue.severity)),
                issue.title
            );
            if let Some(file) = &issue.file {
                println!("    Location: {}", file);
                if let Some(line) = issue.line {
                    println!("    Line: {}", line);
                }
            }
            if let Some(suggestion) = &issue.suggestion {
                println!("    Suggestion: {}", suggestion);
            }

            print!(" Apply fix? [y/N/a/q] ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_lowercase();

            if input == "q" {
                println!("Exiting...");
                break;
            } else if input == "a" {
                // Apply all remaining
                let result = fix_engine.apply_fixes(fixable_issues.clone())?;
                print_fix_result(&result, dry_run);
                break;
            } else if input == "y" {
                let result = fix_engine.apply_fixes(vec![issue.clone()])?;
                print_fix_result(&result, dry_run);
            }
        }
    } else {
        let result = fix_engine.apply_fixes(fixable_issues)?;
        print_fix_result(&result, dry_run);
    }

    Ok(())
}

/// Report command - Generate reports
fn cmd_report(
    path: PathBuf,
    format: Option<OutputFormat>,
    output: Option<PathBuf>,
    trend: bool,
    all: bool,
) -> anyhow::Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }

    println!("\n{}", "Generating Reports".cyan().bold());

    // Detect and analyze
    let detector = lumenx_detect::FrameworkDetector::new(&path);
    let project_info = detector.detect()?;

    let source_files = scan_source_files(&path)?;
    let test_files = scan_test_files(&path)?;
    let config_files = scan_config_files(&path)?;
    let total_lines = count_lines(&source_files)?;

    let project = lumenx_core::Project {
        info: project_info.clone(),
        source_files: source_files.clone(),
        test_files: test_files.clone(),
        config_files,
        total_lines,
        coverage: None,
    };

    let analyzer = lumenx_analyze::Analyzer::new(project.clone());
    let analysis_result = analyzer.analyze()?;

    let dimension_scores = calculate_dimension_scores(
        &analysis_result,
        &source_files,
        total_lines,
        &test_files,
    );

    let overall = dimension_scores.weighted_sum();
    let grade = lumenx_score::Grade::from_score(overall);

    let mut score = lumenx_score::ProjectScore {
        project_name: project_info.name.clone(),
        commit_sha: get_current_commit_sha(&path),
        timestamp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        overall,
        grade,
        dimensions: dimension_scores,
        trend: None,
        metadata: lumenx_score::ScoreMetadata::default(),
    };

    if trend {
        score.trend = load_score_history(&path)?;
    }

    let output_dir = output.unwrap_or_else(|| path.join("lumen-reports"));

    let report_gen = lumenx_report::ReportGenerator::new(
        project_info,
        score,
        output_dir.clone(),
    );

    let reports = if all {
        report_gen.generate_all()?
    } else {
        let fmt = match format.unwrap_or(OutputFormat::Markdown) {
            OutputFormat::Markdown => lumenx_report::ReportFormat::Markdown,
            OutputFormat::Json => lumenx_report::ReportFormat::Json,
            OutputFormat::Html => lumenx_report::ReportFormat::Html,
            OutputFormat::JUnit => lumenx_report::ReportFormat::JUnit,
        };
        report_gen.generate(fmt)?
    };

    println!("\n{}", "Reports generated:".green().bold());
    for report in &reports {
        println!("  - {}", report.path.display());
    }

    println!("\nOutput directory: {}", output_dir.display());

    Ok(())
}

fn cmd_watch(
    path: PathBuf,
    include: Option<String>,
    exclude: Option<String>,
    include_patterns: Option<String>,
    exclude_patterns: Option<String>,
    debounce_ms: u64,
    no_startup: bool,
) -> anyhow::Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path.display()));
    }

    println!("\n{}", "Watch Mode".cyan().bold());
    println!("Watching: {}", path.display());

    // Build watch config
    let mut config = watch::WatchConfig {
        run_on_startup: !no_startup,
        debounce_ms,
        ..Default::default()
    };

    // Parse include paths
    if let Some(include_str) = include {
        config.include_paths = include_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Parse exclude paths
    if let Some(exclude_str) = exclude {
        config.exclude_paths = exclude_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Parse include patterns
    if let Some(patterns_str) = include_patterns {
        config.include_patterns = patterns_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Parse exclude patterns
    if let Some(patterns_str) = exclude_patterns {
        config.exclude_patterns = patterns_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Create analysis callback
    let project_path = path.clone();
    let callback = move |changed_files: Vec<std::path::PathBuf>| {
        println!("\n{}", "─".repeat(60).dimmed());
        println!("{}", "Running analysis...".cyan().bold());

        // Detect project
        let detector = lumenx_detect::FrameworkDetector::new(&project_path);
        let project_info: lumenx_core::ProjectInfo = match detector.detect() {
            Ok(info) => info,
            Err(e) => {
                eprintln!("{} Failed to detect project: {}", "ERROR:".red(), e);
                return;
            }
        };

        // Scan files
        let source_files = match scan_source_files(&project_path) {
            Ok(files) => files,
            Err(e) => {
                eprintln!("{} Failed to scan source files: {}", "ERROR:".red(), e);
                return;
            }
        };

        let test_files = match scan_test_files(&project_path) {
            Ok(files) => files,
            Err(e) => {
                eprintln!("{} Failed to scan test files: {}", "ERROR:".red(), e);
                return;
            }
        };

        let config_files = match scan_config_files(&project_path) {
            Ok(files) => files,
            Err(e) => {
                eprintln!("{} Failed to scan config files: {}", "ERROR:".red(), e);
                return;
            }
        };

        let total_lines = match count_lines(&source_files) {
            Ok(lines) => lines,
            Err(e) => {
                eprintln!("{} Failed to count lines: {}", "ERROR:".red(), e);
                return;
            }
        };

        let project = lumenx_core::Project {
            info: project_info.clone(),
            source_files: source_files.clone(),
            test_files: test_files.clone(),
            config_files,
            total_lines,
            coverage: None,
        };

        // Run analysis
        let analyzer = lumenx_analyze::Analyzer::new(project.clone());
        match analyzer.analyze() {
            Ok(analysis_result) => {
                let dimension_scores = calculate_dimension_scores(
                    &analysis_result,
                    &source_files,
                    total_lines,
                    &test_files,
                );

                let overall = dimension_scores.weighted_sum();
                let grade = lumenx_score::Grade::from_score(overall);

                println!("{} Overall Score: {:.1}/100 ({})", "✓".green(), overall, grade);
            }
            Err(e) => {
                eprintln!("{} Analysis failed: {}", "ERROR:".red(), e);
            }
        }
    };

    // Create and start watcher
    let mut handler = watch::WatchHandler::new(
        path,
        config,
        callback,
    ).map_err(|e| anyhow::anyhow!("Failed to create watcher: {}", e))?;

    handler.start().map_err(|e| anyhow::anyhow!("Watch error: {}", e))
}

// ============================================================================
// Helper Functions
// ============================================================================

fn print_banner() {
    println!();
    println!(
        "{}",
        r"
 ╔══════════════════════════════════════════════════╗
 ║                                                  ║
 ║   ██╗  ██╗██╗███╗   ██╗ █████╗ ███╗   ███╗███████╗
 ║   ██║ ██╔╝██║████╗  ██║██╔══██╗████╗ ████║██╔════╝
 ║   █████╔╝ ██║██╔██╗ ██║███████║██╔████╔██║█████╗
 ║   ██╔═██╗ ██║██║╚██╗██║██╔══██║██║╚██╔╝██║██╔══╝
 ║   ██║  ██╗██║██║ ╚████║██║  ██║██║ ╚═╝ ██║███████╗
 ║   ╚═╝  ╚═╝╚═╝╚═╝  ╚═══╝╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝
 ║                                                  ║
 ║                AI Code Analysis Toolkit           ║
 ╚══════════════════════════════════════════════════╝
        "
        .bright_blue()
    );
    println!(
        "                         v{}                       ",
        VERSION.bright_white()
    );
    println!();
}

fn print_score_summary(score: &lumenx_score::ProjectScore) {
    let grade_color = match score.grade {
        lumenx_score::Grade::APlus | lumenx_score::Grade::A | lumenx_score::Grade::AMinus => {
            colored::Color::Green
        }
        lumenx_score::Grade::BPlus | lumenx_score::Grade::B | lumenx_score::Grade::BMinus => {
            colored::Color::Blue
        }
        lumenx_score::Grade::CPlus | lumenx_score::Grade::C | lumenx_score::Grade::CMinus => {
            colored::Color::Yellow
        }
        lumenx_score::Grade::DPlus | lumenx_score::Grade::D => colored::Color::Magenta,
        lumenx_score::Grade::F => colored::Color::Red,
    };

    println!(
        "\n{}",
        "╔═══════════════════════════════════════════════════════════╗".bold()
    );
    println!("{}", "║                    LUMEN SCAN REPORT                     ║".bold());
    println!(
        "{}",
        "╠═══════════════════════════════════════════════════════════╣".bold()
    );
    println!("║                                                         ║");
    println!(
        "║  Overall: {:.0}/100 ({:<20})               ║",
        score.overall,
        format!("{:?}", score.grade).color(grade_color)
    );
    println!("║                                                         ║");
    println!("║  ┌────────────────────────────────────────────────────┐   ║");
    println!("║  │  DIMENSIONS                                        │   ║");
    println!("║  │                                                     │   ║");

    for (name, dim) in score.dimensions.all() {
        let bar = "█".repeat((dim.score / 5.0) as usize);
        let bar_empty = "░".repeat(20 - bar.len());
        println!(
            "║  │  {:12} {}{}{:<3} {:.0}/100              │   ║",
            name,
            bar.color(colored::Color::Cyan),
            bar_empty,
            "",
            dim.score
        );
    }

    println!(
        "║  └────────────────────────────────────────────────────┘   ║"
    );
    println!("║                                                         ║");
    println!(
        "{}",
        "╚═══════════════════════════════════════════════════════════╝".bold()
    );
}

fn print_detailed_score(score: &lumenx_score::ProjectScore) {
    print_score_summary(score);

    println!("\n{}", "Detailed Breakdown".cyan().bold());
    println!("{}", "═".repeat(60).cyan());

    for (name, dim) in score.dimensions.all() {
        println!("\n{}", name.to_uppercase().cyan());
        println!("  Score: {:.0}/100", dim.score);
        println!("  Grade: {:?}", dim.grade);
        println!("  Weighted: {:.1} ({:.0}%)", dim.weighted, dim.weight * 100.0);

        if !dim.issues.is_empty() {
            println!("  Issues: {}", dim.issues.len());
            for issue in dim.issues.iter().take(3) {
                println!(
                    "    - {} {}",
                    format!("{:?}", issue.severity).color(severity_color(issue.severity)),
                    issue.title
                );
            }
        }

        if !dim.improvements.is_empty() {
            println!("  Top Improvements:");
            for imp in dim.improvements.iter().take(2) {
                println!(
                    "    - {} ({:?})",
                    imp.title,
                    imp.effort
                );
            }
        }
    }
}

fn print_scan_summary(
    score: &lumenx_score::ProjectScore,
    analysis: &lumenx_analyze::AnalysisResult,
    source_files: &[PathBuf],
    duration: std::time::Duration,
) {
    println!("\n{}", "Scan Summary".cyan().bold());
    println!("{}", "═".repeat(60).cyan());

    println!("  Files Scanned: {}", source_files.len());
    println!("  Lines of Code: {}", score.metadata.lines_of_code);
    println!("  Duration: {:.2}s", duration.as_secs_f64());

    println!("\n{}", "Issues Found".cyan().bold());
    println!("  Critical: {} {}", analysis.total_critical(), "●".red());
    println!("  High: {} {}", analysis.total_high(), "●".bright_red());
    println!("  Medium: {} {}", analysis.total_medium(), "●".yellow());
    println!("  Low: {} {}", analysis.total_low(), "●".white());
}

fn print_findings_table(findings: &[lumenx_score::ScoreIssue]) {
    if findings.is_empty() {
        println!("\n{}", "No issues found!".green().bold());
        return;
    }

    println!("\n{}", "Issues Found".cyan().bold());
    println!(
        "{:<6} {:<8} {:<20} {:<30} {:<20}",
        "ID", "Severity", "Category", "Title", "Location"
    );
    println!("{}", "─".repeat(100));

    for (i, finding) in findings.iter().enumerate().take(50) {
        let location = finding
            .file
            .as_ref()
            .map(|f| {
                if let Some(line) = finding.line {
                    format!("{}:{}", f, line)
                } else {
                    f.clone()
                }
            })
            .unwrap_or_else(|| "N/A".to_string());

        println!(
            "{:<6} {:<8} {:<20} {:<30} {:<20}",
            i + 1,
            format!("{:?}", finding.severity).color(severity_color(finding.severity)),
            finding.category,
            if finding.title.len() > 28 {
                format!("{}...", &finding.title[..28])
            } else {
                finding.title.clone()
            },
            if location.len() > 18 {
                format!("{}...", &location[..18])
            } else {
                location
            }
        );
    }

    if findings.len() > 50 {
        println!("\n... and {} more issues", findings.len() - 50);
    }
}

fn print_fix_result(result: &lumenx_fix::FixResult, dry_run: bool) {
    if dry_run {
        println!("\n{}", "Dry Run Results".yellow());
    } else {
        println!("\n{}", "Fix Results".cyan());
    }

    println!("  Fixed: {} {}", result.fixed.len(), "✓".green());
    println!("  Failed: {}", result.failed.len());
    println!("  Files Modified: {}", result.modified_files.len());

    if !result.fixed.is_empty() {
        println!("\n{}", "Fixed Issues:".green());
        for fix in &result.fixed {
            println!(
                "  ✓ {} {}",
                format!("{:?}", fix.issue.severity).color(severity_color(fix.issue.severity)),
                fix.issue.title
            );
        }
    }

    if !result.failed.is_empty() {
        println!("\n{}", "Failed to Fix:".red());
        for issue in &result.failed {
            println!(
                "  ✗ {} {}",
                format!("{:?}", issue.issue.severity).color(severity_color(issue.issue.severity)),
                issue.issue.title
            );
        }
    }
}

fn print_detection_row(label: &str, value: &str) {
    println!(
        "  {:<20}: {}",
        label.cyan(),
        value.white().bold()
    );
}

fn severity_color(severity: lumenx_score::IssueSeverity) -> colored::Color {
    match severity {
        lumenx_score::IssueSeverity::Info => colored::Color::Blue,
        lumenx_score::IssueSeverity::Low => colored::Color::Cyan,
        lumenx_score::IssueSeverity::Medium => colored::Color::Yellow,
        lumenx_score::IssueSeverity::High => colored::Color::Red,
        lumenx_score::IssueSeverity::Critical => colored::Color::BrightRed,
    }
}

fn get_default_config() -> String {
    r#"# Lumen Configuration

[general]
verbose = false
quiet = false
no_color = false

[scoring.weights]
coverage = 0.25
quality = 0.20
performance = 0.15
security = 0.15
seo = 0.10
docs = 0.05
uiux = 0.10

[scoring.thresholds]
excellent = 80.0
good = 60.0

[analysis]
static_analysis = true
security = true
dependencies = true
performance = true
seo = true
uiux = true

[report]
output_dir = "./lumen-reports"
formats = ["md", "json"]

[analysis.exclude]
paths = ["node_modules", "target", "dist", "build", ".git", "vendor"]
"#.to_string()
}

fn generate_vitest_sample(info: &lumenx_core::ProjectInfo) -> String {
    format!(r#"import {{ describe, it, expect }} from 'vitest';

// Auto-generated test file for {}
// Generated by Lumen v{}

describe('{}', () => {{
  it('should pass placeholder test', () => {{
    expect(true).toBe(true);
  }});

  // Add more tests based on your source code
  // Run `lumen analyze` to find uncovered code
}});
"#, info.name, VERSION, info.name)
}

fn generate_jest_sample(info: &lumenx_core::ProjectInfo) -> String {
    format!(r#"// Auto-generated test file for {}
// Generated by Lumen v{}

describe('{}', () => {{
  it('should pass placeholder test', () => {{
    expect(true).toBe(true);
  }});

  // Add more tests based on your source code
  // Run `lumen analyze` to find uncovered code
}});
"#, info.name, VERSION, info.name)
}

fn generate_rust_test_sample(info: &lumenx_core::ProjectInfo) -> String {
    format!(r"//! Auto-generated tests for {}
//! Generated by Lumen v{}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn placeholder_test() {{
        assert!(true);
    }}

    // Add more tests based on your source code
    // Run `lumen analyze` to find uncovered code
}}
", info.name, VERSION)
}

fn scan_source_files(path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Save current directory and change to target path for glob patterns
    let original_dir = std::env::current_dir()?;

    // Change to the target directory for relative glob patterns
    std::env::set_current_dir(path)?;

    let patterns = [
        "**/*.rs",
        "**/*.ts",
        "**/*.tsx",
        "**/*.js",
        "**/*.jsx",
        "**/*.py",
        "**/*.go",
    ];

    for pattern in &patterns {
        if let Ok(glob) = glob::glob(pattern) {
            for entry in glob.flatten() {
                // Skip exclusions
                let path_str = entry.to_string_lossy().to_lowercase();
                if !path_str.contains("node_modules")
                    && !path_str.contains("target")
                    && !path_str.contains(".git")
                    && !path_str.contains("dist")
                    && !path_str.contains("build")
                    && !path_str.contains(".next")
                    && !path_str.contains("turbo")
                {
                    // Convert to absolute path
                    if let Ok(abs_path) = entry.canonicalize() {
                        files.push(abs_path);
                    } else {
                        files.push(entry);
                    }
                }
            }
        }
    }

    // Restore original directory
    std::env::set_current_dir(original_dir)?;

    tracing::debug!("Found {} source files", files.len());
    Ok(files)
}

fn scan_test_files(path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path_str = entry.path().to_string_lossy().to_lowercase();
        if path_str.contains(".test.")
            || path_str.contains(".spec.")
            || path_str.contains("/tests/")
            || path_str.contains("/__tests__/")
        {
            files.push(entry.path().to_path_buf());
        }
    }

    Ok(files)
}

fn scan_config_files(path: &PathBuf) -> anyhow::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let config_names = [
        "package.json",
        "tsconfig.json",
        "Cargo.toml",
        "pyproject.toml",
        "go.mod",
        "vite.config.ts",
        "vitest.config.ts",
        "jest.config.js",
        ".eslintrc",
        ".prettierrc",
    ];

    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Some(name) = entry.file_name().to_str() {
            if config_names.contains(&name) {
                files.push(entry.path().to_path_buf());
            }
        }
    }

    Ok(files)
}

fn count_lines(files: &[PathBuf]) -> anyhow::Result<usize> {
    let mut total = 0;
    for file in files {
        if let Ok(content) = fs::read_to_string(file) {
            total += content.lines().count();
        }
    }
    Ok(total)
}

fn count_files_by_extension(path: &PathBuf) -> anyhow::Result<HashMap<String, usize>> {
    let mut counts = HashMap::new();

    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Some(ext) = entry.path().extension() {
            let ext_str = format!(".{}", ext.to_string_lossy());
            let path_str = entry.path().to_string_lossy().to_lowercase();
            if !path_str.contains("node_modules")
                && !path_str.contains("target")
                && !path_str.contains(".git")
            {
                *counts.entry(ext_str).or_insert(0) += 1;
            }
        }
    }

    Ok(counts)
}

fn calculate_language_breakdown(files: &[PathBuf]) -> HashMap<String, usize> {
    let mut breakdown = HashMap::new();

    for file in files {
        if let Some(ext) = file.extension() {
            let lang = match ext.to_string_lossy().as_ref() {
                "rs" => "Rust",
                "ts" | "tsx" => "TypeScript",
                "js" | "jsx" => "JavaScript",
                "py" => "Python",
                "go" => "Go",
                _ => "Other",
            };
            *breakdown.entry(lang.to_string()).or_insert(0) += 1;
        }
    }

    breakdown
}

fn calculate_dimension_scores(
    analysis: &lumenx_analyze::AnalysisResult,
    source_files: &[PathBuf],
    total_lines: usize,
    test_files: &[PathBuf],
) -> lumenx_score::DimensionScores {
    // Calculate coverage score
    let test_ratio = if source_files.is_empty() {
        0.0
    } else {
        (test_files.len() as f64 / source_files.len() as f64 * 100.0).min(100.0)
    };
    let coverage_score = lumenx_score::DimensionScore::new("coverage".to_string(), test_ratio, 0.25)
        .with_issues(analysis.static_findings.clone());

    // Calculate quality score (inverse of issues)
    let issue_count = analysis.static_findings.len() + analysis.dependency_findings.len();
    let quality_score = lumenx_score::DimensionScore::new(
        "quality".to_string(),
        (100.0 - (issue_count as f64 * 2.0)).max(0.0).min(100.0),
        0.20,
    )
    .with_issues(analysis.static_findings.clone());

    // Performance score
    let perf_score = lumenx_score::DimensionScore::new("performance".to_string(), 75.0, 0.15)
        .with_issues(analysis.performance_findings.clone());

    // Security score
    let sec_issues = analysis.security_findings.len();
    let security_score = lumenx_score::DimensionScore::new(
        "security".to_string(),
        (100.0 - (sec_issues as f64 * 10.0)).max(0.0).min(100.0),
        0.15,
    )
    .with_issues(analysis.security_findings.clone());

    // SEO score
    let seo_score = lumenx_score::DimensionScore::new("seo".to_string(), 70.0, 0.10)
        .with_issues(analysis.seo_findings.clone());

    // Docs score
    let docs_score = lumenx_score::DimensionScore::new("docs".to_string(), 60.0, 0.05)
        .with_issues(analysis.docs_findings.clone());

    // UI/UX score
    let uiux_score = lumenx_score::DimensionScore::new("uiux".to_string(), 72.0, 0.10)
        .with_issues(analysis.uiux_findings.clone());

    lumenx_score::DimensionScores {
        coverage: coverage_score,
        quality: quality_score,
        performance: perf_score,
        security: security_score,
        seo: seo_score,
        docs: docs_score,
        uiux: uiux_score,
    }
}

fn get_current_commit_sha(path: &PathBuf) -> String {
    // Try to get git commit SHA
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(path)
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    "unknown".to_string()
}

fn load_score_history(path: &PathBuf) -> anyhow::Result<Option<lumenx_score::TrendAnalysis>> {
    let history_path = path.join(".lumen").join("history.json");
    if history_path.exists() {
        if let Ok(content) = fs::read_to_string(&history_path) {
            if let Ok(history) = serde_json::from_str::<lumenx_score::ScoreHistory>(&content) {
                let current = history.scores.first().map(|s| s.score).unwrap_or(0.0);
                let previous = history.scores.get(1).map(|s| s.score);

                let direction = if let Some(prev) = previous {
                    if current > prev {
                        lumenx_score::TrendDirection::Improving
                    } else if current < prev {
                        lumenx_score::TrendDirection::Declining
                    } else {
                        lumenx_score::TrendDirection::Stable
                    }
                } else {
                    lumenx_score::TrendDirection::Stable
                };

                let delta = lumenx_score::ScoreDelta {
                    overall: previous.map_or(0.0, |p| current - p),
                    coverage: 0.0,
                    quality: 0.0,
                    performance: 0.0,
                    security: 0.0,
                    seo: 0.0,
                    docs: 0.0,
                    uiux: 0.0,
                };

                let scores: Vec<f64> = history.scores.iter().map(|s| s.score).collect();
                let moving_avg = lumenx_score::MovingAverage::calculate(&scores);

                return Ok(Some(lumenx_score::TrendAnalysis {
                    current,
                    previous,
                    direction,
                    delta,
                    moving_avg,
                    prediction: None,
                }));
            }
        }
    }
    Ok(None)
}

fn print_history_comparison(
    path: &PathBuf,
    current_score: &lumenx_score::ProjectScore,
) -> anyhow::Result<()> {
    if let Some(trend) = load_score_history(path)? {
        println!("\n{}", "Score Trend".cyan().bold());
        println!("  Trend: {}", match trend.direction {
            lumenx_score::TrendDirection::Improving => "↑ Improving".green(),
            lumenx_score::TrendDirection::Declining => "↓ Declining".red(),
            lumenx_score::TrendDirection::Stable => "→ Stable".white(),
        });

        if let Some(prev) = trend.previous {
            let diff = current_score.overall - prev;
            let diff_str = if diff > 0.0 {
                format!("+{:.1}", diff).green()
            } else if diff < 0.0 {
                format!("{:.1}", diff).red()
            } else {
                "0.0".white()
            };
            println!("  Change: {} (from {:.1})", diff_str, prev);
        }
    } else {
        println!("\n{}", "No historical data available. Run 'lumen scan' again to build history.".dimmed());
    }

    Ok(())
}
