//! Test templates for different languages and frameworks

pub mod typescript;
pub mod rust;
pub mod python;
pub mod go;

pub mod vitest;
pub mod nestjs;

use crate::{FunctionInfo, TestCase, TestCaseType, TestGeneratorOptions};

/// Generate test assertions based on return type
pub fn generate_assertions(return_type: &Option<String>, framework: &str) -> String {
    match return_type {
        Some(t) if t.is_empty() => String::new(),
        Some(t) if t.contains("Promise") || t.contains("Async") => {
            match framework {
                "vitest" | "jest" => "await expect(result).resolves.toBeDefined();".to_string(),
                _ => "assert!(result.is_ok());".to_string(),
            }
        }
        Some(t) if t.contains("void") => String::new(),
        Some(t) if t.contains("string") => {
            match framework {
                "vitest" | "jest" => "expect(typeof result).toBe('string');".to_string(),
                "rust" => "assert!(!result.is_empty());".to_string(),
                _ => "assert isinstance(result, str)".to_string(),
            }
        }
        Some(t) if t.contains("number") || t.contains("int") => {
            match framework {
                "vitest" | "jest" => "expect(typeof result).toBe('number');".to_string(),
                "rust" => "assert!(!result.is_nan());".to_string(),
                _ => "assert isinstance(result, (int, float))".to_string(),
            }
        }
        Some(t) if t.contains("boolean") => {
            match framework {
                "vitest" | "jest" => "expect(typeof result).toBe('boolean');".to_string(),
                "rust" => "assert!(result);".to_string(),
                _ => "assert isinstance(result, bool)".to_string(),
            }
        }
        Some(t) if t.contains("Array") || t.contains("[]") => {
            match framework {
                "vitest" | "jest" => "expect(Array.isArray(result)).toBe(true);".to_string(),
                "rust" => "assert!(!result.is_empty());".to_string(),
                _ => "assert isinstance(result, list)".to_string(),
            }
        }
        Some(t) if t.contains("Object") || t.contains("{") => {
            match framework {
                "vitest" | "jest" => "expect(typeof result).toBe('object');".to_string(),
                "rust" => "assert!(!result.is_empty());".to_string(),
                _ => "assert isinstance(result, dict)".to_string(),
            }
        }
        _ => match framework {
            "vitest" | "jest" => "expect(result).toBeDefined();".to_string(),
            "rust" => "assert!(result.is_ok());".to_string(),
            _ => "assert result is not None".to_string(),
        },
    }
}

/// Format test case description
pub fn format_test_description(case: &TestCase) -> String {
    let prefix = match case.case_type {
        TestCaseType::HappyPath => "should",
        TestCaseType::EdgeCase => "should handle",
        TestCaseType::ErrorCase => "should throw when",
        TestCaseType::Boundary => "should handle boundary",
    };

    format!("{} {}", prefix, case.description)
}

/// Generate mock setup code
pub fn generate_mock_setup(function: &FunctionInfo, options: &TestGeneratorOptions) -> String {
    if !options.include_mocks {
        return String::new();
    }

    let mut mocks = Vec::new();

    // Generate mocks for each parameter
    for param in &function.parameters {
        if let Some(ref param_type) = param.param_type {
            let mock = match param_type.as_str() {
                t if t.contains("Request") || t.contains("req") => {
                    format!("const mock{} = {{ method: 'GET', url: '/', headers: {{}} }};", param.name)
                }
                t if t.contains("Response") || t.contains("res") => {
                    format!(
                        "const mock{} = {{ status: 200, json: async (data: any) => JSON.stringify(data) }};",
                        param.name
                    )
                }
                t if t.contains("Service") || t.contains("Repository") => {
                    format!("const mock{} = {{ find: vi.fn().mockResolvedValue([]) }};", param.name)
                }
                _ => String::new(),
            };

            if !mock.is_empty() {
                mocks.push(mock);
            }
        }
    }

    if mocks.is_empty() {
        String::new()
    } else {
        format!("\n    {}\n    ", mocks.join("\n    "))
    }
}

/// Generate test imports
pub fn generate_test_imports(function: &FunctionInfo, framework: &str) -> String {
    match framework {
        "vitest" => {
            format!(
                "import {{ describe, it, expect, vi }} from 'vitest';\nimport {{ {} }} from './{}';\n",
                function.name,
                function
                    .file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("module")
            )
        }
        "jest" => {
            format!(
                "import {{ {} }} from './{}';\n",
                function.name,
                function
                    .file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("module")
            )
        }
        "rust" => String::new(),
        "pytest" => {
            format!(
                "from {} import {}\n",
                function
                    .file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("module")
                    .replace('/', "."),
                function.name
            )
        }
        _ => String::new(),
    }
}

