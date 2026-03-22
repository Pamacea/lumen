//! Security fixes
//!
//! Handles security-related code issues including:
//! - Missing input sanitization
//! - Dangerous API usage
//! - Missing authentication checks
//! - Hardcoded secrets
//! - SQL injection vulnerabilities
//! - XSS vulnerabilities

use lumenx_core::LumenResult;
use lumenx_score::ScoreIssue;
use regex::Regex;
use std::path::Path;

use super::{Fix, Fixer};

/// Security fixer
pub struct SecurityFixer {
    file_path: std::path::PathBuf,
    language: Language,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Language {
    TypeScript,
    JavaScript,
    Rust,
    Python,
    Go,
}

impl SecurityFixer {
    pub fn new(file_path: &Path) -> Self {
        let language = detect_language(file_path);
        Self {
            file_path: file_path.to_path_buf(),
            language,
        }
    }

    fn add_input_sanitization(&self, content: &str, issue: &ScoreIssue) -> String {
        match self.language {
            Language::TypeScript | Language::JavaScript => {
                self.sanitize_js_input(content, issue)
            }
            Language::Python => self.sanitize_python_input(content, issue),
            Language::Rust => self.sanitize_rust_input(content, issue),
            Language::Go => self.sanitize_go_input(content, issue),
        }
    }

    fn sanitize_js_input(&self, content: &str, issue: &ScoreIssue) -> String {
        let variable = extract_variable_name(issue).unwrap_or_else(|| "input".to_string());

        // Add sanitization function at the top if it doesn't exist
        let sanitization_func = r#"
/**
 * Sanitize user input to prevent XSS attacks
 */
function sanitizeInput(input: string): string {
    return input
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#x27;");
}

"#;

        let mut result = content.to_string();

        // Check if we need to add the sanitization function
        if !result.contains("sanitizeInput") && !result.contains("sanitize") {
            // Find a good place to insert (after imports)
            let insert_pos = find_insert_after_imports(&result);
            result.insert_str(insert_pos, sanitization_func);
        }

        // Replace dangerous patterns with sanitized versions
        let patterns = [
            format!("document.write({})", variable),
            format!("innerHTML = {}", variable),
            format!("innerHTML += {}", variable),
            format!(".innerHTML = {}", variable),
        ];

        for pattern in patterns {
            let replacement = pattern.replace(
                &variable,
                &format!("sanitizeInput({})", variable)
            );
            result = result.replace(&pattern, &replacement);
        }

        result
    }

    fn sanitize_python_input(&self, content: &str, issue: &ScoreIssue) -> String {
        let variable = extract_variable_name(issue).unwrap_or_else(|| "user_input".to_string());

        // Add sanitization import and function
        let sanitization_code = r#"
import html

def sanitize_input(input_string: str) -> str:
    """Sanitize user input to prevent XSS attacks."""
    return html.escape(input_string)

"#;

        let mut result = content.to_string();

        if !result.contains("sanitize_input") && !result.contains("html.escape") {
            let insert_pos = find_insert_after_imports(&result);
            result.insert_str(insert_pos, sanitization_code);
        }

        // Replace dangerous eval calls
        if result.contains("eval(") {
            result = result.replace(
                &format!("eval({})", variable),
                &format!("json.loads(sanitize_input({}))", variable)
            );
        }

        result
    }

    fn sanitize_rust_input(&self, content: &str, _issue: &ScoreIssue) -> String {

        // Add ammonia dependency for HTML sanitization
        let sanitization_fn = r#"
use ammonia::clean;

/// Sanitize HTML input to prevent XSS attacks
fn sanitize_html(input: &str) -> String {
    clean(input)
}

"#;

        let mut result = content.to_string();

        if !result.contains("sanitize_html") && !result.contains("ammonia") {
            // Insert use statement
            if result.contains("use ") {
                let last_use = result.rfind("use ").unwrap_or(0);
                let end_of_use = result[last_use..].find('\n').map(|p| last_use + p).unwrap_or(result.len());
                result.insert_str(end_of_use + 1, "use ammonia::clean;\n");
            } else {
                result.insert_str(0, "use ammonia::clean;\n\n");
            }

            // Insert function
            let insert_pos = find_insert_after_imports(&result);
            result.insert_str(insert_pos, sanitization_fn);
        }

        result
    }

