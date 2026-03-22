//! Import/Export fixes
//!
//! Handles common issues related to imports and exports including:
//! - Unused imports
//! - Unsorted imports
//! - Missing exports
//! - Duplicate imports

use lumenx_core::LumenResult;
use lumenx_score::ScoreIssue;
use regex::Regex;
use std::path::Path;

use super::{Fix, Fixer};

/// Import fixer
pub struct ImportFixer {
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

impl ImportFixer {
    pub fn new(file_path: &Path) -> Self {
        let language = detect_language(file_path);
        Self {
            file_path: file_path.to_path_buf(),
            language,
        }
    }

    fn remove_unused_import(&self, content: &str, import_name: &str) -> String {
        match self.language {
            Language::Rust => self.remove_rust_import(content, import_name),
            Language::TypeScript | Language::JavaScript => {
                self.remove_ts_import(content, import_name)
            }
            Language::Python => self.remove_python_import(content, import_name),
            Language::Go => content.to_string(), // Go doesn't have unused imports typically
        }
    }

    fn remove_rust_import(&self, content: &str, import_name: &str) -> String {
        // Remove various forms of use statements
        let patterns = [
            format!("use {};", import_name),
            format!("use {}::*;", import_name),
            format!("use {} as ", import_name),
            format!("use {{\n    {}\n}};", import_name),
        ];

        let mut result = content.to_string();
        for pattern in &patterns {
            result = result.replace(pattern, "");
        }

        // Clean up extra blank lines
        clean_extra_blank_lines(&result)
    }

    fn remove_ts_import(&self, content: &str, import_name: &str) -> String {
        let patterns = [
            format!("import {} from", import_name),
            format!("import {{ {} }}", import_name),
            format!("import {{ {} as ", import_name),
            format!("import {{ {}, ", import_name),
            format!(", {} }}", import_name),
            format!("import * as {} from", import_name),
        ];

        let mut result = content.to_string();

        // Try to match complete import lines using regex
        for pattern in &patterns {
            // Escape special regex characters
            let escaped = regex::escape(&pattern.replace("from ", ""));

            if pattern.contains("import * as") {
                let re = Regex::new(&format!(r#"import \* as {} from ['"].*?['"];"#, import_name))
                    .unwrap_or_else(|_| Regex::new(&format!("import \\* as {}", regex::escape(import_name))).unwrap());
                result = re.replace_all(&result, "").to_string();
            } else if pattern.contains('{') {
                // Handle named imports
                let re = Regex::new(&format!(
                    r#"import\s*\{{[^}}]*?{}[^}}]*?\}}\s*from\s*['"].*?['"];"#,
                    regex::escape(import_name)
                )).unwrap_or_else(|_| Regex::new(r"import.*").unwrap());
                if result.contains(import_name) {
                    // Simple fallback: just try to remove the line
                    result = result
                        .lines()
                        .filter(|line| !line.contains(&format!("import {{ {}", import_name))
                            && !line.contains(&format!(", {}", import_name)))
                        .collect::<Vec<_>>()
                        .join("\n");
                }
            } else {
                result = result.replace(pattern, "");
            }
        }

        clean_extra_blank_lines(&result)
    }

    fn remove_python_import(&self, content: &str, import_name: &str) -> String {
        let patterns = [
            format!("import {}", import_name),
            format!("from {} import", import_name),
        ];

        let mut result = content.to_string();
        for pattern in &patterns {
            result = result.replace(&format!("{}\n", pattern), "");
        }

        clean_extra_blank_lines(&result)
    }

    fn sort_imports(&self, content: &str) -> String {
        match self.language {
            Language::Rust => self.sort_rust_imports(content),
            Language::TypeScript | Language::JavaScript => self.sort_ts_imports(content),
            Language::Python => self.sort_python_imports(content),
            Language::Go => self.sort_go_imports(content),
        }
    }

    fn sort_rust_imports(&self, content: &str) -> String {
        let mut lines: Vec<&str> = content.lines().collect();
        let mut imports_start = None;
        let mut imports_end = None;

        // Find the import block
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("use ") && imports_start.is_none() {
                imports_start = Some(i);
            }
            if imports_start.is_some()
                && imports_end.is_none()
                && !trimmed.starts_with("use ")
                && !trimmed.is_empty()
            {
                imports_end = Some(i);
                break;
            }
        }

        if let (Some(start), Some(end)) = (imports_start, imports_end) {
            let mut imports: Vec<&str> = lines[start..end].to_vec();
            imports.sort();
            imports.dedup();

            let mut result = lines[..start].to_vec();
            result.extend(imports);
            result.extend(lines[end..].to_vec());
            result.join("\n")
        } else {
            content.to_string()
        }
    }

    fn sort_ts_imports(&self, content: &str) -> String {
        let mut imports = Vec::new();
        let mut other_lines = Vec::new();
        let mut in_import = false;
        let mut current_import = String::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                in_import = true;
                current_import.push_str(line);
                current_import.push('\n');
            } else if in_import {
                current_import.push_str(line);
                current_import.push('\n');
                if trimmed.ends_with(';') || trimmed.ends_with('}') {
                    in_import = false;
                    imports.push(current_import.clone());
                    current_import.clear();
                }
            } else {
                other_lines.push(line);
            }
        }

