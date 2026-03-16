//! Go test templates

use crate::{FunctionInfo, Parameter, TestGeneratorOptions};

/// Helper trait for string case conversion
trait StringCase {
    fn to_snake_case(&self) -> String;
}

impl StringCase for str {
    fn to_snake_case(&self) -> String {
        self.chars()
            .enumerate()
            .map(|(i, c)| {
                if c.is_uppercase() {
                    if i > 0 {
                        format!("_{}", c.to_lowercase().collect::<String>())
                    } else {
                        c.to_lowercase().collect()
                    }
                } else {
                    c.to_string()
                }
            })
            .collect()
    }
}

/// Generate unit test for a Go function
pub fn generate_unit_test(function: &FunctionInfo, _options: &TestGeneratorOptions) -> String {
    let test_name = format!("Test{}", to_pascal_case(&function.name));

    format!(
        r#"package {package_name}

import (
    "testing"
)

// {test_name} tests the {function_name} function
func {test_name}(t *testing.T) {{
    // Arrange
    {setup}

    // Act
    {call}

    // Assert
    {assertions}
}}
{additional_tests}
"#,
        package_name = function
            .file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("main"),
        test_name = test_name,
        function_name = function.name,
        setup = generate_test_setup(function),
        call = generate_function_call(function),
        assertions = generate_assertions(function),
        additional_tests = generate_additional_tests(function)
    )
}

/// Generate integration test for a Go endpoint handler
pub fn generate_integration_test(function: &FunctionInfo) -> String {
    format!(
        r#"package {package_name}_test

import (
    "bytes"
    "encoding/json"
    "net/http"
    "net/http/httptest"
    "testing"

    "github.com/stretchr/testify/assert"
    "github.com/stretchr/testify/require"
)

// Test{CamelCase}Handler tests the {function_name} HTTP handler
func Test{CamelCase}Handler(t *testing.T) {{
    // TODO: Implement integration test
    // Example with httptest:
    //
    // req := httptest.NewRequest("GET", "/{endpoint}", nil)
    // w := httptest.NewRecorder()
    //
    // {function_name}Handler(w, req)
    //
    // assert.Equal(t, http.StatusOK, w.Code)
    // assert.Contains(t, w.Body.String(), "expected")
}}
"#,
        package_name = "main",  // TODO: Extract from function
        CamelCase = to_pascal_case(&function.name),
        function_name = function.name,
        endpoint = function.name.to_snake_case()
    )
}

/// Generate E2E test for a Go application
pub fn generate_e2e_test(function: &FunctionInfo) -> String {
    format!(
        r#"package {package_name}_test

import (
    "context"
    "testing"
    "time"

    "github.com/testcontainers/testcontainers-go/modules/postgres"
)

// Test{CamelCase}E2E tests the complete flow
func Test{CamelCase}E2E(t *testing.T) {{
    // TODO: Implement end-to-end test
    // Example with testcontainers:
    //
    // ctx := context.Background()
    //
    // postgresContainer, err := postgres.RunContainer(ctx, "")
    // require.NoError(t, err)
    // defer postgresContainer.Terminate(ctx)
    //
    // connectionString, err := postgresContainer.ConnectionString(ctx)
    // require.NoError(t, err)
    //
    // // Run your application with test database
    // // Make HTTP requests and verify behavior
}}
"#,
        package_name = "main",
        CamelCase = to_pascal_case(&function.name)
    )
}

/// Generate test for a single function
pub fn generate_function_test(function: &FunctionInfo) -> String {
    generate_unit_test(function, &TestGeneratorOptions::default())
}

// Helper functions

fn generate_test_setup(function: &FunctionInfo) -> String {
    let mut setup_lines = Vec::new();

    for param in &function.parameters {
        let mock_value = generate_mock_value(param);
        setup_lines.push(format!("{} := {}", param.name, mock_value));
    }

    if setup_lines.is_empty() {
        "// Arrange: Set up test data".to_string()
    } else {
        format!("// Arrange\n{}", setup_lines.join("\n    "))
    }
}

fn generate_function_call(function: &FunctionInfo) -> String {
    let params: Vec<String> = function.parameters.iter().map(|p| p.name.clone()).collect();

    if function.is_async {
        // For async Go functions, we'd need to use goroutines or context
        format!("// Act (async function)\n    // TODO: Handle async call to {}({})", function.name, params.join(", "))
    } else {
        format!("// Act\n    result := {}({})", function.name, params.join(", "))
    }
}

fn generate_assertions(function: &FunctionInfo) -> String {
    match function.return_type.as_deref() {
        Some(t) if t.contains("error") && t.contains("bool") => {
            "assert.NoError(t, result)\n    assert.True(t, result)".to_string()
        }
        Some(t) if t.contains("error") => {
            "assert.NoError(t, result)".to_string()
        }
        Some(t) if t.contains("bool") => {
            "assert.True(t, result)".to_string()
        }
        Some(t) if t.contains("string") => {
            "assert.NotEmpty(t, result)".to_string()
        }
        Some(t) if t.contains("[]") || t.contains("[]") || t.contains("Slice") => {
            "assert.NotEmpty(t, result)".to_string()
        }
        Some(t) if t.contains("int") || t.contains("float") => {
            "assert.Greater(t, result, 0)".to_string()
        }
        _ => "// Assert: Add proper assertion\n    assert.NotNil(t, result)".to_string(),
    }
}

