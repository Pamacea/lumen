//! Code transformers for common refactoring patterns
//!
//! Provides reusable transformers for various code modifications.

use lumenx_core::LumenResult;
use regex::Regex;

/// A code transformer that can modify source code
pub trait Transformer: Send + Sync {
    /// Transform the given content
    fn transform(&self, content: &str) -> LumenResult<String>;

    /// Check if this transformer can apply to the content
    fn can_apply(&self, content: &str) -> bool;

    /// Get a description of what this transformer does
    fn description(&self) -> &str {
        "Code transformation"
    }
}

/// Remove unused imports
#[derive(Debug, Clone)]
pub struct RemoveUnusedImports {
    /// Imports to remove
    pub imports: Vec<String>,
    /// Language of the file
    pub language: Language,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    TypeScript,
    JavaScript,
    Rust,
    Python,
    Go,
}

impl RemoveUnusedImports {
    /// Create a new import remover
    pub fn new(imports: Vec<String>, language: Language) -> Self {
        Self { imports, language }
    }

    /// Detect language from file extension
    pub fn from_file_path(imports: Vec<String>, path: &str) -> Self {
        let language = match std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
        {
            Some("rs") => Language::Rust,
            Some("py") => Language::Python,
            Some("go") => Language::Go,
            Some("ts" | "tsx") => Language::TypeScript,
            Some("js" | "jsx") => Language::JavaScript,
            _ => Language::TypeScript,
        };
        Self { imports, language }
    }
}

impl Transformer for RemoveUnusedImports {
    fn transform(&self, content: &str) -> LumenResult<String> {
        let mut result = content.to_string();

        for import in &self.imports {
            result = match self.language {
                Language::Rust => {
                    // Remove Rust use statements
                    result
                        .replace(&format!("use {};", import), "")
                        .replace(&format!("use {}::*;", import), "")
                        .replace(&format!("use {} as ", import), "")
                }
                Language::TypeScript | Language::JavaScript => {
                    // Remove TypeScript/JavaScript imports
                    result
                        .replace(&format!("import {} from", import), "")
                        .replace(&format!("import {{ {} }}", import), "")
                        .replace(&format!("import {{ {} as ", import), "")
                        .replace(&format!("import * as {} from", import), "")
                }
                Language::Python => {
                    // Remove Python imports
                    result
                        .replace(&format!("import {}", import), "")
                        .replace(&format!("from {} import", import), "")
                }
                Language::Go => {
                    // Skip Go imports (handled differently)
                    result
                }
            };

            // Clean up extra blank lines
            result = clean_blank_lines(&result);
        }

        Ok(result)
    }

    fn can_apply(&self, content: &str) -> bool {
        self.imports.iter().any(|import| {
            match self.language {
                Language::Rust => {
                    content.contains(&format!("use {};", import))
                        || content.contains(&format!("use {}::*;", import))
                }
                Language::TypeScript | Language::JavaScript => {
                    content.contains(&format!("import {} from", import))
                        || content.contains(&format!("import {{ {} }}", import))
                }
                Language::Python => {
                    content.contains(&format!("import {}", import))
                        || content.contains(&format!("from {} import", import))
                }
                Language::Go => false, // Go imports are in a block
            }
        })
    }

    fn description(&self) -> &str {
        "Remove unused imports"
    }
}

/// Sort imports alphabetically
#[derive(Debug, Clone)]
pub struct SortImports {
    /// Language of the file
    pub language: Language,
}

impl SortImports {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    pub fn from_file_path(path: &str) -> Self {
        let language = match std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
        {
            Some("rs") => Language::Rust,
            Some("py") => Language::Python,
            Some("go") => Language::Go,
            Some("ts" | "tsx") => Language::TypeScript,
            Some("js" | "jsx") => Language::JavaScript,
            _ => Language::TypeScript,
        };
        Self { language }
    }
}

impl Transformer for SortImports {
    fn transform(&self, content: &str) -> LumenResult<String> {
        match self.language {
            Language::Rust => self.sort_rust_imports(content),
            Language::TypeScript | Language::JavaScript => self.sort_ts_imports(content),
            Language::Python => self.sort_python_imports(content),
            Language::Go => self.sort_go_imports(content),
        }
    }

    fn can_apply(&self, content: &str) -> bool {
        content.contains("import ") || content.contains("use ") || content.contains("from ")
    }

    fn description(&self) -> &str {
        "Sort imports alphabetically"
    }
}

