//! # Lumen Test Generation
//!
//! Framework-specific test template generator.
//!
//! This crate analyzes source code and generates intelligent test templates
//! for multiple programming languages and testing frameworks.

pub mod analyzer;
pub mod code_parser;
pub mod templates;
pub mod generator;

use lumen_core::{Framework, LumenResult, Language, ProjectInfo};
use std::path::{Path, PathBuf};

/// Test generation result
#[derive(Debug, Clone)]
pub struct TestGenerationResult {
    /// Generated test files
    pub test_files: Vec<TestFile>,
    /// Number of unit tests
    pub unit_tests: usize,
    /// Number of integration tests
    pub integration_tests: usize,
    /// Number of E2E tests
    pub e2e_tests: usize,
    /// Functions that were analyzed
    pub analyzed_functions: Vec<FunctionInfo>,
    /// Functions without tests (coverage gaps)
    pub untested_functions: Vec<FunctionInfo>,
}

/// Generated test file
#[derive(Debug, Clone)]
pub struct TestFile {
    /// File path (relative to project root)
    pub path: PathBuf,
    /// File content
    pub content: String,
    /// Test type
    pub test_type: TestType,
}

/// Test type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestType {
    Unit,
    Integration,
    E2E,
}

/// Function information extracted from source code
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Source file path
    pub file_path: PathBuf,
    /// Line number
    pub line: usize,
    /// Function signature
    pub signature: String,
    /// Parameters
    pub parameters: Vec<Parameter>,
    /// Return type
    pub return_type: Option<String>,
    /// Is async
    pub is_async: bool,
    /// Visibility
    pub visibility: Visibility,
    /// Language
    pub language: Language,
    /// Doc comment
    pub doc_comment: Option<String>,
    /// Suggested test cases
    pub suggested_tests: Vec<TestCase>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Parameter {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: Option<String>,
    /// Is optional
    pub is_optional: bool,
    /// Default value (if any)
    pub default_value: Option<String>,
}

/// Function/method visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

/// Suggested test case
#[derive(Debug, Clone)]
pub struct TestCase {
    /// Test case description
    pub description: String,
    /// Test case type (happy path, edge case, error case)
    pub case_type: TestCaseType,
    /// Input values (suggested)
    pub inputs: Vec<TestValue>,
    /// Expected output
    pub expected_output: Option<String>,
}

/// Test case type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestCaseType {
    HappyPath,
    EdgeCase,
    ErrorCase,
    Boundary,
}

/// Test value placeholder
#[derive(Debug, Clone)]
pub struct TestValue {
    /// Parameter name
    pub name: String,
    /// Suggested value
    pub value: String,
    /// Value type
    pub value_type: String,
}

/// Test generation options
#[derive(Debug, Clone)]
pub struct TestGeneratorOptions {
    /// Generate only unit tests
    pub unit_only: bool,
    /// Generate only integration tests
    pub integration_only: bool,
    /// Generate only E2E tests
    pub e2e_only: bool,
    /// Skip existing test files
    pub skip_existing: bool,
    /// Include mock data generation
    pub include_mocks: bool,
    /// Maximum test cases per function
    pub max_tests_per_function: usize,
    /// Target coverage percentage
    pub target_coverage: f64,
}

impl Default for TestGeneratorOptions {
    fn default() -> Self {
        Self {
            unit_only: false,
            integration_only: false,
            e2e_only: false,
            skip_existing: true,
            include_mocks: true,
            max_tests_per_function: 5,
            target_coverage: 80.0,
        }
    }
}

/// Test framework
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestFramework {
    Vitest,
    Jest,
    RustBuiltIn,
    Pytest,
    GoTest,
    Mocha,
}

