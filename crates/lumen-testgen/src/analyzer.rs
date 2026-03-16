//! Project analyzer for finding untested code

use crate::{code_parser, FunctionInfo};
use lumen_core::{Framework, Language, LumenResult, ProjectInfo};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Analyze a project and extract all functions
pub fn analyze_project(project_root: &Path, info: &ProjectInfo) -> LumenResult<Vec<FunctionInfo>> {
    let mut all_functions = Vec::new();

    // Find source files based on language and framework
    let source_files = find_source_files(project_root, info)?;

    tracing::info!("Found {} source files to analyze", source_files.len());

    for file_path in source_files {
        tracing::debug!("Analyzing file: {:?}", file_path);

        match code_parser::parse_file(&file_path, crate::TestFramework::from_project(info)) {
            Ok(functions) => {
                for func in functions {
                    // Filter out test functions and private implementations
                    if !is_test_function(&func) && should_include_function(&func, info) {
                        all_functions.push(func);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to parse file {:?}: {}", file_path, e);
            }
        }
    }

    // Identify untested functions by checking for existing test files
    let untested = identify_untested(&all_functions, project_root, info);

    Ok(untested)
}

/// Find all source files in the project
fn find_source_files(project_root: &Path, info: &ProjectInfo) -> LumenResult<Vec<PathBuf>> {
    let mut source_files = Vec::new();

    let source_dirs = get_source_directories(info);

    for dir in &source_dirs {
        let full_path = project_root.join(dir);
        if !full_path.exists() {
            continue;
        }

        let extensions = get_source_extensions(info.language.clone());

        for entry in WalkDir::new(&full_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if extensions.contains(&ext) {
                        // Skip test files
                        if !is_test_file(path) {
                            source_files.push(path.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    Ok(source_files)
}

/// Get source directories based on framework/language
fn get_source_directories(info: &ProjectInfo) -> Vec<String> {
    match info.language {
        Language::TypeScript | Language::JavaScript => {
            match info.framework {
                Framework::NextJs => vec!["app".to_string(), "components".to_string(), "lib".to_string(), "src".to_string()],
                Framework::Remix => vec!["app".to_string(), "routes".to_string()],
                Framework::NestJS => vec!["src".to_string()],
                _ => vec!["src".to_string(), "lib".to_string()],
            }
        }
        Language::Rust => vec!["src".to_string()],
        Language::Python => vec!["src".to_string()],
        Language::Go => vec![".".to_string()], // Go uses the current directory
        _ => vec!["src".to_string()],
    }
}

/// Get source file extensions for a language
fn get_source_extensions(language: Language) -> Vec<&'static str> {
    match language {
        Language::TypeScript => vec!["ts", "tsx", "mts", "cts"],
        Language::JavaScript => vec!["js", "jsx", "mjs", "cjs"],
        Language::Rust => vec!["rs"],
        Language::Python => vec!["py"],
        Language::Go => vec!["go"],
        _ => vec![],
    }
}

/// Check if a file is a test file
fn is_test_file(path: &Path) -> bool {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    file_name.contains(".test.")
        || file_name.contains(".spec.")
        || file_name.contains("_test.")
        || file_name.contains("_spec.")
        || file_name.starts_with("test_")
}

/// Check if a function is a test function
fn is_test_function(func: &FunctionInfo) -> bool {
    let name_lower = func.name.to_lowercase();

    name_lower.starts_with("test")
        || name_lower.starts_with("spec")
        || name_lower.contains("mock")
        || name_lower.contains("fixture")
        || name_lower.contains("describe")
}

/// Check if we should include this function in analysis
fn should_include_function(func: &FunctionInfo, info: &ProjectInfo) -> bool {
    // Skip private/internal functions for some languages
    match func.language {
        Language::Rust => {
            // Include public functions and tests (which are typically in the same module)
            func.visibility == crate::Visibility::Public || func.name.contains("test")
        }
        Language::Python => {
            // Include public (non-private) functions
            !func.name.starts_with('_') || func.name == "__init__"
        }
        Language::Go => {
            // Include exported (uppercase) functions
            matches!(func.visibility, crate::Visibility::Public)
        }
        _ => true, // Include all functions for TypeScript/JavaScript
    }
}

/// Identify functions that don't have tests yet
fn identify_untested(
    functions: &[FunctionInfo],
    project_root: &Path,
    info: &ProjectInfo,
) -> Vec<FunctionInfo> {
    let mut untested = Vec::new();
    let mut tested_functions = HashSet::new();

    // Find existing test files and extract tested function names
    if let Ok(test_files) = find_test_files(project_root, info) {
        for test_file in test_files {
            if let Ok(content) = std::fs::read_to_string(&test_file) {
                let tested = extract_tested_functions(&content, info.language.clone());
                tested_functions.extend(tested);
            }
        }
    }

    for func in functions {
        let function_key = format!("{}::{}", func.file_path.display(), func.name);

        if !tested_functions.contains(&function_key) {
            // Also check by name alone (tests might be in different files)
            let by_name = tested_functions.iter().any(|f| f.ends_with(&func.name));

            if !by_name {
                untested.push(func.clone());
            }
        }
    }

    untested
}

/// Find all test files in the project
fn find_test_files(project_root: &Path, info: &ProjectInfo) -> LumenResult<Vec<PathBuf>> {
    let mut test_files = Vec::new();

    let test_dirs = match info.language {
        Language::TypeScript | Language::JavaScript => {
            vec!["__tests__", "tests", "e2e"]
        }
        Language::Rust => vec!["tests"],
        Language::Python => vec!["tests", "test"],
        Language::Go => vec!["."], // Go tests are in the same directory
        _ => vec!["tests"],
    };

    for dir in &test_dirs {
        let full_path = project_root.join(dir);
        if !full_path.exists() {
            continue;
        }

        for entry in WalkDir::new(&full_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && is_test_file(path) {
                test_files.push(path.to_path_buf());
            }
        }
    }

    // Also check for co-located test files
    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && is_test_file(path) && !test_files.iter().any(|p| p.as_path() == path) {
            test_files.push(path.to_path_buf());
        }
    }

    Ok(test_files)
}

/// Extract function names that are tested from a test file
fn extract_tested_functions(content: &str, language: Language) -> HashSet<String> {
    let mut tested = HashSet::new();

    match language {
        Language::TypeScript | Language::JavaScript => {
            // Look for describe blocks and it/test calls
            if let Ok(re) = regex::Regex::new(r#"describe\(['"]([^'"]+)['"]"#) {
                for caps in re.captures_iter(content) {
                    if let Some(name) = caps.get(1) {
                        tested.insert(format!("::{}", name.as_str()));
                    }
                }
            }

            // Also look for imports
            if let Ok(re) = regex::Regex::new(r#"import\s+.*\s+from\s+['"]([^'"]+)['"]"#) {
                for caps in re.captures_iter(content) {
                    if let Some(path) = caps.get(1) {
                        tested.insert(path.as_str().to_string());
                    }
                }
            }
        }
        Language::Rust => {
            // Look for #[test] functions and their use super:: imports
            if let Ok(re) = regex::Regex::new(r#"use\s+super::([^;]+);"#) {
                for caps in re.captures_iter(content) {
                    if let Some(items) = caps.get(1) {
                        for item in items.as_str().split(',') {
                            tested.insert(format!("::{}", item.trim()));
                        }
                    }
                }
            }
        }
        Language::Python => {
            // Look for imports
            if let Ok(re) = regex::Regex::new(r#"from\s+([^\s]+)\s+import"#) {
                for caps in re.captures_iter(content) {
                    if let Some(module) = caps.get(1) {
                        tested.insert(module.as_str().to_string());
                    }
                }
            }
        }
        _ => {}
    }

    tested
}

/// Calculate test coverage gap
pub fn calculate_coverage_gap(
    all_functions: &[FunctionInfo],
    tested_functions: &HashSet<String>,
) -> (usize, usize, f64) {
    let total = all_functions.len();
    let tested = all_functions
        .iter()
        .filter(|f| {
            tested_functions
                .iter()
                .any(|t| t.ends_with(&f.name) || t.contains(&f.name))
        })
        .count();

    let untested = total - tested;
    let coverage = if total > 0 {
        (tested as f64 / total as f64) * 100.0
    } else {
        100.0
    };

    (tested, untested, coverage)
}

/// Group functions by module/path
pub fn group_by_module(functions: &[FunctionInfo]) -> HashMap<String, Vec<&FunctionInfo>> {
    let mut groups: HashMap<String, Vec<&FunctionInfo>> = HashMap::new();

    for func in functions {
        let module = func
            .file_path
            .strip_prefix("/").ok() // Remove leading slash if present
            .or_else(|| func.file_path.to_str().map(|s| s.as_ref()))
            .and_then(|p| Path::new(p).parent())
            .and_then(|p| p.to_str())
            .unwrap_or("root")
            .to_string();

        groups.entry(module).or_default().push(func);
    }

    groups
}

/// Prioritize functions for testing based on importance
pub fn prioritize_functions(functions: &[FunctionInfo]) -> Vec<&FunctionInfo> {
    let mut scored: Vec<_> = functions.iter().map(|f| (f, calculate_priority_score(f))).collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    scored.into_iter().map(|(f, _)| f).collect()
}

/// Calculate priority score for a function
fn calculate_priority_score(func: &FunctionInfo) -> f64 {
    let mut score = 0.0;

    // Public functions are higher priority
    if matches!(func.visibility, crate::Visibility::Public) {
        score += 10.0;
    }

    // API handlers are high priority
    let name_lower = func.name.to_lowercase();
    if name_lower.contains("handler")
        || name_lower.contains("controller")
        || name_lower.contains("route")
        || name_lower.contains("api")
    {
        score += 15.0;
    }

    // Complex functions (more parameters) are higher priority
    score += func.parameters.len() as f64 * 2.0;

    // Async functions might be more important
    if func.is_async {
        score += 3.0;
    }

    // Functions with return types are more testable
    if func.return_type.is_some() {
        score += 2.0;
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_test_file() {
        assert!(is_test_file(Path::new("test.test.ts")));
        assert!(is_test_file(Path::new("test.spec.ts")));
        assert!(is_test_file(Path::new("test_test.rs")));
        assert!(is_test_file(Path::new("test_test.go")));
        assert!(is_test_file(Path::new("test_component.py")));
        assert!(!is_test_file(Path::new("test.ts")));
        assert!(!is_test_file(Path::new("test.rs")));
    }

    #[test]
    fn test_is_test_function() {
        let func = FunctionInfo {
            name: "test_add".to_string(),
            file_path: PathBuf::from("/test.rs"),
            line: 1,
            signature: String::new(),
            parameters: vec![],
            return_type: None,
            is_async: false,
            visibility: Visibility::Public,
            language: Language::Rust,
            doc_comment: None,
            suggested_tests: vec![],
        };

        assert!(is_test_function(&func));

        let func2 = FunctionInfo {
            name: "add".to_string(),
            ..func.clone()
        };

        assert!(!is_test_function(&func2));
    }

    #[test]
    fn test_get_source_extensions() {
        assert_eq!(get_source_extensions(Language::TypeScript), vec!["ts", "tsx", "mts", "cts"]);
        assert_eq!(get_source_extensions(Language::Rust), vec!["rs"]);
        assert_eq!(get_source_extensions(Language::Python), vec!["py"]);
        assert_eq!(get_source_extensions(Language::Go), vec!["go"]);
    }

    #[test]
    fn test_calculate_coverage_gap() {
        let functions = vec![
            FunctionInfo {
                name: "func1".to_string(),
                file_path: PathBuf::from("/test.ts"),
                line: 1,
                signature: String::new(),
                parameters: vec![],
                return_type: None,
                is_async: false,
                visibility: Visibility::Public,
                language: Language::TypeScript,
                doc_comment: None,
                suggested_tests: vec![],
            },
            FunctionInfo {
                name: "func2".to_string(),
                file_path: PathBuf::from("/test.ts"),
                line: 2,
                signature: String::new(),
                parameters: vec![],
                return_type: None,
                is_async: false,
                visibility: Visibility::Public,
                language: Language::TypeScript,
                doc_comment: None,
                suggested_tests: vec![],
            },
        ];

        let tested: HashSet<String> = vec!["::func1".to_string()].into_iter().collect();

        let (tested_count, untested, coverage) = calculate_coverage_gap(&functions, &tested);
        assert_eq!(tested_count, 1);
        assert_eq!(untested, 1);
        assert_eq!(coverage, 50.0);
    }

    #[test]
    fn test_calculate_priority_score() {
        let func = FunctionInfo {
            name: "apiHandler".to_string(),
            file_path: PathBuf::from("/test.ts"),
            line: 1,
            signature: String::new(),
            parameters: vec![Parameter {
                name: "req".to_string(),
                param_type: Some("Request".to_string()),
                is_optional: false,
                default_value: None,
            }],
            return_type: Some("Response".to_string()),
            is_async: true,
            visibility: Visibility::Public,
            language: Language::TypeScript,
            doc_comment: None,
            suggested_tests: vec![],
        };

        let score = calculate_priority_score(&func);
        // Base: 10 (public) + 15 (handler) + 2 (param) + 3 (async) + 2 (return)
        assert!(score > 25.0);
    }

    use crate::Visibility;

    #[test]
    fn test_group_by_module() {
        let functions = vec![
            FunctionInfo {
                name: "func1".to_string(),
                file_path: PathBuf::from("/src/lib/a.ts"),
                line: 1,
                signature: String::new(),
                parameters: vec![],
                return_type: None,
                is_async: false,
                visibility: Visibility::Public,
                language: Language::TypeScript,
                doc_comment: None,
                suggested_tests: vec![],
            },
            FunctionInfo {
                name: "func2".to_string(),
                file_path: PathBuf::from("/src/lib/b.ts"),
                line: 1,
                signature: String::new(),
                parameters: vec![],
                return_type: None,
                is_async: false,
                visibility: Visibility::Public,
                language: Language::TypeScript,
                doc_comment: None,
                suggested_tests: vec![],
            },
        ];

        let groups = group_by_module(&functions);
        assert_eq!(groups.len(), 1);
        assert!(groups.contains_key("src/lib"));
        assert_eq!(groups["src/lib"].len(), 2);
    }
}