        imports.sort();
        imports.dedup();

        let mut result = String::new();
        for import in &imports {
            result.push_str(import);
            if !import.ends_with('\n') {
                result.push('\n');
            }
        }
        if !imports.is_empty() && !other_lines.is_empty() {
            result.push('\n');
        }
        for line in other_lines {
            result.push_str(line);
            result.push('\n');
        }

        result
    }

    fn sort_python_imports(&self, content: &str) -> String {
        let mut std_imports = Vec::<String>::new();
        let mut third_party = Vec::<String>::new();
        let mut local_imports = Vec::<String>::new();
        let mut other_lines = Vec::<String>::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") {
                std_imports.push(line.to_string());
            } else if trimmed.starts_with("from ") {
                if trimmed.contains("from .") || trimmed.contains("from ..") {
                    local_imports.push(line.to_string());
                } else {
                    std_imports.push(line.to_string());
                }
            } else if !std_imports.is_empty()
                || !local_imports.is_empty()
                || !third_party.is_empty()
            {
                // After we've seen imports, everything else is "other"
                other_lines.push(line.to_string());
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
        if !std_imports.is_empty() || !local_imports.is_empty() {
            result.push('\n');
        }
        for line in &other_lines {
            result.push_str(line);
            result.push('\n');
        }

        result
    }

    fn sort_go_imports(&self, content: &str) -> String {
        // Go has a specific import block format
        let import_re = Regex::new(r"(?ms)import \((.*?)\)").unwrap();

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

        result.to_string()
    }

    fn add_export(&self, content: &str, item_name: &str) -> String {
        match self.language {
            Language::Rust => {
                // Add pub keyword
                content.replace(&format!("fn {}", item_name), &format!("pub fn {}", item_name))
                    .replace(&format!("struct {}", item_name), &format!("pub struct {}", item_name))
                    .replace(&format!("enum {}", item_name), &format!("pub enum {}", item_name))
                    .replace(&format!("const {}", item_name), &format!("pub const {}", item_name))
                    .replace(&format!("static {}", item_name), &format!("pub static {}", item_name))
            }
            Language::TypeScript | Language::JavaScript => {
                // Add export keyword
                content.replace(&format!("function {}", item_name), &format!("export function {}", item_name))
                    .replace(&format!("const {}", item_name), &format!("export const {}", item_name))
                    .replace(&format!("class {}", item_name), &format!("export class {}", item_name))
                    .replace(&format!("interface {}", item_name), &format!("export interface {}", item_name))
                    .replace(&format!("type {}", item_name), &format!("export type {}", item_name))
            }
            Language::Python => {
                // Python doesn't have exports per se, but we can add to __all__
                if content.contains("__all__") {
                    content.replace(
                        &format!("__all__ = ["),
                        &format!("__all__ = [\n    \"{}\",", item_name)
                    )
                } else {
                    format!("__all__ = [\"{}\"]\n\n{}", item_name, content)
                }
            }
            Language::Go => {
                // Go exports are capitalization-based
                content.replace(&format!("func {}", item_name), &format!("func {}", capitalize(item_name)))
                    .replace(&format!("type {}", item_name), &format!("type {}", capitalize(item_name)))
                    .replace(&format!("const {}", item_name), &format!("const {}", capitalize(item_name)))
            }
        }
    }
}

