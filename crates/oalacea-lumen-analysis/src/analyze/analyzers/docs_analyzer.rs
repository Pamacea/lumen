//! Documentation Analyzer
//!
//! Analyzes codebase for documentation quality including:
//! - README coverage and quality
//! - API documentation (JSDoc, rustdoc, docstrings)
//! - Inline code comments
//! - Architecture documentation
//! - Changelog/version history
//! - Contributing guidelines
//! - License presence

use oalacea_lumen_core::{LumenResult, Project, ProjectInfo, Framework, Language};
use oalacea_lumen_core::scoring::{IssueSeverity, MetricValue, ScoreIssue};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

/// Documentation analyzer
pub struct DocsAnalyzer {
    project_root: String,
}

/// Documentation file found
#[derive(Debug, Clone)]
struct DocFile {
    path: String,
    content: String,
    doc_type: DocType,
}

/// Type of documentation file
#[derive(Debug, Clone, PartialEq)]
enum DocType {
    Readme,
    Changelog,
    Contributing,
    License,
    Architecture,
    Api,
    Guide,
    CodeOfConduct,
    Security,
}

impl DocsAnalyzer {
    /// Create a new documentation analyzer
    pub fn new(project_root: String) -> Self {
        Self { project_root }
    }

    /// Analyze the project for documentation coverage
    pub fn analyze(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // 1. Check for essential documentation files
        let (doc_file_metrics, doc_file_issues) = self.check_doc_files(info)?;
        metrics.extend(doc_file_metrics);
        issues.extend(doc_file_issues);

        // 2. Analyze code documentation coverage
        let (code_doc_metrics, code_doc_issues) = self.analyze_code_documentation(info)?;
        metrics.extend(code_doc_metrics);
        issues.extend(code_doc_issues);

        // 3. Check API documentation
        let (api_doc_metrics, api_doc_issues) = self.check_api_documentation(info)?;
        metrics.extend(api_doc_metrics);
        issues.extend(api_doc_issues);

        // 4. Analyze README quality
        let (readme_metrics, readme_issues) = self.analyze_readme_quality(info)?;
        metrics.extend(readme_metrics);
        issues.extend(readme_issues);

        // 5. Check for examples
        let (example_metrics, example_issues) = self.check_examples(info)?;
        metrics.extend(example_metrics);
        issues.extend(example_issues);

        // Calculate overall documentation score
        let base_score = 100.0;
        let penalty: f64 = issues.iter().map(|i| match i.severity {
            IssueSeverity::Critical => 10.0,
            IssueSeverity::High => 7.0,
            IssueSeverity::Medium => 4.0,
            IssueSeverity::Low => 1.5,
            IssueSeverity::Info => 0.0,
        }).sum();

        metrics.insert(
            "docs:overall_score".to_string(),
            MetricValue::Percentage((base_score - penalty).max(0.0)),
        );

        Ok((metrics, issues))
    }