fn generate_additional_tests(function: &FunctionInfo) -> String {
    let mut tests = Vec::new();

    tests.push(generate_table_driven_test(function));

    tests.join("\n\n")
}

fn generate_table_driven_test(function: &FunctionInfo) -> String {
    format!(
        r#"

// Test{CamelCase}TableDriven tests {function_name} with various inputs
func Test{CamelCase}TableDriven(t *testing.T) {{
    tests := []struct {{
        name    string
        setup   func()
        want    {return_type}
        wantErr bool
    }}{{
        {{
            name: "valid input",
            setup: func() {{ {setup} }},
            want: {default_return},
            wantErr: false,
        }},
        {{
            name: "invalid input",
            setup: func() {{ {error_setup} }},
            want: {default_return},
            wantErr: true,
        }},
    }}

    for _, tt := range tests {{
        t.Run(tt.name, func(t *testing.T) {{
            tt.setup()

            got, err := {function_call}
            if (err != nil) != tt.wantErr {{
                t.Errorf("expected error = %v, got %v", tt.wantErr, err)
                return
            }}
            {comparison}
        }})
    }}
}}
"#,
        CamelCase = to_pascal_case(&function.name),
        function_name = function.name,
        return_type = function.return_type.as_deref().unwrap_or("interface{}"),
        default_return = default_return_value(function),
        setup = generate_test_setup(function),
        error_setup = generate_error_setup(function),
        function_call = generate_go_call_with_return(function),
        comparison = generate_table_comparison(function)
    )
}

fn generate_go_call_with_return(function: &FunctionInfo) -> String {
    let params: Vec<String> = function.parameters.iter().map(|p| p.name.clone()).collect();

    if function.return_type.as_ref().map_or(false, |t| t.contains("error")) {
        format!("{}({})", function.name, params.join(", "))
    } else {
        format!("{}({})", function.name, params.join(", "))
    }
}

fn generate_table_comparison(function: &FunctionInfo) -> String {
    if function.return_type.as_ref().map_or(false, |t| !t.contains("error")) {
        "if got != tt.want {\n                t.Errorf(\"got = %v, want %v\", got, tt.want)\n            }".to_string()
    } else {
        "// Verify result matches expectation".to_string()
    }
}

fn default_return_value(function: &FunctionInfo) -> String {
    match function.return_type.as_deref() {
        Some(t) if t.contains("string") => r#""#.to_string(),
        Some(t) if t.contains("int") => "0".to_string(),
        Some(t) if t.contains("float") => "0.0".to_string(),
        Some(t) if t.contains("bool") => "true".to_string(),
        Some(t) if t.contains("[]") => "nil".to_string(),
        Some(t) if t.contains("error") => "nil".to_string(),
        _ => "nil".to_string(),
    }
}

fn generate_error_setup(function: &FunctionInfo) -> String {
    if let Some(first_param) = function.parameters.first() {
        let invalid_value = generate_invalid_value(first_param);
        let mut setup = Vec::new();

        for param in &function.parameters {
            if param.name == first_param.name {
                setup.push(format!("{} := {}", param.name, invalid_value));
            } else {
                setup.push(format!("{} := {}", param.name, generate_mock_value(param)));
            }
        }

        setup.join("\n            ")
    } else {
        "// Error setup".to_string()
    }
}

fn generate_mock_value(param: &Parameter) -> String {
    match param.param_type.as_deref() {
        Some(t) if t.contains("string") => {
            if param.is_optional {
                "".to_string()
            } else {
                r#""test""#.to_string()
            }
        }
        Some(t) if t.contains("int") => {
            if param.is_optional {
                "0".to_string()
            } else {
                "42".to_string()
            }
        }
        Some(t) if t.contains("float64") || t.contains("float32") => {
            if param.is_optional {
                "0.0".to_string()
            } else {
                "3.14".to_string()
            }
        }
        Some(t) if t.contains("bool") => {
            if param.is_optional {
                "false".to_string()
            } else {
                "true".to_string()
            }
        }
        Some(t) if t.contains("[]") || t.contains("[]") || t.contains("Slice") => {
            "nil".to_string()
        }
        Some(t) if t.contains("map") => {
            "nil".to_string()
        }
        _ => {
            if param.is_optional {
                "nil".to_string()
            } else {
                "// TODO: Provide mock value".to_string()
            }
        }
    }
}

fn generate_invalid_value(param: &Parameter) -> String {
    match param.param_type.as_deref() {
        Some(t) if t.contains("string") => r#""""#.to_string(),
        Some(t) if t.contains("int") => "-1".to_string(),
        Some(t) if t.contains("float") => "-1.0".to_string(),
        Some(t) if t.contains("bool") => "false".to_string(),
        _ => "nil".to_string(),
    }
}

fn to_pascal_case(s: &str) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if c == '_' {
                String::new()
            } else if i == 0 || (s.chars().nth(i - 1) == Some('_')) {
                c.to_uppercase().collect::<String>()
            } else {
                c.to_string()
            }
        })
        .collect()
}