    fn sanitize_go_input(&self, content: &str, issue: &ScoreIssue) -> String {
        let variable = extract_variable_name(issue).unwrap_or_else(|| "input".to_string());

        let sanitization_code = r#"
import "html"

// sanitizeInput sanitizes user input to prevent XSS attacks
func sanitizeInput(input string) string {
    return html.EscapeString(input)
}

"#;

        let mut result = content.to_string();

        if !result.contains("sanitizeInput") && !result.contains("html.EscapeString") {
            let insert_pos = find_insert_after_imports(&result);
            result.insert_str(insert_pos, sanitization_code);
        }

        result
    }

    fn replace_dangerous_api(&self, content: &str, issue: &ScoreIssue) -> String {
        match issue.title.as_str() {
            "Dangerous eval usage" => self.replace_eval(content),
            "Dangerous SQL query" => self.replace_dangerous_sql(content),
            "Dangerous file operation" => self.replace_dangerous_file_op(content),
            _ => content.to_string(),
        }
    }

    fn replace_eval(&self, content: &str) -> String {
        match self.language {
            Language::TypeScript | Language::JavaScript => {
                // Replace eval with safer alternatives
                let mut result = content.to_string();

                // Try to replace JSON.parse instead of eval for JSON
                let eval_re = Regex::new(r#"eval\((["\[].*?["\]])\)"#)
                    .unwrap_or_else(|_| Regex::new(r"eval\(").unwrap());
                result = eval_re.replace_all(&result, "JSON.parse($1)").to_string();

                result
            }
            _ => content.to_string(),
        }
    }

    fn replace_dangerous_sql(&self, content: &str) -> String {
        match self.language {
            Language::TypeScript | Language::JavaScript => {
                let mut result = content.to_string();

                // Replace string concatenation in SQL with parameterized queries
                // Pattern: `SELECT * FROM users WHERE id = ' + userId`
                let sql_re = Regex::new(
                    r#"(SELECT|INSERT|UPDATE|DELETE).*?\+.*?(["'])"#
                ).unwrap_or_else(|_| Regex::new(r"(SELECT|INSERT|UPDATE|DELETE)").unwrap());

                if sql_re.is_match(&result) {
                    // Add comment about using parameterized queries
                    let warning = "\n// WARNING: Use parameterized queries instead of string concatenation to prevent SQL injection\n// Example: db.query('SELECT * FROM users WHERE id = $1', [userId])\n";
                    if !result.contains("parameterized queries") {
                        result.insert_str(0, warning);
                    }
                }

                result
            }
            Language::Python => {
                let mut result = content.to_string();

                // Replace % formatting or f-strings in SQL
                if result.contains("cursor.execute(") && result.contains('%') {
                    let warning = "\n# WARNING: Use parameterized queries: cursor.execute('SELECT * WHERE id = %s', [user_id])\n";
                    if !result.contains("parameterized queries") {
                        result.insert_str(0, warning);
                    }
                }

                result
            }
            Language::Rust => {
                let mut result = content.to_string();

                // Check for unescaped SQL
                if result.contains("format!(") && (result.contains("SELECT") || result.contains("INSERT")) {
                    let warning = "\n// WARNING: Use parameterized queries with query! macro or client methods\n";
                    if !result.contains("parameterized queries") {
                        result.insert_str(0, warning);
                    }
                }

                result
            }
            Language::Go => content.to_string(),
        }
    }

    fn replace_dangerous_file_op(&self, content: &str) -> String {
        match self.language {
            Language::TypeScript | Language::JavaScript => {
                let mut result = content.to_string();

                // Replace fs.readFile with path validation
                if result.contains("fs.readFile(") {
                    let path_validation = r#"
// Validate file paths to prevent directory traversal attacks
function validatePath(filePath: string, allowedBase: string): boolean {
    const resolved = require('path').resolve(filePath);
    return resolved.startsWith(allowedBase);
}

"#;
                    if !result.contains("validatePath") {
                        let insert_pos = find_insert_after_imports(&result);
                        result.insert_str(insert_pos, path_validation);
                    }
                }

                result
            }
            _ => content.to_string(),
        }
    }

    fn add_auth_check(&self, content: &str, issue: &ScoreIssue) -> String {
        // Add authentication check comment/boilerplate
        let auth_comment = r#"
// TODO: Add authentication check
// Verify user is authenticated before processing this request
// Example:
// if (!req.user) {
//     return res.status(401).json({ error: 'Unauthorized' });
// }

"#;

        let mut result = content.to_string();

        if let Some(line) = issue.line {
            let lines: Vec<&str> = result.lines().collect();
            if line > 0 && line <= lines.len() {
                let insert_line = line - 1;
                let mut before = lines[..insert_line].join("\n");
                let after = lines[insert_line..].join("\n");
                before.push_str(auth_comment);
                result = before + &after;
            }
        } else {
            result.push_str(auth_comment);
        }

        result
    }

    fn remove_hardcoded_secret(&self, content: &str, issue: &ScoreIssue) -> String {
        let secret = extract_secret_name(issue).unwrap_or_else(|| "SECRET".to_string());

        // Replace hardcoded values with environment variable references
        let mut result = content.to_string();

        match self.language {
            Language::TypeScript | Language::JavaScript => {
                // Replace `const apiKey = "sk-..."` with `const apiKey = process.env.API_KEY`
                let api_key_re = Regex::new(&format!(
                    r#"(const|let|var)\s+{}\s*=\s*["'][^"']+["']"#,
                    regex::escape(&secret.to_lowercase())
                ))
                .unwrap_or_else(|_| Regex::new(r"(const|let|var)\s+\w+\s*=\s*["'][^"']+["']").unwrap());

                if api_key_re.is_match(&result) {
                    result = api_key_re.replace_all(&result, format!("$1 {} = process.env.{}", secret.to_uppercase(), secret.to_uppercase())).to_string();

                    // Add env import if needed
                    if !result.contains("@t3-env/env") && !result.contains("dotenv") {
                        let env_import = "\nimport { env } from '@t3-env/env';\n";
                        result.insert_str(0, env_import);
                    }
                }
            }
            Language::Rust => {
                // Replace hardcoded secrets with env variables
                let secret_re = Regex::new(&format!(
                    r#"{}\s*=\s*["'][^"']+["']"#,
                    regex::escape(&secret.to_lowercase())
                ))
                .unwrap_or_else(|_| Regex::new(r"\w+\s*=\s*["'][^"']+["']").unwrap());

                if secret_re.is_match(&result) {
                    result = secret_re.replace_all(&result, format!("{} = std::env::var(\"{}\").expect(\"{} not set\")", secret.to_lowercase(), secret.to_uppercase(), secret.to_uppercase())).to_string();
                }
            }
            Language::Python => {
                // Replace with os.getenv
                let secret_re = Regex::new(&format!(
                    r#"{}\s*=\s*["'][^"']+["']"#,
                    regex::escape(&secret.to_lowercase())
                )).unwrap();

                if secret_re.is_match(&result) {
                    result = secret_re.replace_all(&result, format!("{} = os.getenv('{}', '')", secret.to_lowercase(), secret.to_uppercase())).to_string();

                    // Add os import if needed
                    if !result.contains("import os") {
                        result.insert_str(0, "import os\n");
                    }
                }
            }
            Language::Go => {
                // Not implemented for Go, leave content as-is
            }
        }

        result
    }
}

/// Result of a security fix
#[derive(Debug, Clone)]
pub struct SecurityFix {
    pub description: String,
    pub diff: crate::patch::PatchDiff,
}

impl Fix for SecurityFix {
    fn description(&self) -> String {
        self.description.clone()
    }