impl SortImports {
    fn sort_rust_imports(&self, content: &str) -> LumenResult<String> {
        let mut result = String::new();
        let mut import_block = Vec::new();
        let mut in_import_block = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("use ") {
                in_import_block = true;
                import_block.push(line.to_string());
            } else if in_import_block && (trimmed.is_empty() || trimmed == "}") {
                // End of import block
                import_block.sort();
                import_block.dedup();
                result.push_str(&import_block.join("\n"));
                result.push('\n');
                result.push_str(line);
                result.push('\n');
                import_block.clear();
                in_import_block = false;
            } else if in_import_block {
                import_block.push(line.to_string());
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        // Handle any remaining imports
        if !import_block.is_empty() {
            import_block.sort();
            import_block.dedup();
            result.push_str(&import_block.join("\n"));
            result.push('\n');
        }

        Ok(result)
    }

    fn sort_ts_imports(&self, content: &str) -> LumenResult<String> {
        let mut imports = Vec::new();
        let mut other_lines = Vec::new();
        let mut lines: Vec<&str> = content.lines().collect();

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("import ") {
                // Handle multi-line imports
                let mut import_line = lines[i].to_string();
                i += 1;
                while i < lines.len() && !lines[i].trim().ends_with(';') {
                    import_line.push('\n');
                    import_line.push_str(lines[i]);
                    i += 1;
                }
                if i < lines.len() {
                    import_line.push('\n');
                    import_line.push_str(lines[i]);
                }
                imports.push(import_line);
            } else {
                other_lines.push(lines[i].to_string());
            }
            i += 1;
        }

        imports.sort();
        imports.dedup();

        let mut result = String::new();
        for import in imports {
            result.push_str(&import);
            result.push('\n');
        }
        result.push('\n');
        for line in other_lines {
            result.push_str(&line);
            result.push('\n');
        }

        Ok(result)
    }

    fn sort_python_imports(&self, content: &str) -> LumenResult<String> {
        let mut std_imports = Vec::new();
        let mut local_imports = Vec::new();
        let mut other_lines = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                // Distinguish std lib from local imports
                if trimmed.contains("from .") || trimmed.contains("from ..") {
                    local_imports.push(line.to_string());
                } else {
                    std_imports.push(line.to_string());
                }
            } else {
                other_lines.push(line.to_string());
            }
        }

        std_imports.sort();
        local_imports.sort();

        let mut result = String::new();
        for imp in &std_imports {
            result.push_str(imp);
            result.push('\n');
        }
        if !std_imports.is_empty() && !local_imports.is_empty() {
            result.push('\n');
        }
        for imp in &local_imports {
            result.push_str(imp);
            result.push('\n');
        }
        result.push('\n');
        for line in &other_lines {
            result.push_str(line);
            result.push('\n');
        }

        Ok(result)
    }

    fn sort_go_imports(&self, content: &str) -> LumenResult<String> {
        // Go imports are usually in a single block
        let import_re = Regex::new(r"(?ms)import \((.*?)\)")?;
        let result = import_re.replace_all(content, |caps: &regex::Captures| {
            if let Some(block) = caps.get(1) {
                let imports: Vec<&str> = block.as_str()
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty() && !l.starts_with('"'))
                    .collect();
                let mut sorted: Vec<&str> = imports;
                sorted.sort();
                sorted.dedup();
                let mut result = "import (\n".to_string();
                for imp in sorted {
                    result.push_str("\t");
                    result.push_str(imp);
                    result.push('\n');
                }
                result.push(')');
                result
            } else {
                caps.get(0).unwrap().as_str().to_string()
            }
        });

        Ok(result.to_string())
    }
}

/// Extract magic numbers to named constants
#[derive(Debug, Clone)]
pub struct ExtractConstants {
    /// Magic numbers to extract with their suggested names
    pub constants: Vec<(String, String)>, // (value, name)
}

impl ExtractConstants {
    pub fn new(constants: Vec<(String, String)>) -> Self {
        Self { constants }
    }
}

impl Transformer for ExtractConstants {
    fn transform(&self, content: &str) -> LumenResult<String> {
        let mut result = content.to_string();

        // Find where to insert constants (after imports/uses)
        let insert_pos = find_insert_position(&result);

        // Insert constant definitions
        let mut constants_block = String::new();
        for (value, name) in &self.constants {
            // Replace magic numbers in code
            result = result.replace(value, name);

            // Add constant definition
            constants_block.push_str(&format!("const {}: {} = {};\n", name, infer_type(value), value));
        }

        if !constants_block.is_empty() {
            constants_block.push('\n');
            result.insert_str(insert_pos, &constants_block);
        }

        Ok(result)
    }

    fn can_apply(&self, content: &str) -> bool {
        self.constants.iter().any(|(value, _)| content.contains(value))
    }

    fn description(&self) -> &str {
        "Extract magic numbers to named constants"
    }
}

/// Add type annotations to untyped function parameters
#[derive(Debug, Clone)]
pub struct AddTypeAnnotations {
    /// Functions to add types to
    pub functions: Vec<(String, Vec<String>)>, // (function_name, [param_types])
}

