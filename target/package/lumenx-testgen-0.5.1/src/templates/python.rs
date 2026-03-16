//! Python test templates

use crate::{FunctionInfo, Parameter, TestGeneratorOptions, TestCase, TestCaseType};
use lumenx_core::Language;

// Constants for Python comments to avoid parsing issues
const PY_COMMENT: &str = "\x23";
const PY_COMMENT_SPACE: &str = "\x23 ";

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

/// Generate unit test for a Python function
pub fn generate_unit_test(function: &FunctionInfo, _options: &TestGeneratorOptions) -> String {
    let _test_name = format!("test_{}", function.name.to_snake_case());

    format!(
        r#"""
import pytest
from {module} import {function_name}


class Test{CamelCase}:
    """Test suite for {function_name}()"""

    def test_{name_snake}_happy_path(self):
        """Test {function_name} with valid inputs"""
        # Arrange
        {setup}

        # Act
        {call}

        # Assert
        {assertions}
{additional_tests}
"""#,
        module = function
            .file_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("module"),
        function_name = function.name,
        CamelCase = to_pascal_case(&function.name),
        name_snake = function.name.to_snake_case(),
        setup = generate_test_setup(function),
        call = generate_function_call(function),
        assertions = generate_assertions(function),
        additional_tests = generate_additional_tests(function)
    )
}

/// Generate integration test for a Python endpoint handler
pub fn generate_integration_test(function: &FunctionInfo) -> String {
    format!(
        r#"""
import pytest
from fastapi.testclient import TestClient
from {module} import app


class Test{CamelCase}Integration:
    """Integration tests for {function_name}"""

    @pytest.fixture
    def client(self):
        """Create test client"""
        return TestClient(app)

    def test_{name_snake}_endpoint(self, client):
        """Test {function_name} endpoint"""
        # TODO: Implement integration test
        # Example:
        # response = client.get("/{endpoint}")
        # assert response.status_code == 200
        # data = response.json()
        # assert "expected_key" in data
"""#,
        module = "app",  // TODO: Extract from function
        CamelCase = to_pascal_case(&function.name),
        name_snake = function.name.to_snake_case(),
        function_name = function.name,
        endpoint = function.name.to_snake_case()
    )
}

/// Generate E2E test for a Python application
pub fn generate_e2e_test(function: &FunctionInfo) -> String {
    format!(
        r#"""
import pytest
from playwright.sync_api import Page, expect


class Test{CamelCase}E2E:
    """End-to-end tests for {function_name}"""

    def test_{name_snake}_user_flow(self, page: Page):
        """Test complete user flow"""
        # TODO: Implement E2E test
        # Example with Playwright:
        # page.goto("https://localhost:8000")
        # page.click("text=Login")
        # page.fill("input[name='email']", "test@example.com")
        # page.fill("input[name='password']", "password")
        # page.click("button[type='submit']")
        # expect(page).to_have_url("https://localhost:8000/dashboard")
"""#,
        CamelCase = to_pascal_case(&function.name),
        name_snake = function.name.to_snake_case(),
        function_name = function.name
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
        setup_lines.push(format!("{} = {}", param.name, mock_value));
    }

    let nl = "\n";
    if setup_lines.is_empty() {
        format!("{} Arrange", PY_COMMENT)
    } else {
        format!("{} Arrange{}{}", PY_COMMENT, nl, setup_lines.join(&format!("{}        ", nl)))
    }
}

fn generate_function_call(function: &FunctionInfo) -> String {
    let params: Vec<String> = function.parameters.iter().map(|p| p.name.clone()).collect();
    let call = if function.is_async {
        format!("await {}({})", function.name, params.join(", "))
    } else {
        format!("{}({})", function.name, params.join(", "))
    };

    let result_binding = if function.return_type.as_ref().map_or(false, |t| !t.is_empty()) {
        "result = "
    } else {
        ""
    };

    let nl = "\n";
    format!("{} Act{}        {}{}", PY_COMMENT, nl, result_binding, call)
}

fn generate_assertions(function: &FunctionInfo) -> String {
    let nl = "\n";
    match function.return_type.as_deref() {
        Some(t) if t.contains("None") => {
            format!("assert result is not {}", "None")
        }
        Some(t) if t.contains("bool") => {
            format!("assert result is {}", "True")
        }
        Some(t) if t.contains("List") || t.contains("list") || t.contains("[]") => {
            String::from("assert len(result) > 0")
        }
        Some(t) if t.contains("Dict") || t.contains("dict") || t.contains("{") => {
            String::from("assert isinstance(result, dict)")
        }
        Some(t) if t.contains("str") => {
            format!("assert isinstance(result, str){}        assert len(result) > 0", nl)
        }
        Some(t) if t.contains("int") || t.contains("float") => {
            String::from("assert isinstance(result, (int, float))")
        }
        _ => format!("{} Assert{}        assert True  {} TODO: Add proper assertion",
                     PY_COMMENT, nl, PY_COMMENT_SPACE),
    }
}

fn generate_additional_tests(function: &FunctionInfo) -> String {
    let mut tests = Vec::new();

    // Happy path
    tests.push(generate_happy_path_test(function));

    // Error case
    tests.push(generate_error_test(function));

    tests.join(r#"
"#)
}

fn generate_happy_path_test(function: &FunctionInfo) -> String {
    format!(
        r#"

    def test_{name_snake}_with_valid_inputs(self):
        """Test {function_name} with typical valid inputs"""
        # Arrange
        {setup}

        # Act
        {call}

        # Assert
        {assertions}
"#,
        name_snake = function.name.to_snake_case(),
        function_name = function.name,
        setup = generate_test_setup(function),
        call = generate_function_call(function),
        assertions = generate_assertions(function)
    )
}

fn generate_error_test(function: &FunctionInfo) -> String {
    format!(
        r#"

    def test_{name_snake}_with_invalid_inputs(self):
        """Test {function_name} with invalid inputs"""
        # Arrange
        with pytest.raises(ValueError) as exc_info:
            # Act
            {error_call}

        # Assert
        assert "expected error" in str(exc_info.value)
"#,
        name_snake = function.name.to_snake_case(),
        function_name = function.name,
        error_call = generate_error_call(function)
    )
}

fn generate_error_call(function: &FunctionInfo) -> String {
    if let Some(first_param) = function.parameters.first() {
        let invalid_value = generate_invalid_value(first_param);
        let params: Vec<String> = function
            .parameters
            .iter()
            .map(|p| {
                if p.name == first_param.name {
                    invalid_value.clone()
                } else {
                    generate_mock_value(p)
                }
            })
            .collect();

        if function.is_async {
            format!("await {}({})", function.name, params.join(", "))
        } else {
            format!("{}({})", function.name, params.join(", "))
        }
    } else {
        format!("{}()", function.name)
    }
}

fn generate_mock_value(param: &Parameter) -> String {
    match param.param_type.as_deref() {
        Some(t) if t.contains("str") => {
            if param.is_optional {
                "None".to_string()
            } else {
                r#""test_value""#.to_string()
            }
        }
        Some(t) if t.contains("int") => {
            if param.is_optional {
                "None".to_string()
            } else {
                "42".to_string()
            }
        }
        Some(t) if t.contains("float") => {
            if param.is_optional {
                "None".to_string()
            } else {
                "3.14".to_string()
            }
        }
        Some(t) if t.contains("bool") => {
            if param.is_optional {
                "None".to_string()
            } else {
                "True".to_string()
            }
        }
        Some(t) if t.contains("List") => {
            if param.is_optional {
                "None".to_string()
            } else {
                "[]".to_string()
            }
        }
        Some(t) if t.contains("Dict") => {
            if param.is_optional {
                "None".to_string()
            } else {
                "{}".to_string()
            }
        }
        Some(t) if t.contains("Optional") => {
            "None".to_string()
        }
        _ => {
            if param.is_optional {
                "None".to_string()
            } else {
                format!("{} TODO: Provide mock value", PY_COMMENT_SPACE)
            }
        }
    }
}

fn generate_invalid_value(param: &Parameter) -> String {
    match param.param_type.as_deref() {
        Some(t) if t.contains("int") => "-1".to_string(),
        Some(t) if t.contains("float") => "-1.0".to_string(),
        Some(t) if t.contains("str") => r##""""##.to_string(),
        Some(t) if t.contains("List") => r##""test""##.to_string(),  // Wrong type
        Some(t) if t.contains("Dict") => r##""test""##.to_string(),  // Wrong type
        _ => "None".to_string(),
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
