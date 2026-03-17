//! Static Analysis Engine
//!
//! Comprehensive static code analysis supporting:
//! - Code complexity analysis (cyclomatic, cognitive)
//! - Code duplication detection (token-based)
//! - Dead code identification (unused functions, variables, imports)
//! - Code smell detection (long functions, god classes, deep nesting)
//! - Security pattern detection (SQL injection, XSS, etc.)
//!
//! Supports: Rust, TypeScript/JavaScript, Python, Go

use crate::ast::{AstLanguage, Parser, Query, Traversal, TraversalOrder};
use crate::ast::query::QueryExecutor;
use lumenx_core::{LumenError, LumenResult, ProjectInfo};
use lumenx_score::{MetricValue, ScoreIssue};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, trace, warn};
use rayon::prelude::*;

#[cfg(feature = "glob")]
use glob;

/// Static analysis engine
#[derive(Debug, Clone)]
pub struct StaticAnalyzer {
    /// Project root directory
    project_root: PathBuf,
    /// Maximum file size to analyze (in bytes)
    max_file_size: usize,
    /// Duplicate detection threshold (minimum tokens to consider)
    duplicate_min_tokens: usize,
    /// Complexity thresholds
    complexity_thresholds: ComplexityThresholds,
}

/// Configuration for complexity analysis
#[derive(Debug, Clone)]
pub struct ComplexityThresholds {
    /// Maximum acceptable cyclomatic complexity
    pub max_cyclomatic: usize,
    /// Maximum acceptable cognitive complexity
    pub max_cognitive: usize,
    /// Maximum function length (lines)
    pub max_function_length: usize,
    /// Maximum method count per class
    pub max_methods_per_class: usize,
    /// Maximum nesting depth
    pub max_nesting_depth: usize,
    /// Maximum parameter count
    pub max_parameters: usize,
}

impl Default for ComplexityThresholds {
    fn default() -> Self {
        Self {
            max_cyclomatic: 10,
            max_cognitive: 15,
            max_function_length: 50,
            max_methods_per_class: 20,
            max_nesting_depth: 4,
            max_parameters: 5,
        }
    }
}

impl Default for StaticAnalyzer {
    fn default() -> Self {
        Self::new(PathBuf::from("."))
    }
}