    /// Check for essential documentation files
    fn check_doc_files(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let root = Path::new(&self.project_root);
        let docs_dir = root.join("docs");

        // Documentation files to check
        let doc_files = [
            ("README", vec!["README.md", "README.txt", "readme.md", "README.rst"]),
            ("CHANGELOG", vec!["CHANGELOG.md", "CHANGES.md", "HISTORY.md", "changelog.md"]),
            ("CONTRIBUTING", vec!["CONTRIBUTING.md", "CONTRIBUTING.rst", "CONTRIBUTE.md", "contributing.md"]),
            ("LICENSE", vec!["LICENSE", "LICENSE.md", "LICENSE.txt", "LICENCE", "COPYING"]),
            ("ARCHITECTURE", vec!["ARCHITECTURE.md", "ARCH.md", "docs/architecture.md", "docs/ARCHITECTURE.md"]),
            ("CODE_OF_CONDUCT", vec!["CODE_OF_CONDUCT.md", "CONDUCT.md", "code-of-conduct.md"]),
            ("SECURITY", vec!["SECURITY.md", "SECURITY.md", "security.md"]),
        ];

        let mut found_docs = HashMap::new();

        for (doc_type, filenames) in &doc_files {
            let mut found = false;
            for filename in filenames {
                let file_path = root.join(filename);
                let docs_path = docs_dir.join(filename);
                if file_path.exists() || docs_path.exists() {
                    found = true;
                    break;
                }
            }
            found_docs.insert(*doc_type, found);
        }

        // Report on missing critical docs
        if !found_docs.get("README").unwrap_or(&false) {
            issues.push(ScoreIssue {
                id: "docs-001".to_string(),
                severity: IssueSeverity::Critical,
                category: "documentation".to_string(),
                title: "Missing README file".to_string(),
                description: "Every project should have a README.md explaining what the project is, how to install it, and how to use it.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 10.0,
                suggestion: Some("Create a README.md with:\n\n```markdown\n# Project Name\n\nA brief description of what your project does and who it's for.\n\n## Installation\n\n```bash\nnpm install your-package\n```\n\n## Quick Start\n\n```javascript\nimport { something } from 'your-package';\n\n// Your code here\n```\n\n## Documentation\n\nFull documentation is available at [link].\n\n## Contributing\n\nContributions are welcome! Please see CONTRIBUTING.md for details.\n\n## License\n\nMIT License - see LICENSE file for details.\n```".to_string()),
            });
        }

        if !found_docs.get("LICENSE").unwrap_or(&false) {
            issues.push(ScoreIssue {
                id: "docs-002".to_string(),
                severity: IssueSeverity::High,
                category: "documentation".to_string(),
                title: "Missing LICENSE file".to_string(),
                description: "An open-source license clarifies how others can use, modify, and distribute your code.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 7.0,
                suggestion: Some("Choose an open-source license:\n\n- **MIT License**: Simple, permissive, most common\n- **Apache 2.0**: Patent protections, more complex\n- **GPL v3**: Copyleft, requires derived works to be GPL\n- **BSD 3-Clause**: Simple, permissive like MIT\n\nUse [choosealicense.com](https://choosealicense.com/) to help decide.\n\n```bash\n# Add a LICENSE file to your project root\necho \"MIT License\n\nCopyright (c) $(date +%Y) Your Name\n\nPermission is hereby granted...\" > LICENSE\n```".to_string()),
            });
        }

        if !found_docs.get("CONTRIBUTING").unwrap_or(&false) {
            issues.push(ScoreIssue {
                id: "docs-003".to_string(),
                severity: IssueSeverity::Medium,
                category: "documentation".to_string(),
                title: "Missing CONTRIBUTING guide".to_string(),
                description: "A CONTRIBUTING guide helps new contributors understand how to participate in your project.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 4.0,
                suggestion: Some("Create a CONTRIBUTING.md with:\n\n```markdown\n# Contributing to Project Name\n\nThank you for your interest in contributing!\n\n## How to Contribute\n\n1. Check existing issues\n2. Fork the repository\n3. Create a feature branch\n4. Make your changes\n5. Write tests\n6. Submit a pull request\n\n## Development Setup\n\n```bash\ngit clone https://github.com/your/repo.git\ncd repo\nnpm install\nnpm test\n```\n\n## Coding Standards\n\n- Follow the existing code style\n- Write tests for new features\n- Update documentation\n\n## Pull Request Process\n\n- Describe your changes clearly\n- Reference related issues\n- Ensure tests pass\n```".to_string()),
            });
        }

        if !found_docs.get("CHANGELOG").unwrap_or(&false) {
            issues.push(ScoreIssue {
                id: "docs-004".to_string(),
                severity: IssueSeverity::Low,
                category: "documentation".to_string(),
                title: "Missing CHANGELOG file".to_string(),
                description: "A CHANGELOG helps users track what changed between versions.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 1.5,
                suggestion: Some("Create a CHANGELOG.md following [Keep a Changelog](https://keepachangelog.com/) format:\n\n```markdown\n# Changelog\n\nAll notable changes to this project will be documented in this file.\n\nThe format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\nand this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n## [Unreleased]\n\n### Added\n- New feature description\n\n### Changed\n- Changed feature description\n\n### Deprecated\n- Deprecated feature description\n\n### Removed\n- Removed feature description\n\n### Fixed\n- Bug fix description\n\n### Security\n- Security fix description\n```".to_string()),
            });
        }

        // Count total essential docs found
        let essential_count = ["README", "LICENSE", "CONTRIBUTING"]
            .iter()
            .filter(|&&doc| *found_docs.get(doc).unwrap_or(&false))
            .count();

        metrics.insert(
            "docs:essential_files_count".to_string(),
            MetricValue::Count(essential_count),
        );

        metrics.insert(
            "docs:has_readme".to_string(),
            MetricValue::Boolean(*found_docs.get("README").unwrap_or(&false)),
        );

        metrics.insert(
            "docs:has_license".to_string(),
            MetricValue::Boolean(*found_docs.get("LICENSE").unwrap_or(&false)),
        );

        metrics.insert(
            "docs:has_contributing".to_string(),
            MetricValue::Boolean(*found_docs.get("CONTRIBUTING").unwrap_or(&false)),
        );

        metrics.insert(
            "docs:has_changelog".to_string(),
            MetricValue::Boolean(*found_docs.get("CHANGELOG").unwrap_or(&false)),
        );

        Ok((metrics, issues))
    }

