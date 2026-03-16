//! Code parsing utilities for extracting function information

use crate::{FunctionInfo, Language, Parameter, TestCase, TestCaseType, TestValue, Visibility};
use lumen_core::{LumenError, LumenResult};
use regex::Regex;
use std::path::{Path, PathBuf};

/// Parse Go file for functions (defined early to avoid compiler issues)
pub fn parse_go_file_early(file_path: &Path, content: &str) -> LumenResult<Vec<FunctionInfo>> {
    parse_go_file_impl(file_path, content)
}

/// Internal implementation of Go file parsing
fn parse_go_file_impl(file_path: &Path, content: &str) -> LumenResult<Vec<FunctionInfo>> {
    let mut functions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Pattern for Go function definitions
    let patterns = [
        // func (receiver) name(params) (return_types)
        r#"func\s+\(([^)]+)\)\s+(\w+)\s*\(([^)]*)\)\s*(?:\(([^)]*)\)|([^{:]*))?\s*\{"#,
        // func name(params) (return_types)
        r#"func\s+(\w+)\s*\(([^)]*)\)\s*(?:\(([^)]*)\)|([^{:]*))?\s*\{"#,
    ];

    for (line_idx, line) in lines.iter().enumerate() {
        if line.trim().starts_with("//") {
            continue;
        }

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(line) {
                    // Determine which capture group has the function name
                    let name = if pattern.contains("(receiver)") {
                        // Pattern with receiver: func (r) Name(...)
                        caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default()
                    } else {
                        // Pattern without receiver: func Name(...)
                        caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default()
                    };

                    if name.is_empty() {
                        continue;
                    }

                    let params_str = if pattern.contains("(receiver)") {
                        caps.get(3).map(|m| m.as_str()).unwrap_or("")
                    } else {
                        caps.get(2).map(|m| m.as_str()).unwrap_or("")
                    };

                    let return_type = if pattern.contains("(receiver)") {
                        caps.get(4).or_else(|| caps.get(5)).map(|m| m.as_str().trim().to_string())
                    } else {
                        caps.get(3).or_else(|| caps.get(4)).map(|m| m.as_str().trim().to_string())
                    };

                    let parameters = parse_go_params(params_str);
                    let visibility = if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };

                    functions.push(FunctionInfo {
                        name,
                        file_path: file_path.to_path_buf(),
                        line: line_idx + 1,
                        signature: line.trim().to_string(),
                        parameters,
                        return_type: if return_type.is_some() && !return_type.as_ref().unwrap().is_empty() {
                            return_type
                        } else {
                            None
                        },
                        is_async: false, // Go uses goroutines, not async/await
                        visibility,
                        language: Language::Go,
                        doc_comment: extract_doc_comment(&lines, line_idx),
                        suggested_tests: vec![],
                    });
                }
            }
        }
    }

    Ok(functions)
}

/// Parse Go function parameters
fn parse_go_params(params_str: &str) -> Vec<Parameter> {
    let mut params = Vec::new();

    for param in params_str.split(',') {
        let param = param.trim();
        if param.is_empty() {
            continue;
        }

        // Go uses Type name or name Type
        let parts: Vec<&str> = param.split_whitespace().collect();
        if parts.len() >= 2 {
            // name Type
            let name = parts[0].to_string();
            let param_type = parts[1..].join(" ");
            params.push(Parameter {
                name,
                param_type: Some(param_type),
                is_optional: false,
                default_value: None,
            });
        } else if parts.len() == 1 {
            // Just the type (unnamed parameter)
            params.push(Parameter {
                name: format!("param{}", params.len()),
                param_type: Some(parts[0].to_string()),
                is_optional: false,
                default_value: None,
            });
        }
    }

    params
}

/// Parse a source file and extract function information
pub fn parse_file(file_path: &Path, framework: crate::TestFramework) -> LumenResult<Vec<FunctionInfo>> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| LumenError::FileNotFound(file_path.to_path_buf()))?;

    let language = detect_language_from_path(file_path);

    match language {
        Language::TypeScript | Language::JavaScript => {
            parse_typescript_file(file_path, &content)
        }
        Language::Rust => parse_rust_file(file_path, &content),
        Language::Python => parse_python_file(file_path, &content),
        Language::Go => parse_go_file_early(file_path, &content),
        _ => Ok(vec![]),
    }
}