impl TestFramework {
    /// Detect test framework from language and project info
    pub fn from_project(info: &ProjectInfo) -> Self {
        match (info.language.clone(), &info.test_runner) {
            (Language::TypeScript | Language::JavaScript, _) => {
                // Check package.json for test runner
                if let Some(pkg) = &info.package_json {
                    if let Some(scripts) = &pkg.scripts {
                        if scripts.contains_key("test") {
                            let test_script = scripts.get("test").unwrap();
                            if test_script.contains("vitest") {
                                return TestFramework::Vitest;
                            } else if test_script.contains("jest") {
                                return TestFramework::Jest;
                            } else if test_script.contains("mocha") {
                                return TestFramework::Mocha;
                            }
                        }
                    }
                    // Check dev dependencies
                    if let Some(dev_deps) = &pkg.dev_dependencies {
                        if dev_deps.contains_key("vitest") {
                            return TestFramework::Vitest;
                        } else if dev_deps.contains_key("jest") {
                            return TestFramework::Jest;
                        } else if dev_deps.contains_key("mocha") {
                            return TestFramework::Mocha;
                        }
                    }
                }
                // Default based on framework
                match info.framework {
                    Framework::NextJs | Framework::Remix | Framework::ViteReact
                    | Framework::ViteVue | Framework::ViteSvelte | Framework::Solid => TestFramework::Vitest,
                    Framework::Nuxt | Framework::SvelteKit | Framework::Astro => TestFramework::Vitest,
                    _ => TestFramework::Vitest,
                }
            }
            (Language::Rust, _) => TestFramework::RustBuiltIn,
            (Language::Python, _) => TestFramework::Pytest,
            (Language::Go, _) => TestFramework::GoTest,
            _ => TestFramework::RustBuiltIn,
        }
    }

    /// Get file extension for test files
    pub fn test_extension(&self) -> &str {
        match self {
            TestFramework::Vitest | TestFramework::Jest | TestFramework::Mocha => ".test.ts",
            TestFramework::RustBuiltIn => ".rs",
            TestFramework::Pytest => "_test.py",
            TestFramework::GoTest => "_test.go",
        }
    }

    /// Get test file naming pattern
    pub fn test_file_pattern(&self) -> &str {
        match self {
            TestFramework::Vitest | TestFramework::Jest | TestFramework::Mocha => "{name}.test.ts",
            TestFramework::RustBuiltIn => "{name}_test.rs",
            TestFramework::Pytest => "test_{name}.py",
            TestFramework::GoTest => "{name}_test.go",
        }
    }
}

/// Main test generator
pub struct TestGenerator {
    /// Project root directory
    project_root: PathBuf,
    /// Test framework
    framework: TestFramework,
    /// Project info
    project_info: Option<ProjectInfo>,
    /// Generation options
    options: TestGeneratorOptions,
    /// Dry run (don't write files)
    dry_run: bool,
}