    /// Analyze code documentation coverage
    fn analyze_code_documentation(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let source_files = self.find_source_files(info)?;

        let mut total_functions = 0;
        let mut documented_functions = 0;
        let mut total_lines = 0;
        let mut comment_lines = 0;

        for file_path in &source_files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                match info.language {
                    Language::Rust => {
                        self.analyze_rust_docs(&content, &mut total_functions, &mut documented_functions, &mut total_lines, &mut comment_lines);
                    }
                    Language::TypeScript | Language::JavaScript => {
                        self.analyze_js_docs(&content, &mut total_functions, &mut documented_functions, &mut total_lines, &mut comment_lines);
                    }
                    Language::Python => {
                        self.analyze_python_docs(&content, &mut total_functions, &mut documented_functions, &mut total_lines, &mut comment_lines);
                    }
                    Language::Go => {
                        self.analyze_go_docs(&content, &mut total_functions, &mut documented_functions, &mut total_lines, &mut comment_lines);
                    }
                    _ => {}
                }
            }
        }

        let doc_coverage = if total_functions > 0 {
            (documented_functions as f64 / total_functions as f64) * 100.0
        } else {
            0.0
        };

        let comment_ratio = if total_lines > 0 {
            (comment_lines as f64 / total_lines as f64) * 100.0
        } else {
            0.0
        };

        metrics.insert(
            "docs:function_coverage".to_string(),
            MetricValue::Percentage(doc_coverage),
        );

        metrics.insert(
            "docs:comment_ratio".to_string(),
            MetricValue::Percentage(comment_ratio),
        );

        metrics.insert(
            "docs:total_functions".to_string(),
            MetricValue::Count(total_functions),
        );

        metrics.insert(
            "docs:documented_functions".to_string(),
            MetricValue::Count(documented_functions),
        );

        if doc_coverage < 50.0 && total_functions > 10 {
            issues.push(ScoreIssue {
                id: "docs-005".to_string(),
                severity: IssueSeverity::Medium,
                category: "documentation".to_string(),
                title: "Low code documentation coverage".to_string(),
                description: format!("Only {:.1}% of functions are documented. Good documentation helps others (and future you) understand the code.", doc_coverage),
                file: None,
                line: None,
                column: None,
                impact: 4.0,
                suggestion: Some(match info.language {
                    Language::Rust => "Add rustdoc comments:\n\n```rust\n/// Calculates the factorial of a number.\n///\n/// # Arguments\n///\n/// * `n` - The number to calculate the factorial of\n///\n/// # Returns\n///\n/// The factorial of n\n///\n/// # Examples\n///\n/// ```\n/// assert_eq!(factorial(5), 120);\n/// ```\nfn factorial(n: u64) -> u64 {\n    (1..=n).product()\n}\n```".to_string(),
                    Language::TypeScript | Language::JavaScript => "Add JSDoc comments:\n\n```typescript\n/**\n * Calculates the factorial of a number.\n *\n * @param n - The number to calculate the factorial of\n * @returns The factorial of n\n *\n * @example\n * ```ts\n * factorial(5) // returns 120\n * ```\n */\nexport function factorial(n: number): number {\n  return n <= 1 ? 1 : n * factorial(n - 1);\n}\n```".to_string(),
                    Language::Python => "Add docstrings:\n\n```python\ndef factorial(n: int) -> int:\n    \"\"\"Calculate the factorial of a number.\n\n    Args:\n        n: The number to calculate the factorial of\n\n    Returns:\n        The factorial of n\n\n    Examples:\n        >>> factorial(5)\n        120\n    \"\"\"\n    return 1 if n <= 1 else n * factorial(n - 1)\n```".to_string(),
                    _ => "Add documentation comments to your functions explaining what they do, their parameters, and return values.".to_string(),
                }),
            });
        }

        Ok((metrics, issues))
    }

    /// Analyze Rust documentation
    fn analyze_rust_docs(&self, content: &str, total_funcs: &mut usize, documented_funcs: &mut usize, total_lines: &mut usize, comment_lines: &mut usize) {
        let function_regex = Regex::new(r"pub\s+fn\s+\w+").unwrap();
        let doc_regex = Regex::new(r#"///[^!].*"#).unwrap();

        *total_funcs += function_regex.find_iter(content).count();
        *documented_funcs += doc_regex.find_iter(content).count();

        for line in content.lines() {
            *total_lines += 1;
            if line.trim().starts_with("///") || line.trim().starts_with("//") {
                *comment_lines += 1;
            }
        }
    }

    /// Analyze JavaScript/TypeScript documentation
    fn analyze_js_docs(&self, content: &str, total_funcs: &mut usize, documented_funcs: &mut usize, total_lines: &mut usize, comment_lines: &mut usize) {
        let function_regex = Regex::new(r"(?:(?:function\s+\w+)|(?:const\s+\w+\s*=\s*(?:async\s+)?\([^)]*\)\s*=>)|(?:\w+\s*\([^)]*\)\s*\{))").unwrap();
        let jsdoc_regex = Regex::new(r#"/\*\*[^*]*\*+(?:[^/*][^*]*\*+)*/"#).unwrap();

        *total_funcs += function_regex.find_iter(content).count();
        *documented_funcs += jsdoc_regex.find_iter(content).count();

        for line in content.lines() {
            *total_lines += 1;
            let trimmed = line.trim();
            if trimmed.starts_with("/**") || trimmed.starts_with("*") || trimmed.starts_with("//") {
                *comment_lines += 1;
            }
        }
    }

    /// Analyze Python documentation
    fn analyze_python_docs(&self, content: &str, total_funcs: &mut usize, documented_funcs: &mut usize, total_lines: &mut usize, comment_lines: &mut usize) {
        let function_regex = Regex::new(r"def\s+\w+\s*\(").unwrap();
        let docstring_regex = Regex::new(r#"\"\"\"[^\"]*\"\"\"|'''[^']*'''"#).unwrap();

        *total_funcs += function_regex.find_iter(content).count();
        *documented_funcs += docstring_regex.find_iter(content).count();

        for line in content.lines() {
            *total_lines += 1;
            if line.trim().starts_with("#") || line.trim().starts_with("\"\"\"") || line.trim().starts_with("'''") {
                *comment_lines += 1;
            }
        }
    }

    /// Analyze Go documentation
    fn analyze_go_docs(&self, content: &str, total_funcs: &mut usize, documented_funcs: &mut usize, total_lines: &mut usize, comment_lines: &mut usize) {
        let function_regex = Regex::new(r"func\s+\w+\s*\(").unwrap();
        let godoc_regex = Regex::new(r"//\s+[A-Z].*").unwrap();

        *total_funcs += function_regex.find_iter(content).count();
        *documented_funcs += godoc_regex.find_iter(content).count();

        for line in content.lines() {
            *total_lines += 1;
            if line.trim().starts_with("//") {
                *comment_lines += 1;
            }
        }
    }

    /// Check API documentation
    fn check_api_documentation(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let root = Path::new(&self.project_root);

        // Check for API documentation in common locations
        let api_doc_locations = [
            root.join("docs").join("api.md"),
            root.join("docs").join("API.md"),
            root.join("docs").join("api").join("README.md"),
            root.join("API.md"),
            root.join("api.md"),
            root.join("openapi.yaml"),
            root.join("openapi.yml"),
            root.join("swagger.yaml"),
            root.join("swagger.yml"),
        ];

        let has_api_docs = api_doc_locations.iter().any(|p| p.exists());

        metrics.insert(
            "docs:has_api_documentation".to_string(),
            MetricValue::Boolean(has_api_docs),
        );

        // Check for OpenAPI/Swagger spec for backend frameworks
        if matches!(info.framework, Framework::NestJS | Framework::Express | Framework::Fastify | Framework::Axum | Framework::ActixWeb) && !has_api_docs {
            issues.push(ScoreIssue {
                id: "docs-006".to_string(),
                severity: IssueSeverity::Low,
                category: "documentation".to_string(),
                title: "Missing API documentation".to_string(),
                description: "API documentation helps consumers understand available endpoints, request/response formats, and authentication.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 3.0,
                suggestion: Some("Create API documentation:\n\n**Option 1: OpenAPI/Swagger Spec**\n\n```yaml\n# openapi.yaml\nopenapi: 3.0.0\ninfo:\n  title: Your API\n  version: 1.0.0\npaths:\n  /users:\n    get:\n      summary: List users\n      responses:\n        '200':\n          description: Successful response\n```\n\n**Option 2: NestJS Swagger**\n\n```typescript\n// main.ts\nimport { DocumentBuilder, SwaggerModule } from '@nestjs/swagger';\n\nconst config = new DocumentBuilder()\n  .setTitle('API')\n  .setVersion('1.0')\n  .build();\nconst document = SwaggerModule.createDocument(app, config);\nSwaggerModule.setup('api-docs', app, document);\n```\n\n**Option 3: Rust with utoipa**\n\n```toml\ncargo add utoipa\n```\n\n```rust\nuse utoipa::ToSchema;\n\n#[derive(ToSchema)]\nstruct User {\n    id: u64,\n    name: String,\n}\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Analyze README quality
    fn analyze_readme_quality(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let root = Path::new(&self.project_root);

        // Find README
        let readme_paths = [
            root.join("README.md"),
            root.join("readme.md"),
            root.join("README.txt"),
        ];

        let readme_path = readme_paths.iter().find(|p| p.exists());

        let readme_score = if let Some(path) = readme_path {
            if let Ok(content) = std::fs::read_to_string(path) {
                self.evaluate_readme_quality(&content)
            } else {
                0.0
            }
        } else {
            0.0
        };

        metrics.insert(
            "docs:readme_quality_score".to_string(),
            MetricValue::Percentage(readme_score),
        );

        if readme_score < 50.0 && readme_path.is_some() {
            issues.push(ScoreIssue {
                id: "docs-007".to_string(),
                severity: IssueSeverity::Medium,
                category: "documentation".to_string(),
                title: "README needs improvement".to_string(),
                description: format!("README quality score is {:.0}/100. A good README should include project description, installation, usage, and contributing instructions.", readme_score),
                file: readme_path.map(|p| p.to_string_lossy().to_string()),
                line: None,
                column: None,
                impact: 4.0,
                suggestion: Some("Improve your README with these sections:\n\n```markdown\n# Project Name\n\n**Short description** - What does this project do?\n\n## 🚀 Features\n\n- Feature 1\n- Feature 2\n- Feature 3\n\n## 📦 Installation\n\n```bash\nnpm install project-name\n```\n\n## 🛠️ Usage\n\n```javascript\nimport { something } from 'project-name';\n\n// Your example code\n```\n\n## 📖 API Documentation\n\nLink to full API docs\n\n## 🤝 Contributing\n\nContributions welcome! See CONTRIBUTING.md\n\n## 📄 License\n\nMIT © Your Name\n\n## 🔗 Links\n\n- Documentation\n- Issue Tracker\n- Changelog\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Evaluate README quality
    fn evaluate_readme_quality(&self, content: &str) -> f64 {
        let mut score: f64 = 0.0;

        // Check for essential sections
        let has_title = Regex::new(r"^#\s+\w+").unwrap().is_match(content);
        let has_description = content.len() > 200 && content.len() < 5000;
        let has_installation = Regex::new(r"(?i)(install|installation|setup|getting started)").unwrap().is_match(content);
        let has_usage = Regex::new(r"(?i)(usage|example|how to use|quick start)").unwrap().is_match(content);
        let has_code_blocks = Regex::new(r"```").unwrap().is_match(content);
        let has_badges = Regex::new(r"!\[.*\]\(.*\)").unwrap().is_match(content);
        let has_contributing = Regex::new(r"(?i)(contributing|contribute)").unwrap().is_match(content);
        let has_license = Regex::new(r"(?i)(license|mit|apache)").unwrap().is_match(content);

        // Score based on presence of sections
        if has_title { score += 10.0; }
        if has_description { score += 15.0; }
        if has_installation { score += 20.0; }
        if has_usage { score += 20.0; }
        if has_code_blocks { score += 10.0; }
        if has_badges { score += 5.0; }
        if has_contributing { score += 10.0; }
        if has_license { score += 10.0; }

        score.min(100.0)
    }

    /// Check for examples
    fn check_examples(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let root = Path::new(&self.project_root);

        // Check for example directories
        let example_dirs = [
            root.join("examples"),
            root.join("example"),
            root.join("docs").join("examples"),
            root.join("samples"),
        ];

        let has_examples = example_dirs.iter().any(|p| p.exists());
        let mut example_count = 0;

        for dir in &example_dirs {
            if dir.exists() && dir.is_dir() {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    example_count += entries.filter_map(|e| e.ok()).count();
                }
            }
        }

        metrics.insert(
            "docs:has_examples".to_string(),
            MetricValue::Boolean(has_examples),
        );

        metrics.insert(
            "docs:example_count".to_string(),
            MetricValue::Count(example_count),
        );

        // Check README for usage examples
        let readme_path = root.join("README.md");
        let has_readme_examples = if readme_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&readme_path) {
                Regex::new(r"```").unwrap().find_iter(&content).count() >= 2
            } else {
                false
            }
        } else {
            false
        };

        if !has_examples && !has_readme_examples {
            issues.push(ScoreIssue {
                id: "docs-008".to_string(),
                severity: IssueSeverity::Low,
                category: "documentation".to_string(),
                title: "No examples found".to_string(),
                description: "Code examples help users understand how to use your project.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 2.0,
                suggestion: Some("Add examples to help users:\n\n**Option 1: Examples directory**\n\n```\nexamples/\n├── basic-usage/\n├── advanced-usage/\n└── integration-examples/\n```\n\n**Option 2: Add to README**\n\n```markdown\n## Usage\n\n### Basic Example\n\n```javascript\nconst result = yourLibrary.configure({\n  option: 'value',\n});\n```\n\n### Advanced Example\n\n```javascript\nconst advanced = yourLibrary.configure({\n  option: 'value',\n  callback: (data) => console.log(data),\n});\n```\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Find all source files in the project
    fn find_source_files(&self, info: &ProjectInfo) -> LumenResult<Vec<std::path::PathBuf>> {
        let mut files = Vec::new();

        let extensions = match info.language {
            Language::TypeScript | Language::JavaScript => &["ts", "tsx", "js", "jsx"][..],
            Language::Rust => &["rs"][..],
            Language::Python => &["py"][..],
            Language::Go => &["go"][..],
            _ => &["ts", "tsx", "js", "jsx", "rs", "py", "go"],
        };

        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if extensions.contains(&ext.to_str().unwrap_or("")) {
                        let path_str = path.to_string_lossy();
                        if !path_str.contains("node_modules")
                            && !path_str.contains("target")
                            && !path_str.contains("dist")
                        {
                            files.push(path.to_path_buf());
                        }
                    }
                }
            }
        }

        Ok(files)
    }
}

/// Public function for backward compatibility with the module interface
pub fn analyze(project: &oalacea_lumen_core::Project) -> Vec<ScoreIssue> {
    let analyzer = DocsAnalyzer::new(project.info.root.to_string_lossy().to_string());
    let (_, issues) = analyzer.analyze(&project.info).unwrap_or_default();
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docs_analyzer_creation() {
        let analyzer = DocsAnalyzer::new("/tmp".to_string());
        assert_eq!(analyzer.project_root, "/tmp");
    }

    #[test]
    fn test_readme_quality_evaluation() {
        let analyzer = DocsAnalyzer::new("/tmp".to_string());

        let good_readme = r#"
# My Project

This is an amazing project that does amazing things.

## Installation

```bash
npm install my-project
```

## Usage

```javascript
const myProject = require('my-project');
myProject.doSomething();
```

## Contributing

We welcome contributions!

## License

MIT
"#;

        let score = analyzer.evaluate_readme_quality(good_readme);
        assert!(score > 70.0, "Good README should score above 70, got {}", score);

        let bad_readme = "# My Project\nShort.\n";
        let bad_score = analyzer.evaluate_readme_quality(bad_readme);
        assert!(bad_score < 30.0, "Bad README should score below 30, got {}", bad_score);
    }
}