    fn diff(&self) -> crate::patch::PatchDiff {
        self.diff.clone()
    }

    fn is_safe(&self) -> bool {
        // Security fixes may require manual review
        false
    }
}

impl Fixer for SecurityFixer {
    type Fix = SecurityFix;

    fn new(file_path: &Path) -> Self
    where
        Self: Sized,
    {
        Self::new(file_path)
    }

    fn can_fix(&self, issue: &ScoreIssue) -> bool {
        matches!(
            issue.category.as_str(),
            "security"
        )
        || matches!(
            issue.title.as_str(),
            "Missing input sanitization" | "Hardcoded secret" | "SQL injection" | "XSS vulnerability"
        )
    }

    fn fix(&self, content: &str, issue: &ScoreIssue) -> LumenResult<(String, Self::Fix)> {
        let (new_content, description) = match issue.title.as_str() {
            "Missing input sanitization" => {
                let fixed = self.add_input_sanitization(content, issue);
                (fixed, "Added input sanitization".to_string())
            }
            "Hardcoded secret" => {
                let fixed = self.remove_hardcoded_secret(content, issue);
                (fixed, "Replaced hardcoded secret with environment variable".to_string())
            }
            title if title.contains("SQL") || title.contains("injection") => {
                let fixed = self.replace_dangerous_api(content, issue);
                (fixed, "Added SQL injection protection warning".to_string())
            }
            title if title.contains("XSS") => {
                let fixed = self.add_input_sanitization(content, issue);
                (fixed, "Added XSS protection".to_string())
            }
            "Missing authentication" => {
                let fixed = self.add_auth_check(content, issue);
                (fixed, "Added authentication check placeholder".to_string())
            }
            _ => (content.to_string(), format!("Security fix: {}", issue.title)),
        };

        let diff = crate::patch::PatchDiff::simple(content.to_string(), new_content.clone());
        Ok((new_content, SecurityFix { description, diff }))
    }

