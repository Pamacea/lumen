//! TypeScript test templates (Vitest, Jest, Mocha)

use crate::{
    FunctionInfo, TestCase, TestCaseType, TestFramework, TestGeneratorOptions,
};
use std::path::PathBuf;

/// Generate unit test for a TypeScript function
pub fn generate_unit_test(function: &FunctionInfo, framework: TestFramework, options: &TestGeneratorOptions) -> String {
    let mut output = String::new();

    // Header
    output.push_str("// Tests for ");
    output.push_str(&function.name);
    output.push_str("\n");
    output.push_str(&format!("// Framework: {}\n\n", framework_name(framework)));

    // Imports
    match framework {
        TestFramework::Vitest => {
            output.push_str("import { describe, it, expect } from 'vitest';\n");
        }
        TestFramework::Jest => {
            output.push_str("import { describe, it, expect } from '@jest/globals';\n");
        }
        TestFramework::Mocha => {
            output.push_str("import { expect } from 'chai';\n");
        }
        _ => {
            output.push_str("// TODO: Add imports\n");
        }
    }
    output.push('\n');

    // Describe block
    output.push_str(&generate_describe_block(function, framework, options));

    output
}

/// Generate integration test for API handlers
pub fn generate_integration_test(function: &FunctionInfo, framework: TestFramework) -> String {
    let mut output = String::new();

    output.push_str("// Integration test for API handler\n");
    output.push_str(&format!("// File: {}\n\n", function.file_path.display()));

    // Framework-specific imports
    match framework {
        TestFramework::Vitest => {
            output.push_str("import { describe, it, expect, beforeAll, afterAll } from 'vitest';\n");
            output.push_str("import { app } from '../app'; // Adjust import path\n");
        }
        TestFramework::Jest => {
            output.push_str("import { describe, it, expect, beforeAll, afterAll } from '@jest/globals';\n");
            output.push_str("import { app } from '../app'; // Adjust import path\n");
        }
        _ => {}
    }

    output.push_str(
        "import { createMocks } from 'node-mocks-http'; // or your preferred testing library\n\n",
    );

    output.push_str(&format!("describe('{} Integration', () => {{\n", function.name));

    // Setup
    output.push_str("  let server: any;\n\n");
    output.push_str("  beforeAll(async () => {\n");
    output.push_str("    // Setup test server/database\n");
    output.push_str("  });\n\n");
    output.push_str("  afterAll(async () => {\n");
    output.push_str("    // Cleanup\n");
    output.push_str("  });\n\n");

    // Happy path test
    output.push_str("  it('should handle valid request', async () => {\n");
    output.push_str(&generate_request_mock(function, true));
    output.push_str(&generate_response_assertions(true));
    output.push_str("  });\n\n");

    // Error case test
    output.push_str("  it('should handle invalid request', async () => {\n");
    output.push_str(&generate_request_mock(function, false));
    output.push_str(&generate_response_assertions(false));
    output.push_str("  });\n");

    output.push_str("});\n");

    output
}

/// Generate E2E test for critical flows
pub fn generate_e2e_test(function: &FunctionInfo, framework: TestFramework) -> String {
    let mut output = String::new();

    output.push_str("// E2E test for critical user flow\n");
    output.push_str(&format!("// Testing: {}\n\n", function.name));

    match framework {
        TestFramework::Vitest => {
            output.push_str("import { describe, it, expect } from 'vitest';\n");
            output.push_str("import { testApi } from '../support/api'; // Adjust import\n\n");
        }
        TestFramework::Jest => {
            output.push_str("import { describe, it, expect } from '@jest/globals';\n");
            output.push_str("import { testApi } from '../support/api'; // Adjust import\n\n");
        }
        _ => {}
    }

    output.push_str(&format!("describe('{} E2E', () => {{\n", function.name));

    // E2E test scenario
    output.push_str("  it('should complete full user flow', async () => {\n");
    output.push_str("    // Step 1: Navigate to resource\n");
    output.push_str(&format!("    const response1 = await testApi.{}({{ /* input */ }});\n", function.name));
    output.push_str("    expect(response1.status).toBe(200);\n\n");
    output.push_str("    // Step 2: Verify side effects\n");
    output.push_str("    const response2 = await testApi.verify();\n");
    output.push_str("    expect(response2.data).toBeDefined();\n");
    output.push_str("  });\n");

    output.push_str("});\n");

    output
}