/// Detect language from file path
fn detect_language_from_path(path: &Path) -> Language {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext {
            "ts" | "mts" | "cts" => Language::TypeScript,
            "js" | "mjs" | "cjs" => Language::JavaScript,
            "rs" => Language::Rust,
            "py" => Language::Python,
            "go" => Language::Go,
            _ => Language::Unknown,
        })
        .unwrap_or(Language::Unknown)
}

/// Parse TypeScript/JavaScript file for functions
fn parse_typescript_file(file_path: &Path, content: &str) -> LumenResult<Vec<FunctionInfo>> {
    let mut functions = Vec::new();

    // Regex patterns for different function declarations
    let patterns = [
        // Named function: function name(params): ReturnType {}
        (r#"function\s+(\w+)\s*\(([^)]*)\)(?:\s*:\s*([^{]+))?\s*\{"#, false),
        // Async function: async function name(params): ReturnType {}
        (r#"async\s+function\s+(\w+)\s*\(([^)]*)\)(?:\s*:\s*([^{]+))?\s*\{"#, true),
        // Arrow function assigned to const: const name = (params): ReturnType => {}
        (r#"const\s+(\w+)\s*=\s*(?:async\s+)?\(([^)]*)\)(?:\s*:\s*([^=]+))?\s*=>"#, false),
        // Exported function: export function name(params)
        (r#"export\s+(?:async\s+)?function\s+(\w+)\s*\(([^)]*)\)(?:\s*:\s*([^{]+))?\s*\{"#, false),
        // Method declaration: name(params): ReturnType {
        (r#"(\w+)\s*\(([^)]*)\)(?:\s*:\s*([^{]+))?\s*\{"#, false),
    ];

    let lines: Vec<&str> = content.lines().collect();

    for (line_idx, line) in lines.iter().enumerate() {
        // Skip commented lines
        if line.trim().starts_with("//") || line.trim().starts_with("*") {
            continue;
        }

        for (pattern, is_async) in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(line) {
                    let name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                    let params_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                    let return_type = caps.get(3).map(|m| m.as_str().trim().to_string());

                    // Skip if this is a React component (starts with uppercase)
                    if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false)
                        && !params_str.contains("props")
                    {
                        continue;
                    }

                    let parameters = parse_typescript_params(params_str);
                    let visibility = if line.contains("export") || line.contains("public") {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };

                    // Extract doc comment if present
                    let doc_comment = extract_doc_comment(&lines, line_idx);

                    functions.push(FunctionInfo {
                        name: name.clone(),
                        file_path: file_path.to_path_buf(),
                        line: line_idx + 1,
                        signature: line.trim().to_string(),
                        parameters: parameters.clone(),
                        return_type: return_type.clone(),
                        is_async: *is_async || line.contains("async"),
                        visibility,
                        language: Language::TypeScript,
                        doc_comment,
                        suggested_tests: generate_suggested_tests(&name, &parameters, &return_type),
                    });
                }
            }
        }
    }

    Ok(functions)
}

/// Parse TypeScript function parameters
fn parse_typescript_params(params_str: &str) -> Vec<Parameter> {
    let mut params = Vec::new();

    for param in params_str.split(',') {
        let param = param.trim();
        if param.is_empty() {
            continue;
        }

        // Parse parameter with optional type: name: Type or name?: Type
        let parts: Vec<&str> = param.split(':').collect();
        let name = parts.get(0).map(|s| s.trim().trim_start_matches("...").to_string()).unwrap_or_default();
        let param_type = parts.get(1).map(|s| s.trim().to_string());
        let is_optional = param.contains("?") || param.contains("undefined");

        params.push(Parameter {
            name,
            param_type,
            is_optional,
            default_value: None,
        });
    }

    params
}

/// Extract doc comment from lines above the function
fn extract_doc_comment(lines: &[&str], line_idx: usize) -> Option<String> {
    let mut comment_lines = Vec::new();

    // Look backwards from the function line
    let mut i = line_idx.saturating_sub(1);
    while i > 0 && i < lines.len() {
        let line = lines[i].trim();

        if line.starts_with("/**") || line.starts_with("/*") {
            comment_lines.push(line.trim_start_matches("/**").trim_start_matches("/*").to_string());
            i = i.saturating_sub(1);
            continue;
        }

        if line.starts_with("*") {
            comment_lines.push(line.trim_start_matches('*').trim().to_string());
            i = i.saturating_sub(1);
            continue;
        }

        if line.starts_with("//") {
            comment_lines.push(line.trim_start_matches("//").trim().to_string());
            i = i.saturating_sub(1);
            continue;
        }

        // Stop if we hit a blank line or another statement
        if line.is_empty() || line.ends_with('{') || line.ends_with(';') {
            break;
        }

        i = i.saturating_sub(1);
    }

    if comment_lines.is_empty() {
        None
    } else {
        comment_lines.reverse();
        Some(comment_lines.join("\n"))
    }
}

/// Generate suggested test cases based on function signature
fn generate_suggested_tests(
    name: &str,
    params: &[Parameter],
    return_type: &Option<String>,
) -> Vec<TestCase> {
    let mut tests = Vec::new();

    // Happy path test
    tests.push(TestCase {
        description: format!("should return expected result for valid {}", name),
        case_type: TestCaseType::HappyPath,
        inputs: params
            .iter()
            .map(|p| TestValue {
                name: p.name.clone(),
                value: generate_mock_value(&p.param_type),
                value_type: p.param_type.clone().unwrap_or_else(|| "unknown".to_string()),
            })
            .collect(),
        expected_output: return_type.clone(),
    });

    // Edge case: empty parameters
    if !params.is_empty() {
        tests.push(TestCase {
            description: format!("should handle empty {} input", params.first().unwrap().name),
            case_type: TestCaseType::EdgeCase,
            inputs: params
                .iter()
                .map(|p| TestValue {
                    name: p.name.clone(),
                    value: generate_empty_value(&p.param_type),
                    value_type: p.param_type.clone().unwrap_or_else(|| "unknown".to_string()),
                })
                .collect(),
            expected_output: return_type.clone(),
        });
    }

    // Error case: null/undefined for optional params
    if params.iter().any(|p| p.is_optional) {
        tests.push(TestCase {
            description: format!("should handle undefined optional parameters"),
            case_type: TestCaseType::EdgeCase,
            inputs: params
                .iter()
                .map(|p| TestValue {
                    name: p.name.clone(),
                    value: if p.is_optional {
                        "undefined".to_string()
                    } else {
                        generate_mock_value(&p.param_type)
                    },
                    value_type: p.param_type.clone().unwrap_or_else(|| "unknown".to_string()),
                })
                .collect(),
            expected_output: return_type.clone(),
        });
    }

    tests
}

/// Generate mock value based on type
fn generate_mock_value(type_str: &Option<String>) -> String {
    match type_str {
        Some(t) if t.contains("string") => "\"test\"".to_string(),
        Some(t) if t.contains("number") | t.contains("int") | t.contains("float") => "42".to_string(),
        Some(t) if t.contains("boolean") | t.contains("bool") => "true".to_string(),
        Some(t) if t.contains("Array") | t.contains("[]") => "[]".to_string(),
        Some(t) if t.contains("Object") | t.contains("{") => "{}".to_string(),
        Some(t) if t.contains("Date") => "new Date()".to_string(),
        Some(t) if t.contains("Promise") | t.contains("Async") => "Promise.resolve()".to_string(),
        _ => "\"mock\"".to_string(),
    }
}

/// Generate empty value based on type
fn generate_empty_value(type_str: &Option<String>) -> String {
    match type_str {
        Some(t) if t.contains("string") => "\"\"".to_string(),
        Some(t) if t.contains("number") | t.contains("int") | t.contains("float") => "0".to_string(),
        Some(t) if t.contains("boolean") | t.contains("bool") => "false".to_string(),
        Some(t) if t.contains("Array") | t.contains("[]") => "[]".to_string(),
        Some(t) if t.contains("Object") | t.contains("{") => "{}".to_string(),
        _ => "null".to_string(),
    }
}

/// Parse Rust file for functions
fn parse_rust_file(file_path: &Path, content: &str) -> LumenResult<Vec<FunctionInfo>> {
    let mut functions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Patterns for Rust function declarations
    let patterns = [
        // pub async fn name(params) -> ReturnType
        (r#"pub\s+async\s+fn\s+(\w+)\s*<([^>]*)>\s*\(([^)]*)\)(?:\s*->\s*([^,{]+))?"#, true, true),
        (r#"pub\s+async\s+fn\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^,{]+))?"#, false, true),
        // pub fn name(params) -> ReturnType
        (r#"pub\s+fn\s+(\w+)\s*<([^>]*)>\s*\(([^)]*)\)(?:\s*->\s*([^,{]+))?"#, true, false),
        (r#"pub\s+fn\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^,{]+))?"#, false, false),
        // Private functions (no pub)
        (r#"fn\s+(\w+)\s*<([^>]*)>\s*\(([^)]*)\)(?:\s*->\s*([^,{]+))?"#, true, false),
        (r#"fn\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^,{]+))?"#, false, false),
    ];

    for (line_idx, line) in lines.iter().enumerate() {
        // Skip test modules
        if line.contains("#[cfg(test)]") || line.trim().starts_with("//") {
            continue;
        }

        for has_generics in [true, false] {
            for is_async in [true, false] {
            let pattern = if has_generics {
                if is_async {
                    r#"pub\s+async\s+fn\s+(\w+)\s*<([^>]*)>\s*\(([^)]*)\)(?:\s*->\s*([^,{]+))?"#
                } else {
                    r#"pub\s+fn\s+(\w+)\s*<([^>]*)>\s*\(([^)]*)\)(?:\s*->\s*([^,{]+))?"#
                }
            } else if is_async {
                r#"pub\s+async\s+fn\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^,{]+))?"#
            } else {
                r#"pub\s+fn\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^,{]+))?"#
            };

            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(line) {
                    let name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                    let params_str = if has_generics {
                        caps.get(3).map(|m| m.as_str()).unwrap_or("")
                    } else {
                        caps.get(2).map(|m| m.as_str()).unwrap_or("")
                    };
                    let return_type = if has_generics {
                        caps.get(4).map(|m| m.as_str().trim().to_string())
                    } else {
                        caps.get(3).map(|m| m.as_str().trim().to_string())
                    };

                    let parameters = parse_rust_params(params_str);
                    let visibility = if line.contains("pub") {
                        Visibility::Public
                    } else {
                        Visibility::Private
                    };

                    let doc_comment = extract_doc_comment(&lines, line_idx);

                    functions.push(FunctionInfo {
                        name,
                        file_path: file_path.to_path_buf(),
                        line: line_idx + 1,
                        signature: line.trim().to_string(),
                        parameters,
                        return_type,
                        is_async,
                        visibility,
                        language: Language::Rust,
                        doc_comment,
                        suggested_tests: vec![],
                    });
                }
            }
        }
        }  // Close for is_async
    }  // Close for has_generics

    Ok(functions)
}

/// Parse Rust function parameters
fn parse_rust_params(params_str: &str) -> Vec<Parameter> {
    let mut params = Vec::new();

    for param in params_str.split(',') {
        let param = param.trim();
        if param.is_empty() || param.starts_with('&') {
            continue;
        }

        // Parse: name: Type or mut name: Type or &self or &mut self
        let parts: Vec<&str> = param.split(':').collect();
        let name = parts
            .get(0)
            .map(|s| s.trim().trim_start_matches("mut").trim().to_string())
            .unwrap_or_default();

        if name == "self" || name == "&self" || name == "&mut self" {
            continue;
        }

        let param_type = parts.get(1).map(|s| s.trim().to_string());
        let is_optional = param_type.as_ref().map_or(false, |t| t.contains("Option"));

        params.push(Parameter {
            name,
            param_type,
            is_optional,
            default_value: None,
        });
    }

    params
}

/// Parse Python file for functions
fn parse_python_file(file_path: &Path, content: &str) -> LumenResult<Vec<FunctionInfo>> {
    let mut functions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Patterns for Python function definitions
    let patterns = [
        // async def name(params) -> ReturnType:
        r"async\s+def\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^:]+))?:",
        // def name(params) -> ReturnType:
        r"def\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([^:]+))?:",
    ];

    for (line_idx, line) in lines.iter().enumerate() {
        // Skip comments and inside strings
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with('\'') || trimmed.starts_with('"') {
            continue;
        }

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(line) {
                    let name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                    let params_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                    let return_type = caps.get(3).map(|m| m.as_str().trim().to_string());

                    // Skip if this is a test function
                    if name.starts_with("test_") {
                        continue;
                    }

                    let parameters = parse_python_params(params_str);
                    let is_async = line.contains("async def");
                    let visibility = if name.starts_with('_') {
                        Visibility::Private
                    } else {
                        Visibility::Public
                    };

                    let doc_comment = extract_python_docstring(&lines, line_idx);

                    functions.push(FunctionInfo {
                        name,
                        file_path: file_path.to_path_buf(),
                        line: line_idx + 1,
                        signature: line.trim().to_string(),
                        parameters,
                        return_type,
                        is_async,
                        visibility,
                        language: Language::Python,
                        doc_comment,
                        suggested_tests: vec![],
                    });
                }
            }
        }
    }

    Ok(functions)
}

/// Parse Python function parameters
fn parse_python_params(params_str: &str) -> Vec<Parameter> {
    let mut params = Vec::new();

    for param in params_str.split(',') {
        let param = param.trim();
        if param.is_empty() {
            continue;
        }

        // Parse: name: Type = default or name = default or name: Type
        let param = param.trim_start_matches('*').trim_start_matches("**");

        if let Some(colon_pos) = param.find(':') {
            let name = param[..colon_pos].trim().to_string();
            let type_part = &param[colon_pos + 1..];

            // Check for default value
            let (param_type, default_value) = if let Some(eq_pos) = type_part.find('=') {
                (type_part[..eq_pos].trim().to_string(), Some(type_part[eq_pos + 1..].trim().to_string()))
            } else {
                (type_part.trim().to_string(), None)
            };

            params.push(Parameter {
                name,
                param_type: Some(param_type),
                is_optional: default_value.is_some(),
                default_value,
            });
        } else if let Some(eq_pos) = param.find('=') {
            let name = param[..eq_pos].trim().to_string();
            let default_value = Some(param[eq_pos + 1..].trim().to_string());
            params.push(Parameter {
                name,
                param_type: None,
                is_optional: true,
                default_value,
            });
        } else {
            params.push(Parameter {
                name: param.to_string(),
                param_type: None,
                is_optional: false,
                default_value: None,
            });
        }
    }

    params
}

/// Extract Python docstring
fn extract_python_docstring(lines: &[&str], line_idx: usize) -> Option<String> {
    // Look for docstring after function definition
    if let Some(next_line) = lines.get(line_idx + 1) {
        let trimmed = next_line.trim();
        if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
            let mut doc_lines = vec![trimmed.trim_start_matches("\"\"\"").trim_start_matches("'''").to_string()];
            let mut i = line_idx + 2;
            while i < lines.len() {
                let line = lines[i].trim();
                if line.contains("\"\"\"") || line.contains("'''") {
                    doc_lines.push(line.replace("\"\"\"", "").replace("'''", ""));
                    break;
                }
                doc_lines.push(line.to_string());
                i += 1;
            }
            return Some(doc_lines.join("\n"));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_language_from_path() {
        assert_eq!(detect_language_from_path(Path::new("test.ts")), Language::TypeScript);
        assert_eq!(detect_language_from_path(Path::new("test.js")), Language::JavaScript);
        assert_eq!(detect_language_from_path(Path::new("test.rs")), Language::Rust);
        assert_eq!(detect_language_from_path(Path::new("test.py")), Language::Python);
        assert_eq!(detect_language_from_path(Path::new("test.go")), Language::Go);
        assert_eq!(detect_language_from_path(Path::new("test.txt")), Language::Unknown);
    }

    #[test]
    fn test_parse_typescript_function() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.ts");
        let content = "export function add(a: number, b: number): number {
    return a + b;
}
";
        fs::write(&file_path, content).unwrap();

        let functions = parse_typescript_file(&file_path, &fs::read_to_string(&file_path).unwrap()).unwrap();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "add");
        assert_eq!(functions[0].parameters.len(), 2);
        assert_eq!(functions[0].parameters[0].name, "a");
        assert_eq!(functions[0].parameters[1].name, "b");
    }

    #[test]
    fn test_parse_rust_function() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.rs");
        let content = "pub fn greet(name: &str) -> String {
    format!(\"Hello, {}\", name)
}
";
        fs::write(&file_path, content).unwrap();

        let functions = parse_rust_file(&file_path, &fs::read_to_string(&file_path).unwrap()).unwrap();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "greet");
        assert_eq!(functions[0].parameters.len(), 1);
        assert_eq!(functions[0].parameters[0].name, "name");
    }

    #[test]
    fn test_generate_mock_value() {
        assert_eq!(generate_mock_value(&Some("string".to_string())), "\"test\"");
        assert_eq!(generate_mock_value(&Some("number".to_string())), "42");
        assert_eq!(generate_mock_value(&Some("boolean".to_string())), "true");
        assert_eq!(generate_mock_value(&Some("Array<any>".to_string())), "[]");
        assert_eq!(generate_mock_value(&Some("Object".to_string())), "{}");
    }
}