/// Generate test file header
pub fn generate_test_header(function: &FunctionInfo, framework: &str) -> String {
    match framework {
        "vitest" | "jest" => {
            format!(
                "// Auto-generated test file for {}\n// Generated by Lumen v{}\n\n",
                function.name,
                env!("CARGO_PKG_VERSION")
            )
        }
        "rust" => {
            format!(
                "// Auto-generated tests for {}\n// Generated by Lumen v{}\n\n",
                function.name,
                env!("CARGO_PKG_VERSION")
            )
        }
        "pytest" => {
            format!(
                "# Auto-generated test file for {}\n# Generated by Lumen v{}\n\n",
                function.name,
                env!("CARGO_PKG_VERSION")
            )
        }
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Parameter, TestFramework};
    use lumen_core::Language;
    use std::path::PathBuf;

    #[test]
    fn test_generate_assertions_typescript() {
        assert_eq!(
            generate_assertions(&Some("string".to_string()), "vitest"),
            "expect(typeof result).toBe('string');"
        );
        assert_eq!(
            generate_assertions(&Some("number".to_string()), "vitest"),
            "expect(typeof result).toBe('number');"
        );
        assert_eq!(
            generate_assertions(&Some("boolean".to_string()), "vitest"),
            "expect(typeof result).toBe('boolean');"
        );
        assert_eq!(
            generate_assertions(&Some("Array<any>".to_string()), "vitest"),
            "expect(Array.isArray(result)).toBe(true);"
        );
    }

    #[test]
    fn test_generate_assertions_rust() {
        assert_eq!(generate_assertions(&Some("String".to_string()), "rust"), "assert!(!result.is_empty());");
        assert_eq!(generate_assertions(&Some("i32".to_string()), "rust"), "assert!(!result.is_nan());");
        assert_eq!(generate_assertions(&Some("bool".to_string()), "rust"), "assert!(result);");
        assert_eq!(generate_assertions(&Some("Vec<T>".to_string()), "rust"), "assert!(!result.is_empty());");
    }

    #[test]
    fn test_format_test_description() {
        let case = TestCase {
            description: "return correct value".to_string(),
            case_type: TestCaseType::HappyPath,
            inputs: vec![],
            expected_output: None,
        };

        assert_eq!(format_test_description(&case), "should return correct value");

        let case2 = TestCase {
            description: "empty input".to_string(),
            case_type: TestCaseType::EdgeCase,
            inputs: vec![],
            expected_output: None,
        };

        assert_eq!(format_test_description(&case2), "should handle empty input");
    }

    #[test]
    fn test_generate_mock_setup() {
        let function = FunctionInfo {
            name: "handler".to_string(),
            file_path: PathBuf::from("/test.ts"),
            line: 1,
            signature: String::new(),
            parameters: vec![
                Parameter {
                    name: "req".to_string(),
                    param_type: Some("Request".to_string()),
                    is_optional: false,
                    default_value: None,
                },
                Parameter {
                    name: "res".to_string(),
                    param_type: Some("Response".to_string()),
                    is_optional: false,
                    default_value: None,
                },
            ],
            return_type: Some("Promise<Response>".to_string()),
            is_async: true,
            visibility: crate::Visibility::Public,
            language: Language::TypeScript,
            doc_comment: None,
            suggested_tests: vec![],
        };

        let options = TestGeneratorOptions::default();
        let mocks = generate_mock_setup(&function, &options);

        assert!(mocks.contains("mockReq"));
        assert!(mocks.contains("mockRes"));
    }

    #[test]
    fn test_generate_test_imports() {
        let function = FunctionInfo {
            name: "add".to_string(),
            file_path: PathBuf::from("/src/math.ts"),
            line: 1,
            signature: String::new(),
            parameters: vec![],
            return_type: None,
            is_async: false,
            visibility: crate::Visibility::Public,
            language: Language::TypeScript,
            doc_comment: None,
            suggested_tests: vec![],
        };

        let imports = generate_test_imports(&function, "vitest");
        assert!(imports.contains("import"));
        assert!(imports.contains("add"));
        assert!(imports.contains("math"));
    }

    #[test]
    fn test_generate_test_header() {
        let function = FunctionInfo {
            name: "myFunction".to_string(),
            file_path: PathBuf::from("/test.ts"),
            line: 1,
            signature: String::new(),
            parameters: vec![],
            return_type: None,
            is_async: false,
            visibility: crate::Visibility::Public,
            language: Language::TypeScript,
            doc_comment: None,
            suggested_tests: vec![],
        };

        let header = generate_test_header(&function, "vitest");
        assert!(header.contains("myFunction"));
        assert!(header.contains("Auto-generated"));
        assert!(header.contains("Lumen"));
    }
}