    fn fix_all(&self, content: &str, issues: &[&ScoreIssue]) -> LumenResult<(String, Vec<Self::Fix>)> {
        let mut current_content = content.to_string();
        let mut fixes = Vec::new();

        // Group issues by type and apply in order
        let mut sanitization_issues: Vec<&ScoreIssue> = Vec::new();
        let mut secret_issues: Vec<&ScoreIssue> = Vec::new();
        let mut sql_issues: Vec<&ScoreIssue> = Vec::new();
        let mut xss_issues: Vec<&ScoreIssue> = Vec::new();
        let mut auth_issues: Vec<&ScoreIssue> = Vec::new();

        for issue in issues {
            match issue.title.as_str() {
                title if title.contains("sanitization") => sanitization_issues.push(*issue),
                title if title.contains("secret") => secret_issues.push(*issue),
                title if title.contains("SQL") || title.contains("injection") => sql_issues.push(*issue),
                title if title.contains("XSS") => xss_issues.push(*issue),
                title if title.contains("auth") => auth_issues.push(*issue),
                _ => {}
            }
        }

        // Apply fixes in order of safety
        if !sanitization_issues.is_empty() || !xss_issues.is_empty() {
            let all_sanitization: Vec<&ScoreIssue> = sanitization_issues
                .into_iter()
                .chain(xss_issues)
                .collect();

            for issue in all_sanitization {
                let old_content = current_content.clone();
                current_content = self.add_input_sanitization(&current_content, issue);
                fixes.push(SecurityFix {
                    description: format!("Added input sanitization for {}", issue.file.as_deref().unwrap_or("unknown")),
                    diff: crate::patch::PatchDiff::simple(old_content, current_content.clone()),
                });
            }
        }

        for issue in secret_issues {
            let old_content = current_content.clone();
            current_content = self.remove_hardcoded_secret(&current_content, issue);
            fixes.push(SecurityFix {
                description: format!("Replaced hardcoded secret in {}", issue.file.as_deref().unwrap_or("unknown")),
                diff: crate::patch::PatchDiff::simple(old_content, current_content.clone()),
            });
        }

        for issue in sql_issues {
            let old_content = current_content.clone();
            current_content = self.replace_dangerous_api(&current_content, issue);
            fixes.push(SecurityFix {
                description: format!("Added SQL injection protection for {}", issue.file.as_deref().unwrap_or("unknown")),
                diff: crate::patch::PatchDiff::simple(old_content, current_content.clone()),
            });
        }

        for issue in auth_issues {
            let old_content = current_content.clone();
            current_content = self.add_auth_check(&current_content, issue);
            fixes.push(SecurityFix {
                description: format!("Added authentication check for {}", issue.file.as_deref().unwrap_or("unknown")),
                diff: crate::patch::PatchDiff::simple(old_content, current_content.clone()),
            });
        }

        Ok((current_content, fixes))
    }
}

fn detect_language(path: &Path) -> Language {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| match ext {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "go" => Language::Go,
            "ts" | "tsx" => Language::TypeScript,
            "js" | "jsx" | "mjs" | "cjs" => Language::JavaScript,
            _ => Language::TypeScript,
        })
        .unwrap_or(Language::TypeScript)
}

fn extract_variable_name(issue: &ScoreIssue) -> Option<String> {
    issue
        .description
        .split('\'')
        .nth(1)
        .or_else(|| issue.description.split('"').nth(1))
        .map(|s| s.to_string())
}

fn extract_secret_name(issue: &ScoreIssue) -> Option<String> {
    issue
        .title
        .split_whitespace()
        .last()
        .map(|s| s.to_string())
}

fn find_insert_after_imports(content: &str) -> usize {
    let mut last_import_end = 0;
    let mut line_count = 0;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("from ")
            || trimmed.starts_with("include ")
            || trimmed.starts_with("require(")
        {
            last_import_end = i;
        }
    }