/// Generate a single test for a function
pub fn generate_function_test(function: &FunctionInfo, framework: TestFramework) -> String {
    let mut output = String::new();

    output.push_str(&format!("// Test for {}\n", function.name));

    // Generate test cases
    for case in &function.suggested_tests {
        output.push_str(&generate_single_test_case(function, case, framework));
    }

    output
}

/// Generate describe block with test cases
fn generate_describe_block(function: &FunctionInfo, framework: TestFramework, options: &TestGeneratorOptions) -> String {
    let mut output = String::new();

    output.push_str(&format!("describe('{}', () => {{\n", function.name));

    // Generate suggested tests
    for (idx, case) in function.suggested_tests.iter().enumerate() {
        if idx >= options.max_tests_per_function {
            break;
        }

        output.push_str(&generate_test_case(function, case, framework));
    }

    // Add error case if function can throw
    if can_throw_error(function) {
        output.push_str(&generate_error_test_case(function, framework));
    }

    output.push_str("});\n");

    output
}

/// Generate a single test case
fn generate_test_case(function: &FunctionInfo, case: &TestCase, framework: TestFramework) -> String {
    let mut output = String::new();

    let test_fn = if function.is_async { "it(" } else { "it(" };
    let async_kw = if function.is_async { "async " } else { "" };

    output.push_str(&format!("  {}'{}', {}() => {{\n", test_fn, case.description, async_kw));

    // Setup
    if !case.inputs.is_empty() {
        output.push_str("    // Arrange\n");
        for input in &case.inputs {
            output.push_str(&format!("    const {} = {};\n", input.name, input.value));
        }
        output.push_str("\n");
    }

    // Act
    output.push_str("    // Act\n");
    let call = generate_function_call(function);
    if function.is_async {
        output.push_str(&format!("    const result = await {};\n", call));
    } else {
        output.push_str(&format!("    const result = {};\n", call));
    }
    output.push('\n');

    // Assert
    output.push_str("    // Assert\n");
    let assertions = generate_ts_assertions(&function.return_type);
    if !assertions.is_empty() {
        output.push_str(&format!("    {}\n", assertions));
    } else {
        output.push_str("    expect(result).toBeDefined();\n");
    }

    output.push_str("  });\n\n");

    output
}

/// Generate error test case
fn generate_error_test_case(function: &FunctionInfo, _framework: TestFramework) -> String {
    let async_kw = if function.is_async { "async " } else { "" };
    let params_part = if function.parameters.is_empty() {
        String::new()
    } else {
        format!("(invalidInput)")
    };

    format!(
        r#"  {}('should throw on invalid input', {}() => {{
    // Arrange
    const invalidInput = null;

    // Act & Assert
    expect(async () => await {}{}()).toThrow();
  }});

"#,
        "it",
        async_kw,
        function.name,
        params_part
    )
}