impl AddTypeAnnotations {
    pub fn new(functions: Vec<(String, Vec<String>)>) -> Self {
        Self { functions }
    }
}

impl Transformer for AddTypeAnnotations {
    fn transform(&self, content: &str) -> LumenResult<String> {
        let mut result = content.to_string();

        for (func_name, param_types) in &self.functions {
            // Find function definition and add types
            let func_re = Regex::new(&format!(
                r"(function|const)\s+{}\s*\(([^)]*)\)",
                regex::escape(func_name)
            ))?;

            result = func_re.replace_all(&result, |caps: &regex::Captures| {
                let prefix = caps.get(1).unwrap().as_str();
                let params = caps.get(2).unwrap().as_str();

                // Parse parameters and add types
                let params: Vec<&str> = params.split(',').map(|p| p.trim()).collect();
                let typed_params: Vec<String> = params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        if p.contains(':') {
                            p.to_string()
                        } else if let Some(typ) = param_types.get(i) {
                            format!("{}: {}", p, typ)
                        } else {
                            format!("{}: any", p)
                        }
                    })
                    .collect();

                format!("{} {} ({})", prefix, func_name, typed_params.join(", "))
            }).to_string();
        }

        Ok(result)
    }

    fn can_apply(&self, content: &str) -> bool {
        self.functions.iter().any(|(name, _)| content.contains(name))
    }

    fn description(&self) -> &str {
        "Add TypeScript type annotations"
    }
}

/// Helper function to remove extra blank lines
fn clean_blank_lines(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut blank_count = 0;

    for line in &lines {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                result.push(*line);
            }
        } else {
            blank_count = 0;
            result.push(*line);
        }
    }

    result.join("\n")
}

/// Find a good position to insert code (after imports)
fn find_insert_position(content: &str) -> usize {
    let mut pos = 0;
    let mut last_import_end = 0;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("from ")
            || trimmed.starts_with("include ")
        {
            last_import_end = i;
        }
    }

    // Find the byte position after the last import line
    let mut line_count = 0;
    for (byte_pos, ch) in content.char_indices() {
        if ch == '\n' {
            line_count += 1;
            if line_count > last_import_end {
                pos = byte_pos + 1;
                break;
            }
        }
    }

    pos
}

/// Infer type from a value string
fn infer_type(value: &str) -> &'static str {
    if value.contains('.') || value.parse::<f64>().is_ok() {
        "number"
    } else if value == "true" || value == "false" {
        "boolean"
    } else if value.starts_with('"') || value.starts_with('\'') {
        "string"
    } else if value.starts_with('[') {
        "Array"
    } else if value.starts_with('{') {
        "Record"
    } else {
        "any"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_unused_imports_rust() {
        let transformer = RemoveUnusedImports::new(
            vec!["std::collections::HashMap".to_string()],
            Language::Rust,
        );

        let input = "use std::collections::HashMap;\nfn main() {}";
        let output = transformer.transform(input).unwrap();

        assert!(!output.contains("use std::collections::HashMap"));
    }

    #[test]
    fn test_remove_unused_imports_typescript() {
        let transformer = RemoveUnusedImports::new(
            vec!["React".to_string()],
            Language::TypeScript,
        );

        let input = "import React from 'react';\nexport const foo = 1;";
        let output = transformer.transform(input).unwrap();

        assert!(!output.contains("import React"));
    }

    #[test]
    fn test_sort_rust_imports() {
        let transformer = SortImports::new(Language::Rust);

        let input = "use std::b;\nuse std::a;\n\nfn main() {}";
        let output = transformer.transform(input).unwrap();

        let lines: Vec<&str> = output.lines().collect();
        assert!(lines[0].contains("std::a"));
        assert!(lines[1].contains("std::b"));
    }

    #[test]
    fn test_extract_constants() {
        let transformer = ExtractConstants::new(vec![
            ("100".to_string(), "MAX_ITEMS".to_string()),
            ("3.14".to_string(), "PI".to_string()),
        ]);

        let input = "function area(r) { return 3.14 * r * r; }\nif (items.length < 100) {}";
        let output = transformer.transform(input).unwrap();

        assert!(output.contains("const MAX_ITEMS: number = 100"));
        assert!(output.contains("const PI: number = 3.14"));
        assert!(output.contains("PI * r * r"));
        assert!(output.contains("< MAX_ITEMS"));
    }

    #[test]
    fn test_add_type_annotations() {
        let transformer = AddTypeAnnotations::new(vec![
            ("calculate".to_string(), vec!["number".to_string(), "number".to_string()]),
        ]);

        let input = "const calculate = (a, b) => a + b;";
        let output = transformer.transform(input).unwrap();

        assert!(output.contains("a: number"));
        assert!(output.contains("b: number"));
    }
}