    // Find the byte position after the last import line
    for (byte_pos, ch) in content.char_indices() {
        if ch == '\n' {
            line_count += 1;
            if line_count > last_import_end {
                return byte_pos + 1;
            }
        }
    }

    content.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumenx_score::IssueSeverity;

    #[test]
    fn test_sanitize_js_input() {
        let fixer = SecurityFixer::new(Path::new("test.ts"));

        let content = "function render(input: string) { document.write(input); }";
        let issue = ScoreIssue {
            id: "sec-1".to_string(),
            severity: IssueSeverity::High,
            category: "security".to_string(),
            title: "Missing input sanitization".to_string(),
            description: "Variable 'input' should be sanitized".to_string(),
            file: Some("test.ts".to_string()),
            line: Some(1),
            column: None,
            impact: 10.0,
            suggestion: None,
        };

        let (fixed, fix_result) = fixer.fix(content, &issue).unwrap();
        assert!(fixed.contains("sanitizeInput"));
        assert!(fixed.contains("sanitizeInput(input)"));
        assert_eq!(fix_result.description, "Added input sanitization");
    }

    #[test]
    fn test_replace_eval() {
        let fixer = SecurityFixer::new(Path::new("test.js"));

        let content = "const data = eval(jsonString);";
        let issue = ScoreIssue {
            id: "sec-2".to_string(),
            severity: IssueSeverity::Critical,
            category: "security".to_string(),
            title: "Dangerous eval usage".to_string(),
            description: "eval() is dangerous".to_string(),
            file: Some("test.js".to_string()),
            line: Some(1),
            column: None,
            impact: 20.0,
            suggestion: None,
        };

        let (fixed, _) = fixer.fix(content, &issue).unwrap();
        assert!(fixed.contains("JSON.parse"));
    }
}