/// Result of an import fix
#[derive(Debug, Clone)]
pub struct ImportFix {
    pub description: String,
    pub diff: crate::patch::PatchDiff,
}

impl Fix for ImportFix {
    fn description(&self) -> String {
        self.description.clone()
    }

    fn diff(&self) -> crate::patch::PatchDiff {
        self.diff.clone()
    }
}

impl Fixer for ImportFixer {
    type Fix = ImportFix;

    fn new(file_path: &Path) -> Self
    where
        Self: Sized,
    {
        Self::new(file_path)
    }

    fn can_fix(&self, issue: &ScoreIssue) -> bool {
        matches!(
            issue.category.as_str(),
            "unused-import" | "missing-export" | "unsorted-imports" | "duplicate-import"
        )
    }

    fn fix(&self, content: &str, issue: &ScoreIssue) -> LumenResult<(String, Self::Fix)> {
        let (new_content, description) = match issue.category.as_str() {
            "unused-import" => {
                let import_name = extract_import_name(issue)?;
                let fixed = self.remove_unused_import(content, &import_name);
                (fixed, format!("Removed unused import: {}", import_name))
            }
            "unsorted-imports" => {
                let fixed = self.sort_imports(content);
                (fixed, "Sorted imports alphabetically".to_string())
            }
            "missing-export" => {
                let item_name = extract_item_name(issue)?;
                let fixed = self.add_export(content, &item_name);
                (fixed, format!("Added export for: {}", item_name))
            }
            _ => (content.to_string(), "Unknown import issue".to_string()),
        };

        let diff = crate::patch::PatchDiff::simple(content.to_string(), new_content.clone());
        Ok((new_content, ImportFix { description, diff }))
    }