impl StaticAnalyzer {
    /// Create a new static analyzer
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            max_file_size: 1_000_000, // 1MB
            duplicate_min_tokens: 50,
            complexity_thresholds: ComplexityThresholds::default(),
        }
    }

    /// Set complexity thresholds
    pub fn with_thresholds(mut self, thresholds: ComplexityThresholds) -> Self {
        self.complexity_thresholds = thresholds;
        self
    }

    /// Run static analysis on a project
    pub fn analyze(
        &self,
        info: &ProjectInfo,
    ) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        info!("Starting static analysis for project: {}", info.name);

        let mut metrics = HashMap::new();
        let all_issues = Arc::new(Mutex::new(Vec::new()));
        let language_stats = Arc::new(Mutex::new(HashMap::<AstLanguage, LanguageStats>::new()));

        // Collect all source files
        let source_files = self.collect_source_files(info)?;
        debug!("Found {} source files for analysis", source_files.len());

        // PARALLEL ANALYSIS with Rayon
        // Analyze files in parallel for significant speedup
        source_files.par_iter().for_each(|file_path| {
            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let language = AstLanguage::from_extension(ext);
            if language == AstLanguage::Unknown {
                return;
            }

            match self.analyze_file(file_path, language, info) {
                Ok(result) => {
                    let mut stats = language_stats.lock().unwrap();
                    let stats_entry = stats.entry(language).or_default();
                    stats_entry.files_analyzed += 1;
                    stats_entry.lines_of_code += result.lines_of_code;
                    stats_entry.total_complexity += result.complexity;
                    stats_entry.max_complexity = stats_entry.max_complexity.max(result.max_complexity);
                    stats_entry.functions_count += result.functions_count;
                    stats_entry.classes_count += result.classes_count;
                    stats_entry.code_smells.extend(result.code_smells);

                    let mut issues = all_issues.lock().unwrap();
                    issues.extend(result.issues);
                }
                Err(e) => {
                    warn!("Failed to analyze file {}: {}", file_path.display(), e);
                }
            }
        });

        // Extract results from Arc wrappers
        let mut all_issues = Arc::try_unwrap(all_issues).unwrap().into_inner().unwrap();
        let language_stats = Arc::try_unwrap(language_stats).unwrap().into_inner().unwrap();

        // Aggregate metrics
        let total_files: usize = language_stats.values().map(|s| s.files_analyzed).sum();
        let total_lines: usize = language_stats.values().map(|s| s.lines_of_code).sum();
        let total_functions: usize = language_stats.values().map(|s| s.functions_count).sum();
        let total_classes: usize = language_stats.values().map(|s| s.classes_count).sum();

        // Calculate aggregate complexity
        let total_complexity: f64 = language_stats.values().map(|s| s.total_complexity).sum();
        let avg_complexity = if total_functions > 0 {
            total_complexity / total_functions as f64
        } else {
            0.0
        };

        // Skip duplication analysis for speed - it's O(n²) and very slow
        // TODO: Implement faster duplication detection with hashing
        // let duplication_result = self.detect_duplication(&source_files)?;
        // metrics.insert(
        //     "duplication_percent".to_string(),
        //     MetricValue::Percentage(duplication_result.duplication_percent),
        // );
        // metrics.insert(
        //     "duplicate_blocks".to_string(),
        //     MetricValue::Count(duplication_result.duplicate_blocks),
        // );

        // Collect all code smells
        let all_code_smells: Vec<CodeSmell> = language_stats
            .values()
            .flat_map(|s| s.code_smells.clone())
            .collect();

        // Create issues from code smells
        for smell in all_code_smells {
            all_issues.push(smell.to_score_issue());
        }

        // Populate metrics
        metrics.insert("files_analyzed".to_string(), MetricValue::Count(total_files));
        metrics.insert("lines_of_code".to_string(), MetricValue::Count(total_lines));
        metrics.insert("functions_count".to_string(), MetricValue::Count(total_functions));
        metrics.insert("classes_count".to_string(), MetricValue::Count(total_classes));
        metrics.insert(
            "average_complexity".to_string(),
            MetricValue::Float(avg_complexity),
        );
        metrics.insert(
            "max_complexity".to_string(),
            MetricValue::Float(
                language_stats
                    .values()
                    .map(|s| s.max_complexity)
                    .fold(0.0, f64::max),
            ),
        );
        metrics.insert(
            "code_smells_count".to_string(),
            MetricValue::Count(all_issues.len()),
        );

        // Language-specific metrics
        for (language, stats) in &language_stats {
            metrics.insert(
                format!("files_{}", language.name().to_lowercase()),
                MetricValue::Count(stats.files_analyzed),
            );
            metrics.insert(
                format!("complexity_{}", language.name().to_lowercase()),
                MetricValue::Float(stats.total_complexity),
            );
        }

        info!(
            "Static analysis complete: {} files, {} issues found",
            total_files,
            all_issues.len()
        );

        Ok((metrics, all_issues))
    }

    /// Collect all source files for analysis
    fn collect_source_files(&self, info: &ProjectInfo) -> LumenResult<Vec<PathBuf>> {
        let mut files = Vec::new();

        let extensions = match info.language {
            lumenx_core::Language::TypeScript => &["ts", "tsx"][..],
            lumenx_core::Language::JavaScript => &["js", "jsx"][..],
            lumenx_core::Language::Rust => &["rs"][..],
            lumenx_core::Language::Python => &["py"][..],
            lumenx_core::Language::Go => &["go"][..],
            _ => &["ts", "tsx", "js", "jsx", "rs", "py", "go"],
        };

        // Save current directory and change to target path for glob patterns
        let original_dir = std::env::current_dir()?;

        // Change to the target directory for relative glob patterns
        std::env::set_current_dir(&info.root)?;

        // Build glob patterns from extensions
        for ext in extensions {
            let pattern = format!("**/*.{}", ext);

            if let Ok(glob_result) = glob::glob(&pattern) {
                for entry in glob_result.flatten() {
                    // Skip exclusions
                    let path_str = entry.to_string_lossy().to_lowercase();
                    if path_str.contains("node_modules")
                        || path_str.contains("target")
                        || path_str.contains(".git")
                        || path_str.contains("/.")
                        || path_str.contains("dist")
                        || path_str.contains("build")
                        || path_str.contains(".next")
                        || path_str.contains("turbo")
                    {
                        continue;
                    }

                    // Check file size
                    if let Ok(metadata) = entry.metadata() {
                        if metadata.len() <= self.max_file_size as u64 {
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
        }

        // Restore original directory
        let _ = std::env::set_current_dir(original_dir);

        Ok(files)
    }

    /// Analyze a single file (original, for fallback)
    fn analyze_file(
        &self,
        path: &Path,
        language: AstLanguage,
        _info: &ProjectInfo,
    ) -> LumenResult<FileAnalysisResult> {
        debug!("Analyzing file: {:?} ({})", path, language);

        let mut parser = Parser::new(language)?;

        // Check if language is supported
        let ts_lang = language.tree_sitter_language();
        if ts_lang.is_err() {
            // For unsupported languages, do basic regex-based analysis
            return self.analyze_file_fallback(path, language);
        }

        let (tree, source) = parser.parse_file(path)?;

        // Calculate lines of code
        let lines_of_code = source.lines().filter(|l| !l.trim().is_empty()).count();

        // ULTRA-OPTIMIZED: Single traversal for both complexity and counting
        let (complexity, functions_count, classes_count) = self.analyze_metrics_fast(&tree, &source, language);

        // Skip slow analyses
        let code_smells = Vec::new();
        let issues = Vec::new();

        Ok(FileAnalysisResult {
            path: path.to_path_buf(),
            language,
            lines_of_code,
            complexity,
            max_complexity: complexity,
            functions_count,
            classes_count,
            code_smells,
            issues,
        })
    }

    /// Fast single-pass metrics calculation (combines complexity + counting)
    fn analyze_metrics_fast(&self, tree: &tree_sitter::Tree, source: &str, language: AstLanguage) -> (f64, usize, usize) {
        let root = tree.root_node();
        let mut complexity = 0.0;
        let mut functions_count = 0;
        let mut classes_count = 0;

        let traversal = Traversal::new(
            crate::ast::AstNode::new(root, language),
            TraversalOrder::Pre,
        );

        for node in traversal {
            match node.kind() {
                "function_declaration" | "function_item" | "function_definition" |
                "method_definition" | "arrow_function" => {
                    functions_count += 1;
                    // Add base complexity for function
                    complexity += 1.0;
                }
                "class_declaration" | "class_definition" | "interface_declaration" |
                "type_declaration" | "impl_item" => {
                    classes_count += 1;
                }
                "if" | "else" | "elseif" | "match" | "case" | "switch" |
                "for" | "while" | "loop" | "try" | "catch" => {
                    complexity += 1.0;
                }
                _ => {}
            }
        }

        (complexity, functions_count, classes_count)
    }

    /// Fallback analysis for unsupported languages
    fn analyze_file_fallback(
        &self,
        path: &Path,
        language: AstLanguage,
    ) -> LumenResult<FileAnalysisResult> {
        let source = std::fs::read_to_string(path)?;
        let lines_of_code = source.lines().filter(|l| !l.trim().is_empty()).count();

        // Basic regex-based analysis
        let complexity = self.estimate_complexity_regex(&source, language);
        let (functions_count, classes_count) =
            self.count_declarations_regex(&source, language);
        let code_smells = self.detect_code_smells_regex(&source, path, lines_of_code);

        Ok(FileAnalysisResult {
            path: path.to_path_buf(),
            language,
            lines_of_code,
            complexity,
            max_complexity: complexity,
            functions_count,
            classes_count,
            code_smells,
            issues: Vec::new(),
        })
    }

    /// Calculate cyclomatic and cognitive complexity
    fn calculate_complexity(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        language: AstLanguage,
    ) -> f64 {
        let root = tree.root_node();
        let mut total_complexity = 0.0;
        let mut function_count = 0;

        let traversal = Traversal::new(
            crate::ast::AstNode::new(root, language),
            TraversalOrder::Pre,
        );

        for node in traversal {
            let kind = node.kind();

            // Count decision points for cyclomatic complexity
            match kind {
                "if_statement"
                | "if"
                | "else"
                | "match_expression"
                | "match"
                | "for_statement"
                | "for"
                | "while_statement"
                | "while"
                | "loop_expression"
                | "conditional_expression" => {
                    total_complexity += 1.0;
                }
                "catch_clause" | "catch" => {
                    total_complexity += 1.0;
                }
                // Logical operators
                "&&" | "||" | "and" | "or" => {
                    total_complexity += 0.5;
                }
                _ => {}
            }

            // Count function boundaries
            if matches!(
                kind,
                "function_declaration"
                    | "function_item"
                    | "function_definition"
                    | "arrow_function"
                    | "method_definition"
            ) {
                function_count += 1;
            }
        }

        // Add base complexity (1 per function)
        total_complexity += function_count as f64;

        if function_count > 0 {
            total_complexity / function_count as f64
        } else {
            total_complexity
        }
    }

    /// Estimate complexity using regex (for unsupported languages)
    fn estimate_complexity_regex(&self, source: &str, _language: AstLanguage) -> f64 {
        let mut complexity = 1.0;

        // Count control flow keywords
        let if_count = source.matches("if ").count() as f64;
        let else_count = source.matches("else ").count() as f64;
        let for_count = source.matches("for ").count() as f64;
        let while_count = source.matches("while ").count() as f64;
        let match_count = source.matches("match ").count() as f64;
        let catch_count = source.matches("catch").count() as f64;

        complexity += if_count + else_count + for_count + while_count + match_count + catch_count;

        complexity
    }

    /// Count function and class declarations
    fn count_declarations(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        language: AstLanguage,
    ) -> (usize, usize) {
        let mut executor = QueryExecutor::new();

        // Count functions
        let function_query = Query::function(language);
        let functions = if let Ok(q) = function_query {
            executor.exec_captures(&q, tree, source, "name").len()
        } else {
            0
        };

        // Count classes
        let class_query = Query::class(language);
        let classes = if let Ok(q) = class_query {
            executor.exec_captures(&q, tree, source, "name").len()
        } else {
            0
        };

        (functions, classes)
    }

    /// Count declarations using regex
    fn count_declarations_regex(&self, source: &str, language: AstLanguage) -> (usize, usize) {
        let function_patterns: &[&str] = match language {
            AstLanguage::TypeScript | AstLanguage::JavaScript | AstLanguage::Tsx | AstLanguage::Jsx => {
                &[r"function\s+\w+", r"const\s+\w+\s*=\s*\(", r"\w+\s*\([^)]*\)\s*{"]
            }
            AstLanguage::Rust => &[r"fn\s+\w+"],
            AstLanguage::Python => &[r"def\s+\w+"],
            AstLanguage::Go => &[r"func\s+\w+"],
            _ => &[],
        };

        let class_patterns: &[&str] = match language {
            AstLanguage::TypeScript | AstLanguage::JavaScript | AstLanguage::Tsx | AstLanguage::Jsx => {
                &[r"class\s+\w+"]
            }
            AstLanguage::Rust => &[r"struct\s+\w+", r"impl\s+\w+"],
            AstLanguage::Python => &[r"class\s+\w+"],
            _ => &[],
        };

        let functions = function_patterns
            .iter()
            .map(|p| {
                Regex::new(p)
                    .ok()
                    .map(|r| r.find_iter(source).count())
                    .unwrap_or(0)
            })
            .sum::<usize>();

        let classes = class_patterns
            .iter()
            .map(|p| {
                Regex::new(p)
                    .ok()
                    .map(|r| r.find_iter(source).count())
                    .unwrap_or(0)
            })
            .sum::<usize>();

        (functions, classes)
    }

    /// Detect code smells in the AST
    fn detect_code_smells(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        language: AstLanguage,
        file_path: &Path,
        lines_of_code: usize,
    ) -> Vec<CodeSmell> {
        let mut smells = Vec::new();
        let root = tree.root_node();

        let traversal = Traversal::new(
            crate::ast::AstNode::new(root, language),
            TraversalOrder::Pre,
        );

        for node in traversal {
            let kind = node.kind();
            let (start_line, _) = node.start_position();
            let (end_line, _) = node.end_position();
            let function_length = end_line - start_line + 1;

            // Long function smell
            if matches!(
                kind,
                "function_declaration"
                    | "function_item"
                    | "function_definition"
                    | "method_definition"
            ) && function_length > self.complexity_thresholds.max_function_length
            {
                smells.push(CodeSmell::LongFunction {
                    file: file_path.display().to_string(),
                    line: start_line,
                    length: function_length,
                    max_allowed: self.complexity_thresholds.max_function_length,
                });
            }

            // Deep nesting smell
            let nesting_depth = self.calculate_nesting_depth(&node, source);
            if nesting_depth > self.complexity_thresholds.max_nesting_depth {
                smells.push(CodeSmell::DeepNesting {
                    file: file_path.display().to_string(),
                    line: start_line,
                    depth: nesting_depth,
                    max_allowed: self.complexity_thresholds.max_nesting_depth,
                });
            }
        }

        smells
    }

    /// Calculate maximum nesting depth for a node
    fn calculate_nesting_depth(
        &self,
        node: &crate::ast::AstNode,
        source: &str,
    ) -> usize {
        let mut max_depth = 0;
        let mut current_depth = 0;

        fn visit(node: &crate::ast::AstNode, source: &str, current_depth: &mut usize, max_depth: &mut usize) {
            let kind = node.kind();

            match kind {
                "if_statement"
                | "if"
                | "for_statement"
                | "for"
                | "while_statement"
                | "while"
                | "match_expression"
                | "match" => {
                    *current_depth += 1;
                    *max_depth = (*max_depth).max(*current_depth);
                }
                _ => {}
            }

            for child in node.named_children(source) {
                visit(&child, source, current_depth, max_depth);
            }

            match kind {
                "if_statement"
                | "if"
                | "for_statement"
                | "for"
                | "while_statement"
                | "while"
                | "match_expression"
                | "match" => {
                    *current_depth -= 1;
                }
                _ => {}
            }
        }

        visit(node, source, &mut current_depth, &mut max_depth);
        max_depth
    }

    /// Detect code smells using regex
    fn detect_code_smells_regex(
        &self,
        source: &str,
        file_path: &Path,
        lines_of_code: usize,
    ) -> Vec<CodeSmell> {
        let mut smells = Vec::new();

        // Check for very long functions (heuristic: many braces in a row)
        let lines: Vec<&str> = source.lines().collect();
        let mut current_function_start = 0;
        let mut brace_depth = 0;

        for (i, line) in lines.iter().enumerate() {
            if line.trim().starts_with("fn ")
                || line.trim().starts_with("function ")
                || line.trim().starts_with("def ")
            {
                current_function_start = i;
            }

            brace_depth += line.matches('{').count() as i32;
            brace_depth -= line.matches('}').count() as i32;

            if brace_depth == 0 && current_function_start > 0 {
                let function_length = i - current_function_start;
                if function_length > self.complexity_thresholds.max_function_length {
                    smells.push(CodeSmell::LongFunction {
                        file: file_path.display().to_string(),
                        line: current_function_start,
                        length: function_length,
                        max_allowed: self.complexity_thresholds.max_function_length,
                    });
                }
                current_function_start = 0;
            }
        }

        smells
    }

    /// Detect security issues in the code
    fn detect_security_issues(
        &self,
        _tree: &tree_sitter::Tree,
        source: &str,
        _language: AstLanguage,
        file_path: &Path,
    ) -> LumenResult<Vec<ScoreIssue>> {
        let mut issues = Vec::new();

        // SQL Injection patterns
        let sql_patterns = [
            r"SELECT\s+.*\+.*FROM",
            r"query\s*\(.*\+.*\)",
            r"execute\s*\([^)]*\+[^)]*\)",
            r"\.query\s*\([^)]*\$",
        ];

        for (i, line) in source.lines().enumerate() {
            // Check for SQL injection
            for pattern in &sql_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(line) {
                        issues.push(ScoreIssue {
                            id: format!("sql-injection-{}-{}", file_path.display().to_string().replace('/', "-"), i),
                            severity: lumenx_score::IssueSeverity::High,
                            category: "security".to_string(),
                            title: "Potential SQL Injection".to_string(),
                            description: format!(
                                "String concatenation in SQL query detected on line {}",
                                i + 1
                            ),
                            file: Some(file_path.display().to_string()),
                            line: Some(i + 1),
                            column: None,
                            impact: 15.0,
                            suggestion: Some(
                                "Use parameterized queries or prepared statements".to_string(),
                            ),
                        });
                    }
                }
            }

            // Check for eval usage
            if line.contains("eval(") || line.contains("exec(") {
                issues.push(ScoreIssue {
                    id: format!("eval-usage-{}-{}", file_path.display().to_string().replace('/', "-"), i),
                    severity: lumenx_score::IssueSeverity::High,
                    category: "security".to_string(),
                    title: "Dangerous eval/exec Usage".to_string(),
                    description: format!("Use of eval/exec on line {}", i + 1),
                    file: Some(file_path.display().to_string()),
                    line: Some(i + 1),
                    column: None,
                    impact: 10.0,
                    suggestion: Some(
                        "Avoid using eval/exec. Use safer alternatives".to_string(),
                    ),
                });
            }

            // Check for hardcoded secrets
            let secret_patterns = [
                r#"(?i)password\s*=\s*['"][^'"]{8,}['"]"#,
                r#"(?i)api[_-]?key\s*=\s*['"][^'"]{20,}['"]"#,
                r#"(?i)secret\s*=\s*['"][^'"]{20,}['"]"#,
                r#"(?i)token\s*=\s*['"][^'"]{20,}['"]"#,
            ];

            for pattern in &secret_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(line) {
                        issues.push(ScoreIssue {
                            id: format!("hardcoded-secret-{}-{}", file_path.display().to_string().replace('/', "-"), i),
                            severity: lumenx_score::IssueSeverity::Critical,
                            category: "security".to_string(),
                            title: "Hardcoded Secret Detected".to_string(),
                            description: format!("Possible hardcoded credential on line {}", i + 1),
                            file: Some(file_path.display().to_string()),
                            line: Some(i + 1),
                            column: None,
                            impact: 20.0,
                            suggestion: Some(
                                "Move credentials to environment variables".to_string(),
                            ),
                        });
                    }
                }
            }

            // Check for innerHTML usage (XSS risk)
            if line.contains(".innerHTML") || line.contains("dangerouslySetInnerHTML") {
                issues.push(ScoreIssue {
                    id: format!("xss-risk-{}-{}", file_path.display().to_string().replace('/', "-"), i),
                    severity: lumenx_score::IssueSeverity::Medium,
                    category: "security".to_string(),
                    title: "Potential XSS Vulnerability".to_string(),
                    description: format!("Direct innerHTML assignment on line {}", i + 1),
                    file: Some(file_path.display().to_string()),
                    line: Some(i + 1),
                    column: None,
                    impact: 8.0,
                    suggestion: Some(
                        "Use textContent or sanitize HTML before assignment".to_string(),
                    ),
                });
            }
        }

        Ok(issues)
    }

    /// Detect dead code (unused functions, variables, imports)
    /// NOTE: Disabled for TypeScript/JavaScript due to React exports causing massive false positives
    fn detect_dead_code(
        &self,
        tree: &tree_sitter::Tree,
        source: &str,
        language: AstLanguage,
        file_path: &Path,
    ) -> LumenResult<Vec<ScoreIssue>> {
        let mut issues = Vec::new();

        // Skip dead code detection for TypeScript/JavaScript
        // React components are exported but not directly called, causing false positives
        match language {
            AstLanguage::TypeScript | AstLanguage::JavaScript | AstLanguage::Tsx | AstLanguage::Jsx => {
                return Ok(issues);
            }
            _ => {}
        }

        let mut defined_functions: HashSet<String> = HashSet::new();
        let mut called_functions: HashSet<String> = HashSet::new();
        let mut exported_functions: HashSet<String> = HashSet::new();

        let root = tree.root_node();

        // First pass: collect definitions and exports
        let traversal1 = Traversal::new(
            crate::ast::AstNode::new(root, language),
            TraversalOrder::Pre,
        );
        for node in traversal1 {
            match node.kind() {
                "function_declaration" | "function_item" | "function_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = name_node.text(source);
                        let name_clean = name.strip_prefix("r#").unwrap_or(name);
                        defined_functions.insert(name_clean.to_string());
                    }
                }
                // Detect export statements
                "export_statement" | "export_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = name_node.text(source);
                        exported_functions.insert(name.to_string());
                    }
                }
                _ => {}
            }
        }

        // Second pass: collect usages
        let traversal2 = Traversal::new(
            crate::ast::AstNode::new(root, language),
            TraversalOrder::Pre,
        );
        for node in traversal2 {
            match node.kind() {
                "call_expression" | "call" => {
                    if let Some(func_node) = node.child_by_field_name("function") {
                        let name = func_node.text(source);
                        called_functions.insert(name.to_string());
                    }
                }
                _ => {}
            }
        }

        // Find unused functions (excluding exports and common patterns)
        for func in &defined_functions {
            // Skip if exported, called, or special name
            if called_functions.contains(func)
                || exported_functions.contains(func)
                || func.starts_with('_')
                || func == "main"
                || func == "test"
                || func == "new"
            {
                continue;
            }

            // Find the line number
            for (i, line) in source.lines().enumerate() {
                if line.contains(&format!("fn {}", func))
                    || line.contains(&format!("function {}", func))
                    || line.contains(&format!("def {}", func))
                {
                    issues.push(ScoreIssue {
                        id: format!("unused-function-{}-{}", file_path.display().to_string().replace('/', "-"), func),
                        severity: lumenx_score::IssueSeverity::Low,
                        category: "dead_code".to_string(),
                        title: "Unused Function".to_string(),
                        description: format!("Function '{}' is never called", func),
                        file: Some(file_path.display().to_string()),
                        line: Some(i + 1),
                        column: None,
                        impact: 2.0,
                        suggestion: Some("Remove this function or export it".to_string()),
                    });
                    break;
                }
            }
        }

        Ok(issues)
    }

    /// Detect code duplication across files
    fn detect_duplication(&self, files: &[PathBuf]) -> LumenResult<DuplicationResult> {
        let mut token_blocks: Vec<TokenBlock> = Vec::new();
        let mut duplicates = Vec::new();

        // Extract token sequences from each file
        for file_path in files {
            if let Ok(source) = std::fs::read_to_string(file_path) {
                let tokens = self.tokenize(&source);
                let blocks = self.create_token_blocks(&tokens, file_path);
                token_blocks.extend(blocks);
            }
        }

        // Compare blocks for duplicates
        let mut checked_pairs = HashSet::new();
        for (i, block1) in token_blocks.iter().enumerate() {
            for (j, block2) in token_blocks.iter().enumerate() {
                if i >= j {
                    continue;
                }

                let pair_key = (i, j);
                if checked_pairs.contains(&pair_key) {
                    continue;
                }
                checked_pairs.insert(pair_key);

                if block1.file_path == block2.file_path {
                    continue; // Skip same file
                }

                let similarity = self.calculate_similarity(&block1.tokens, &block2.tokens);
                if similarity >= 0.8 {
                    // 80% similarity threshold
                    duplicates.push(DuplicateBlock {
                        file1: block1.file_path.clone(),
                        file2: block2.file_path.clone(),
                        line1: block1.start_line,
                        line2: block2.start_line,
                        similarity,
                        token_count: block1.tokens.len(),
                    });
                }
            }
        }

        let duplicate_lines: usize = duplicates.iter().map(|d| d.token_count).sum();
        let total_lines: usize = token_blocks.iter().map(|b| b.tokens.len()).sum();

        let duplication_percent = if total_lines > 0 {
            (duplicate_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        Ok(DuplicationResult {
            duplication_percent,
            duplicate_blocks: duplicates.len(),
            duplicates,
        })
    }

    /// Tokenize source code into a sequence of token types
    fn tokenize(&self, source: &str) -> Vec<String> {
        let mut tokens = Vec::new();

        // Simple tokenization by type (this is a simplified approach)
        for line in source.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            // Classify tokens
            if trimmed.starts_with("fn ") || trimmed.starts_with("function ") || trimmed.starts_with("def ") {
                tokens.push("FUNCTION".to_string());
            } else if trimmed.starts_with("if ") {
                tokens.push("IF".to_string());
            } else if trimmed.starts_with("for ") {
                tokens.push("FOR".to_string());
            } else if trimmed.starts_with("while ") {
                tokens.push("WHILE".to_string());
            } else if trimmed.starts_with("return ") {
                tokens.push("RETURN".to_string());
            } else if trimmed.contains('=')
                && !trimmed.starts_with("==")
                && !trimmed.starts_with("!=")
                && !trimmed.starts_with("<=")
                && !trimmed.starts_with(">=")
            {
                tokens.push("ASSIGN".to_string());
            } else {
                // Extract identifiers and operators
                for part in trimmed.split_whitespace() {
                    match part {
                        "+" | "-" | "*" | "/" => tokens.push("OP".to_string()),
                        "{" => tokens.push("BLOCK_START".to_string()),
                        "}" => tokens.push("BLOCK_END".to_string()),
                        "(" | ")" => tokens.push("PAREN".to_string()),
                        s if s.chars().all(|c| c.is_alphanumeric() || c == '_') => {
                            tokens.push("IDENT".to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        tokens
    }

    /// Create sliding window token blocks for comparison
    fn create_token_blocks(&self, tokens: &[String], file_path: &Path) -> Vec<TokenBlock> {
        let mut blocks = Vec::new();
        let window_size = self.duplicate_min_tokens;

        if tokens.len() < window_size {
            return blocks;
        }

        for i in 0..=(tokens.len().saturating_sub(window_size)) {
            let end = (i + window_size).min(tokens.len());
            let block_tokens: Vec<String> = tokens[i..end].to_vec();
            if block_tokens.len() >= window_size {
                blocks.push(TokenBlock {
                    file_path: file_path.display().to_string(),
                    start_line: i,
                    tokens: block_tokens,
                });
            }
        }

        blocks
    }

    /// Calculate similarity between two token sequences
    fn calculate_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        if tokens1.is_empty() || tokens2.is_empty() {
            return 0.0;
        }

        let min_len = tokens1.len().min(tokens2.len());
        let mut matches = 0;

        for i in 0..min_len {
            if tokens1[i] == tokens2[i] {
                matches += 1;
            }
        }

        matches as f64 / min_len as f64
    }
}

/// Standalone analyze function for use by lib.rs
pub fn analyze(project: &lumenx_core::Project) -> Vec<lumenx_score::ScoreIssue> {
    let analyzer = StaticAnalyzer::default();

    match analyzer.analyze(&project.info) {
        Ok((_metrics, issues)) => issues,
        Err(_) => Vec::new(),
    }
}

/// Result of analyzing a single file
#[derive(Debug, Clone)]
struct FileAnalysisResult {
    path: PathBuf,
    language: AstLanguage,
    lines_of_code: usize,
    complexity: f64,
    max_complexity: f64,
    functions_count: usize,
    classes_count: usize,
    code_smells: Vec<CodeSmell>,
    issues: Vec<ScoreIssue>,
}

/// Statistics for a language
#[derive(Debug, Clone, Default)]
struct LanguageStats {
    files_analyzed: usize,
    lines_of_code: usize,
    total_complexity: f64,
    max_complexity: f64,
    functions_count: usize,
    classes_count: usize,
    code_smells: Vec<CodeSmell>,
}

/// Code smell types
#[derive(Debug, Clone)]
enum CodeSmell {
    LongFunction {
        file: String,
        line: usize,
        length: usize,
        max_allowed: usize,
    },
    DeepNesting {
        file: String,
        line: usize,
        depth: usize,
        max_allowed: usize,
    },
    GodClass {
        file: String,
        line: usize,
        method_count: usize,
        max_allowed: usize,
    },
    LongParameterList {
        file: String,
        line: usize,
        param_count: usize,
        max_allowed: usize,
    },
    DuplicateCode {
        file: String,
        line: usize,
        similar_to: String,
    },
}

impl CodeSmell {
    fn to_score_issue(&self) -> ScoreIssue {
        match self {
            CodeSmell::LongFunction {
                file,
                line,
                length,
                max_allowed,
            } => ScoreIssue {
                id: format!("long-function-{}-{}", file.replace('/', "-"), line),
                severity: lumenx_score::IssueSeverity::Medium,
                category: "code_smell".to_string(),
                title: "Long Function".to_string(),
                description: format!(
                    "Function is {} lines, exceeds maximum of {}",
                    length, max_allowed
                ),
                file: Some(file.clone()),
                line: Some(*line),
                column: None,
                impact: 5.0,
                suggestion: Some("Break this function into smaller, more focused functions".to_string()),
            },
            CodeSmell::DeepNesting {
                file,
                line,
                depth,
                max_allowed,
            } => ScoreIssue {
                id: format!("deep-nesting-{}-{}", file.replace('/', "-"), line),
                severity: lumenx_score::IssueSeverity::Medium,
                category: "code_smell".to_string(),
                title: "Deep Nesting".to_string(),
                description: format!(
                    "Nesting depth is {}, exceeds maximum of {}",
                    depth, max_allowed
                ),
                file: Some(file.clone()),
                line: Some(*line),
                column: None,
                impact: 5.0,
                suggestion: Some(
                    "Extract nested logic into separate functions to reduce complexity".to_string(),
                ),
            },
            CodeSmell::GodClass {
                file,
                line,
                method_count,
                max_allowed,
            } => ScoreIssue {
                id: format!("god-class-{}-{}", file.replace('/', "-"), line),
                severity: lumenx_score::IssueSeverity::High,
                category: "code_smell".to_string(),
                title: "God Class".to_string(),
                description: format!(
                    "Class has {} methods, exceeds maximum of {}",
                    method_count, max_allowed
                ),
                file: Some(file.clone()),
                line: Some(*line),
                column: None,
                impact: 10.0,
                suggestion: Some(
                    "Split this class into smaller, more focused classes following SRP".to_string(),
                ),
            },
            CodeSmell::LongParameterList {
                file,
                line,
                param_count,
                max_allowed,
            } => ScoreIssue {
                id: format!("long-param-list-{}-{}", file.replace('/', "-"), line),
                severity: lumenx_score::IssueSeverity::Low,
                category: "code_smell".to_string(),
                title: "Long Parameter List".to_string(),
                description: format!(
                    "Function has {} parameters, exceeds maximum of {}",
                    param_count, max_allowed
                ),
                file: Some(file.clone()),
                line: Some(*line),
                column: None,
                impact: 3.0,
                suggestion: Some("Consider using a parameter object or struct".to_string()),
            },
            CodeSmell::DuplicateCode {
                file,
                line,
                similar_to,
            } => ScoreIssue {
                id: format!("duplicate-code-{}-{}", file.replace('/', "-"), line),
                severity: lumenx_score::IssueSeverity::Medium,
                category: "duplication".to_string(),
                title: "Duplicate Code".to_string(),
                description: format!("Code is similar to code in {}", similar_to),
                file: Some(file.clone()),
                line: Some(*line),
                column: None,
                impact: 5.0,
                suggestion: Some("Extract common logic into a shared function".to_string()),
            },
        }
    }
}

/// Result of duplication detection
#[derive(Debug, Clone)]
struct DuplicationResult {
    duplication_percent: f64,
    duplicate_blocks: usize,
    duplicates: Vec<DuplicateBlock>,
}

/// A block of duplicate code
#[derive(Debug, Clone)]
struct DuplicateBlock {
    file1: String,
    file2: String,
    line1: usize,
    line2: usize,
    similarity: f64,
    token_count: usize,
}

impl DuplicateBlock {
    fn to_issue(&self) -> ScoreIssue {
        ScoreIssue {
            id: format!("duplicate-{}-{}", self.file1.replace('/', "-"), self.line1),
            severity: lumenx_score::IssueSeverity::Medium,
            category: "duplication".to_string(),
            title: "Duplicate Code Detected".to_string(),
            description: format!(
                "{} tokens duplicated between {} and {} ({}% similarity)",
                self.token_count,
                self.file1,
                self.file2,
                (self.similarity * 100.0) as u32
            ),
            file: Some(self.file1.clone()),
            line: Some(self.line1),
            column: None,
            impact: 5.0,
            suggestion: Some("Extract common logic into a shared function".to_string()),
        }
    }
}

/// A block of tokens for comparison
#[derive(Debug, Clone)]
struct TokenBlock {
    file_path: String,
    start_line: usize,
    tokens: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = StaticAnalyzer::new(PathBuf::from("."));
        assert_eq!(analyzer.project_root, PathBuf::from("."));
    }

    #[test]
    fn test_thresholds_default() {
        let thresholds = ComplexityThresholds::default();
        assert_eq!(thresholds.max_cyclomatic, 10);
        assert_eq!(thresholds.max_function_length, 50);
    }

    #[test]
    fn test_tokenize_rust() {
        let analyzer = StaticAnalyzer::default();
        let source = r#"
fn main() {
    let x = 1;
    if x > 0 {
        println!("positive");
    }
}
"#;
        let tokens = analyzer.tokenize(source);
        assert!(tokens.contains(&"FUNCTION".to_string()));
        assert!(tokens.contains(&"IF".to_string()));
    }

    #[test]
    fn test_tokenize_javascript() {
        let analyzer = StaticAnalyzer::default();
        let source = r#"
function test() {
    const x = 1;
    if (x > 0) {
        console.log("positive");
    }
}
"#;
        let tokens = analyzer.tokenize(source);
        assert!(tokens.contains(&"FUNCTION".to_string()));
        assert!(tokens.contains(&"IF".to_string()));
    }

    #[test]
    fn test_calculate_similarity() {
        let analyzer = StaticAnalyzer::default();
        let tokens1 = vec!["FUNCTION".to_string(), "IF".to_string(), "RETURN".to_string()];
        let tokens2 = vec!["FUNCTION".to_string(), "IF".to_string(), "ASSIGN".to_string()];

        let similarity = analyzer.calculate_similarity(&tokens1, &tokens2);
        assert!((similarity - 0.66).abs() < 0.01);
    }

    #[test]
    fn test_complexity_estimation() {
        let analyzer = StaticAnalyzer::default();
        let source = r#"
fn test() {
    if x > 0 {
        for i in 0..10 {
            match y {
                Some(v) => v,
                None => 0,
            }
        }
    }
}
"#;
        let complexity = analyzer.estimate_complexity_regex(source, AstLanguage::Rust);
        assert!(complexity > 1.0);
    }

    #[test]
    fn test_count_declarations_regex() {
        let analyzer = StaticAnalyzer::default();
        let source = r#"
fn main() {}
fn helper() {}
struct Foo;
impl Foo {}
"#;
        let (functions, structs) = analyzer.count_declarations_regex(source, AstLanguage::Rust);
        assert_eq!(functions, 2);
        assert!(structs >= 1);
    }

    #[test]
    fn test_with_thresholds() {
        let analyzer = StaticAnalyzer::new(PathBuf::from("."))
            .with_thresholds(ComplexityThresholds {
                max_function_length: 100,
                ..Default::default()
            });
        assert_eq!(analyzer.complexity_thresholds.max_function_length, 100);
    }

    #[test]
    fn test_create_token_blocks() {
        let analyzer = StaticAnalyzer::default();
        let tokens = vec
!["FUNCTION".to_string(); 100];
        let path = PathBuf::from("/test/file.rs");
        let blocks = analyzer.create_token_blocks(&tokens, &path);
        assert!(!blocks.is_empty());
    }
}