impl TestGenerator {
    /// Create a new test generator
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            framework: TestFramework::Vitest,
            project_info: None,
            options: TestGeneratorOptions::default(),
            dry_run: false,
        }
    }

    /// Set the test framework
    pub fn with_framework(mut self, framework: TestFramework) -> Self {
        self.framework = framework;
        self
    }

    /// Set generation options
    pub fn with_options(mut self, options: TestGeneratorOptions) -> Self {
        self.options = options;
        self
    }

    /// Set dry run mode
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Set project info
    pub fn with_project_info(mut self, info: ProjectInfo) -> Self {
        self.framework = TestFramework::from_project(&info);
        self.project_info = Some(info);
        self
    }

    /// Generate tests for the project
    pub fn generate(&self, info: &ProjectInfo) -> LumenResult<TestGenerationResult> {
        tracing::info!("Generating tests for project: {}", info.name);

        // Analyze source code to extract functions
        let functions = analyzer::analyze_project(&self.project_root, info)?;

        // Determine which functions need tests
        let untested: Vec<FunctionInfo> = if self.options.skip_existing {
            functions
                .into_iter()
                .filter(|f| !self.has_existing_test(f))
                .collect()
        } else {
            functions
        };

        tracing::info!("Found {} untested functions", untested.len());

        // Generate test files
        let mut test_files = Vec::new();
        let mut unit_count = 0;
        let mut integration_count = 0;
        let mut e2e_count = 0;

        for function in &untested {
            if self.options.unit_only {
                let tests = self.generate_unit_test(function)?;
                if !tests.content.is_empty() {
                    unit_count += 1;
                    test_files.push(tests);
                }
            } else if self.options.integration_only {
                if let Some(tests) = self.generate_integration_test(function)? {
                    integration_count += 1;
                    test_files.push(tests);
                }
            } else if self.options.e2e_only {
                if let Some(tests) = self.generate_e2e_test(function)? {
                    e2e_count += 1;
                    test_files.push(tests);
                }
            } else {
                // Generate all types based on function characteristics
                let unit_test = self.generate_unit_test(function)?;
                if !unit_test.content.is_empty() {
                    unit_count += 1;
                    test_files.push(unit_test);
                }

                if self.is_endpoint_handler(function) {
                    if let Some(integration_test) = self.generate_integration_test(function)? {
                        integration_count += 1;
                        test_files.push(integration_test);
                    }
                }
            }
        }

        // Generate E2E tests for critical flows if not e2e_only
        if !self.options.unit_only && !self.options.integration_only && !self.options.e2e_only {
            let e2e_tests = self.generate_e2e_tests(info)?;
            e2e_count += e2e_tests.len();
            test_files.extend(e2e_tests);
        }

        Ok(TestGenerationResult {
            test_files,
            unit_tests: unit_count,
            integration_tests: integration_count,
            e2e_tests: e2e_count,
            analyzed_functions: untested.clone(),
            untested_functions: untested,
        })
    }

    /// Generate a single test for a specific function
    pub fn generate_test_for_function(&self, file: &Path, fn_name: &str) -> LumenResult<String> {
        let functions = code_parser::parse_file(file, self.framework)?;
        let func = functions
            .iter()
            .find(|f| f.name == fn_name)
            .ok_or_else(|| {
                lumen_core::LumenError::ParseError(format!("Function '{}' not found in {:?}", fn_name, file))
            })?;

        Ok(match self.framework {
            TestFramework::Vitest | TestFramework::Jest => {
                templates::typescript::generate_function_test(func, self.framework)
            }
            TestFramework::RustBuiltIn => templates::rust::generate_function_test(func),
            TestFramework::Pytest => templates::python::generate_function_test(func),
            TestFramework::GoTest => templates::go::generate_function_test(func),
            TestFramework::Mocha => templates::typescript::generate_function_test(func, self.framework),
        })
    }

    /// Check if a function already has tests
    fn has_existing_test(&self, function: &FunctionInfo) -> bool {
        let test_path = self.get_test_path_for_function(function);
        test_path.exists()
    }

    /// Get the test file path for a function
    fn get_test_path_for_function(&self, function: &FunctionInfo) -> PathBuf {
        let source_stem = function
            .file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let test_name = format!("{}.{}", source_stem, self.framework.test_extension().trim_start_matches('.'));

        // Determine test directory
        let test_dir = match function.language {
            Language::TypeScript | Language::JavaScript => {
                // Check for __tests__ directory or co-located tests
                let tests_dir = self.project_root.join("__tests__");
                if tests_dir.exists() {
                    tests_dir
                } else {
                    // Co-located with source
                    function
                        .file_path
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| self.project_root.clone())
                }
            }
            Language::Rust => {
                // Rust tests are in the same module
                function
                    .file_path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| self.project_root.join("src"))
            }
            Language::Python => {
                // Python tests are in tests/ directory
                self.project_root.join("tests")
            }
            Language::Go => {
                // Go tests are in the same package
                function
                    .file_path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| self.project_root.clone())
            }
            _ => self.project_root.clone(),
        };

        test_dir.join(test_name)
    }

    /// Generate unit test for a function
    fn generate_unit_test(&self, function: &FunctionInfo) -> LumenResult<TestFile> {
        let test_path = self.get_test_path_for_function(function);
        let content = match self.framework {
            TestFramework::Vitest | TestFramework::Jest | TestFramework::Mocha => {
                templates::typescript::generate_unit_test(function, self.framework, &self.options)
            }
            TestFramework::RustBuiltIn => templates::rust::generate_unit_test(function, &self.options),
            TestFramework::Pytest => templates::python::generate_unit_test(function, &self.options),
            TestFramework::GoTest => templates::go::generate_unit_test(function, &self.options),
        };

        Ok(TestFile {
            path: test_path
                .strip_prefix(&self.project_root)
                .unwrap_or(&test_path)
                .to_path_buf(),
            content,
            test_type: TestType::Unit,
        })
    }

    /// Generate integration test for an endpoint handler
    fn generate_integration_test(&self, function: &FunctionInfo) -> LumenResult<Option<TestFile>> {
        if !self.is_endpoint_handler(function) {
            return Ok(None);
        }

        let test_path = self.get_integration_test_path(function);
        let content = match self.framework {
            TestFramework::Vitest | TestFramework::Jest | TestFramework::Mocha => {
                templates::typescript::generate_integration_test(function, self.framework)
            }
            TestFramework::RustBuiltIn => templates::rust::generate_integration_test(function),
            TestFramework::Pytest => templates::python::generate_integration_test(function),
            TestFramework::GoTest => templates::go::generate_integration_test(function),
        };

        Ok(Some(TestFile {
            path: test_path
                .strip_prefix(&self.project_root)
                .unwrap_or(&test_path)
                .to_path_buf(),
            content,
            test_type: TestType::Integration,
        }))
    }

    /// Generate E2E test for a critical flow
    fn generate_e2e_test(&self, function: &FunctionInfo) -> LumenResult<Option<TestFile>> {
        // Only generate E2E tests for API routes/controllers
        if !self.is_endpoint_handler(function) {
            return Ok(None);
        }

        let test_path = self.get_e2e_test_path();
        let content = match self.framework {
            TestFramework::Vitest | TestFramework::Jest | TestFramework::Mocha => {
                templates::typescript::generate_e2e_test(function, self.framework)
            }
            TestFramework::RustBuiltIn => templates::rust::generate_e2e_test(function),
            TestFramework::Pytest => templates::python::generate_e2e_test(function),
            TestFramework::GoTest => templates::go::generate_e2e_test(function),
        };

        Ok(Some(TestFile {
            path: test_path
                .strip_prefix(&self.project_root)
                .unwrap_or(&test_path)
                .to_path_buf(),
            content,
            test_type: TestType::E2E,
        }))
    }

    /// Generate E2E tests for critical user flows
    fn generate_e2e_tests(&self, info: &ProjectInfo) -> LumenResult<Vec<TestFile>> {
        match info.framework {
            Framework::NextJs => Ok(vec![templates::vitest::generate_nextjs_e2e(info)]),
            Framework::NestJS => Ok(vec![templates::nestjs::generate_nestjs_e2e(info)]),
            Framework::Axum => Ok(vec![templates::rust::generate_axum_e2e(info)]),
            _ => Ok(vec![]),
        }
    }

    /// Get integration test path
    fn get_integration_test_path(&self, function: &FunctionInfo) -> PathBuf {
        match function.language {
            Language::TypeScript | Language::JavaScript => {
                self.project_root.join("tests").join(format!(
                    "{}.{}.ts",
                    function.name.to_lowercase(),
                    "integration"
                ))
            }
            Language::Rust => {
                self.project_root.join("tests").join(format!(
                    "{}_test.rs",
                    function.name.to_lowercase()
                ))
            }
            Language::Python => {
                self.project_root.join("tests").join(format!(
                    "test_{}_integration.py",
                    function.name.to_lowercase()
                ))
            }
            _ => self.project_root.join("tests").join("integration.test.ts"),
        }
    }

    /// Get E2E test path
    fn get_e2e_test_path(&self) -> PathBuf {
        self.project_root.join("e2e").join("critical-flows.test.ts")
    }

    /// Check if function is an endpoint handler
    fn is_endpoint_handler(&self, function: &FunctionInfo) -> bool {
        // Check by naming convention
        let handler_keywords = [
            "handler",
            "route",
            "controller",
            "action",
            "endpoint",
            "api",
        ];

        let name_lower = function.name.to_lowercase();

        // Check function name
        if handler_keywords.iter().any(|kw| name_lower.contains(kw)) {
            return true;
        }

        // Check parameters for HTTP request/response types
        function.parameters.iter().any(|p| {
            p.param_type
                .as_ref()
                .map(|t| {
                    t.contains("Request")
                        || t.contains("Response")
                        || t.contains("req")
                        || t.contains("res")
                        || t.contains("ctx")
                })
                .unwrap_or(false)
        })
    }

    /// Write generated tests to disk
    pub fn write_tests(&self, result: &TestGenerationResult) -> LumenResult<()> {
        if self.dry_run {
            tracing::info!("Dry run: skipping file write");
            for test_file in &result.test_files {
                tracing::info!("Would write: {}", test_file.path.display());
                tracing::info!("--- Content preview (first 500 chars) ---");
                let preview: String = test_file.content.chars().take(500).collect();
                tracing::info!("{}", preview);
                if test_file.content.len() > 500 {
                    tracing::info!("... ({} more chars)", test_file.content.len() - 500);
                }
            }
            return Ok(());
        }

        for test_file in &result.test_files {
            let full_path = self.project_root.join(&test_file.path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Check if file exists
            if full_path.exists() && self.options.skip_existing {
                tracing::info!("Skipping existing test file: {}", test_file.path.display());
                continue;
            }

            std::fs::write(&full_path, &test_file.content)?;
            tracing::info!("Wrote test file: {}", test_file.path.display());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_from_project_typescript() {
        let info = ProjectInfo {
            name: "test-project".to_string(),
            root: PathBuf::from("/test"),
            framework: Framework::NextJs,
            language: Language::TypeScript,
            test_runner: lumen_core::TestRunner::Vitest,
            package_manager: Some("npm".to_string()),
            dependencies: vec![],
            dev_dependencies: vec!["vitest".to_string()],
            database: None,
            package_json: None,
            cargo_dependencies: None,
        };

        let framework = TestFramework::from_project(&info);
        assert_eq!(framework, TestFramework::Vitest);
    }

    #[test]
    fn test_framework_from_project_rust() {
        let info = ProjectInfo {
            name: "test-project".to_string(),
            root: PathBuf::from("/test"),
            framework: Framework::Axum,
            language: Language::Rust,
            test_runner: lumen_core::TestRunner::CargoTest,
            package_manager: None,
            dependencies: vec![],
            dev_dependencies: vec![],
            database: None,
            package_json: None,
            cargo_dependencies: None,
        };

        let framework = TestFramework::from_project(&info);
        assert_eq!(framework, TestFramework::RustBuiltIn);
    }

    #[test]
    fn test_framework_test_extension() {
        assert_eq!(TestFramework::Vitest.test_extension(), ".test.ts");
        assert_eq!(TestFramework::Jest.test_extension(), ".test.ts");
        assert_eq!(TestFramework::RustBuiltIn.test_extension(), ".rs");
        assert_eq!(TestFramework::Pytest.test_extension(), "_test.py");
        assert_eq!(TestFramework::GoTest.test_extension(), "_test.go");
    }

    #[test]
    fn test_options_default() {
        let options = TestGeneratorOptions::default();
        assert!(!options.unit_only);
        assert!(!options.integration_only);
        assert!(!options.e2e_only);
        assert!(options.skip_existing);
        assert!(options.include_mocks);
        assert_eq!(options.max_tests_per_function, 5);
        assert_eq!(options.target_coverage, 80.0);
    }

    #[test]
    fn test_generator_creation() {
        let generator = TestGenerator::new(PathBuf::from("/test/project"));
        assert_eq!(generator.project_root, PathBuf::from("/test/project"));
        assert!(!generator.dry_run);
    }

    #[test]
    fn test_generator_with_framework() {
        let generator = TestGenerator::new(PathBuf::from("/test/project"))
            .with_framework(TestFramework::Jest);
        assert_eq!(generator.framework, TestFramework::Jest);
    }

    #[test]
    fn test_generator_with_dry_run() {
        let generator = TestGenerator::new(PathBuf::from("/test/project"))
            .with_dry_run(true);
        assert!(generator.dry_run);
    }

    #[test]
    fn test_test_type_equality() {
        assert_eq!(TestType::Unit, TestType::Unit);
        assert_ne!(TestType::Unit, TestType::Integration);
        assert_ne!(TestType::Integration, TestType::E2E);
    }
}