/// Generate a single test case (standalone)
fn generate_single_test_case(function: &FunctionInfo, case: &TestCase, framework: TestFramework) -> String {
    let async_kw = if function.is_async { "async " } else { "" };

    let mut output = format!("it('{}', {}() => {{\n", case.description, async_kw);

    // Generate input setup
    for input in &case.inputs {
        output.push_str(&format!("  const {} = {};\n", input.name, input.value));
    }

    // Function call
    let call_args = case
        .inputs
        .iter()
        .map(|i| i.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    output.push_str(&format!("  const result = {}{}{};\n", if function.is_async { "await " } else { "" }, function.name, if call_args.is_empty() { String::new() } else { format!("({})", call_args) }));

    // Assertion
    output.push_str("  expect(result).toBeDefined();\n");
    output.push_str("});\n\n");

    output
}

/// Generate function call with parameters
fn generate_function_call(function: &FunctionInfo) -> String {
    let args: Vec<String> = function
        .parameters
        .iter()
        .map(|p| {
            if p.param_type.as_ref().map_or(false, |t| t.contains("...")) {
                format!("...{}", p.name)
            } else {
                p.name.clone()
            }
        })
        .collect();

    if args.is_empty() {
        format!("{}()", function.name)
    } else {
        format!("{}({})", function.name, args.join(", "))
    }
}

/// Generate request mock for integration tests
fn generate_request_mock(function: &FunctionInfo, valid: bool) -> String {
    if valid {
        format!(
            "    const {{ req, res }} = createMocks({{\n      method: 'POST',\n      url: '/{}',\n      body: {{ /* valid data */ }},\n    }});\n",
            function.name.to_lowercase()
        )
    } else {
        format!(
            "    const {{ req, res }} = createMocks({{\n      method: 'POST',\n      url: '/{}',\n      body: {{ invalid: 'data' }},\n    }});\n",
            function.name.to_lowercase()
        )
    }
}

/// Generate response assertions for integration tests
fn generate_response_assertions(success: bool) -> String {
    if success {
        "    expect(res.statusCode).toBe(200);\n    expect(res._getJSONData()).toBeDefined();\n"
    } else {
        "    expect(res.statusCode).toBeGreaterThanOrEqual(400);\n"
    }
    .to_string()
}

/// Check if function can throw errors
fn can_throw_error(function: &FunctionInfo) -> bool {
    function
        .parameters
        .iter()
        .any(|p| !p.is_optional && p.param_type.is_some())
}

/// Get framework name as string
fn framework_name(framework: TestFramework) -> &'static str {
    match framework {
        TestFramework::Vitest => "vitest",
        TestFramework::Jest => "jest",
        TestFramework::Mocha => "mocha",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Parameter, TestGeneratorOptions};
    use lumenx_core::Language;
    use lumenx_core::Visibility;

    fn create_test_function() -> FunctionInfo {
        FunctionInfo {
            name: "add".to_string(),
            file_path: PathBuf::from("/src/math.ts"),
            line: 1,
            signature: "function add(a: number, b: number): number".to_string(),
            parameters: vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: Some("number".to_string()),
                    is_optional: false,
                    default_value: None,
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: Some("number".to_string()),
                    is_optional: false,
                    default_value: None,
                },
            ],
            return_type: Some("number".to_string()),
            is_async: false,
            visibility: Visibility::Public,
            language: Language::TypeScript,
            doc_comment: Some("Adds two numbers together.".to_string()),
            suggested_tests: vec![],
        }
    }

    #[test]
    fn test_generate_unit_test() {
        let func = create_test_function();
        let options = TestGeneratorOptions::default();
        let result = generate_unit_test(&func, TestFramework::Vitest, &options);

        assert!(result.contains("describe('add'"));
        assert!(result.contains("import"));
        assert!(result.contains("vitest"));
    }

    #[test]
    fn test_generate_integration_test() {
        let func = FunctionInfo {
            name: "handleRequest".to_string(),
            file_path: PathBuf::from("/src/api.ts"),
            line: 1,
            signature: "function handleRequest(req: Request, res: Response)".to_string(),
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
            visibility: Visibility::Public,
            language: Language::TypeScript,
            doc_comment: None,
            suggested_tests: vec![],
        };

        let result = generate_integration_test(&func, TestFramework::Vitest);

        assert!(result.contains("Integration"));
        assert!(result.contains("beforeAll"));
        assert!(result.contains("afterAll"));
    }

    #[test]
    fn test_generate_function_call() {
        let func = create_test_function();
        let call = generate_function_call(&func);

        assert_eq!(call, "add(a, b)");
    }

    #[test]
    fn test_framework_name() {
        assert_eq!(framework_name(TestFramework::Vitest), "vitest");
        assert_eq!(framework_name(TestFramework::Jest), "jest");
        assert_eq!(framework_name(TestFramework::Mocha), "mocha");
    }
}

/// Generate assertions based on return type
fn generate_ts_assertions(return_type: &Option<String>) -> String {
    match return_type.as_deref() {
        Some(t) if t.contains("number") => "expect(typeof result).toBe('number');".to_string(),
        Some(t) if t.contains("string") => "expect(typeof result).toBe('string');".to_string(),
        Some(t) if t.contains("boolean") => "expect(typeof result).toBe('boolean');".to_string(),
        Some(t) if t.contains("[]") || t.contains("Array") => "expect(Array.isArray(result)).toBe(true);".to_string(),
        Some(t) if t.contains("Promise") => "expect(result).resolves.toBeDefined();".to_string(),
        _ => String::new(),
    }
}