    fn fix_all(&self, content: &str, issues: &[&ScoreIssue]) -> LumenResult<(String, Vec<Self::Fix>)> {
        let mut current_content = content.to_string();
        let mut fixes = Vec::new();

        // Group issues by type
        let mut unused_imports: Vec<&ScoreIssue> = Vec::new();
        let mut missing_exports: Vec<&ScoreIssue> = Vec::new();
        let mut has_unsorted = false;

        for issue in issues {
            match issue.category.as_str() {
                "unused-import" => unused_imports.push(*issue),
                "missing-export" => missing_exports.push(*issue),
                "unsorted-imports" => has_unsorted = true,
                _ => {}
            }
        }

        // Remove all unused imports at once
        if !unused_imports.is_empty() {
            let mut imports_to_remove = Vec::new();
            for issue in &unused_imports {
                if let Ok(name) = extract_import_name(issue) {
                    imports_to_remove.push(name);
                }
            }

            for import_name in &imports_to_remove {
                let old_content = current_content.clone();
                current_content = self.remove_unused_import(&current_content, import_name);
                fixes.push(ImportFix {
                    description: format!("Removed unused import: {}", import_name),
                    diff: crate::patch::PatchDiff::simple(old_content, current_content.clone()),
                });
            }
        }

        // Sort imports once if needed
        if has_unsorted {
            let old_content = current_content.clone();
            current_content = self.sort_imports(&current_content);
            fixes.push(ImportFix {
                description: "Sorted imports alphabetically".to_string(),
                diff: crate::patch::PatchDiff::simple(old_content, current_content.clone()),
            });
        }

        // Add exports
        for issue in &missing_exports {
            let old_content = current_content.clone();
            if let Ok(item_name) = extract_item_name(issue) {
                current_content = self.add_export(&current_content, &item_name);
                fixes.push(ImportFix {
                    description: format!("Added export for: {}", item_name),
                    diff: crate::patch::PatchDiff::simple(old_content, current_content.clone()),
                });
            }
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

fn extract_import_name(issue: &ScoreIssue) -> LumenResult<String> {
    issue
        .description
        .split_whitespace()
        .last()
        .or_else(|| issue.description.split('\'').nth(1))
        .or_else(|| issue.description.split('"').nth(1))
        .map(|s| s.trim_end_matches([';', ',', '.']).to_string())
        .ok_or_else(|| {
            lumenx_core::LumenError::FixFailed(format!(
                "Could not extract import name from: {}",
                issue.description
            ))
        })
}

fn extract_item_name(issue: &ScoreIssue) -> LumenResult<String> {
    issue
        .description
        .split_whitespace()
        .last()
        .or_else(|| issue.description.split('\'').nth(1))
        .or_else(|| issue.description.split('"').nth(1))
        .map(|s| s.trim_end_matches([';', ',', '.']).to_string())
        .ok_or_else(|| {
            lumenx_core::LumenError::FixFailed(format!(
                "Could not extract item name from: {}",
                issue.description
            ))
        })
}

fn clean_extra_blank_lines(content: &str) -> String {
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

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumenx_score::IssueSeverity;

    #[test]
    fn test_remove_unused_import_rust() {
        let fixer = ImportFixer::new(Path::new("test.rs"));

        let content = "use std::collections::HashMap;\n\nfn main() {}";
        let issue = ScoreIssue {
            id: "test-1".to_string(),
            severity: IssueSeverity::Low,
            category: "unused-import".to_string(),
            title: "Unused import".to_string(),
            description: "Unused import: std::collections::HashMap".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(1),
            column: None,
            impact: 1.0,
            suggestion: None,
        };

        let (fixed, fix_result) = fixer.fix(content, &issue).unwrap();
        assert!(!fixed.contains("use std::collections::HashMap"));
        assert_eq!(fix_result.description, "Removed unused import: std::collections::HashMap");
    }

    #[test]
    fn test_sort_rust_imports() {
        let fixer = ImportFixer::new(Path::new("test.rs"));

        let content = "use std::b;\nuse std::a;\n\nfn main() {}";
        let issue = ScoreIssue {
            id: "test-2".to_string(),
            severity: IssueSeverity::Info,
            category: "unsorted-imports".to_string(),
            title: "Unsorted imports".to_string(),
            description: "Imports should be sorted".to_string(),
            file: Some("test.rs".to_string()),
            line: None,
            column: None,
            impact: 0.0,
            suggestion: None,
        };

        let (fixed, fix_result) = fixer.fix(content, &issue).unwrap();
        let lines: Vec<&str> = fixed.lines().collect();
        assert!(lines[0].contains("std::a"));
        assert!(lines[1].contains("std::b"));
    }

    #[test]
    fn test_add_export_rust() {
        let fixer = ImportFixer::new(Path::new("test.rs"));

        let content = "fn helper() {}\n\nfn main() {}";
        let issue = ScoreIssue {
            id: "test-3".to_string(),
            severity: IssueSeverity::Medium,
            category: "missing-export".to_string(),
            title: "Missing export".to_string(),
            description: "Function 'helper' should be exported".to_string(),
            file: Some("test.rs".to_string()),
            line: Some(1),
            column: None,
            impact: 5.0,
            suggestion: None,
        };

        let (fixed, fix_result) = fixer.fix(content, &issue).unwrap();
        assert!(fixed.contains("pub fn helper"));
        assert_eq!(fix_result.description, "Added export for: helper");
    }
}
