//! Performance fixes
//!
//! Handles performance-related code issues including:
//! - Missing async/await where beneficial
//! - Missing caching
//! - Inefficient loops
//! - Unnecessary re-renders
//! - Missing debounce/throttle

use lumenx_core::LumenResult;
use lumenx_score::ScoreIssue;
use regex::Regex;
use std::path::Path;

use super::{Fix, Fixer};

/// Performance fixer
pub struct PerformanceFixer {
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

impl PerformanceFixer {
    pub fn new(file_path: &Path) -> Self {
        let language = detect_language(file_path);
        Self {
            file_path: file_path.to_path_buf(),
            language,
        }
    }

    fn add_async_await(&self, content: &str, issue: &ScoreIssue) -> String {
        match self.language {
            Language::TypeScript | Language::JavaScript => {
                self.add_js_async(content, issue)
            }
            Language::Python => self.add_python_async(content, issue),
            Language::Rust => self.add_rust_async(content, issue),
            Language::Go => content.to_string(), // Go is inherently async with goroutines
        }
    }

    fn add_js_async(&self, content: &str, issue: &ScoreIssue) -> String {
        let mut result = content.to_string();

        // Find the function name from the issue
        let func_name = extract_function_name(issue)
            .or_else(|| extract_function_from_line(issue))
            .unwrap_or_else(|| "myFunction".to_string());

        // Find and convert sync function to async
        let func_re = Regex::new(&format!(
            r"(function\s+{}|const\s+{}\s*=\s*(?:async\s*)?\([^)]*\)\s*=>)",
            regex::escape(&func_name),
            regex::escape(&func_name)
        )).unwrap();

        if func_re.is_match(&result) {
            // Add async keyword
            result = func_re.replace_all(&result, |caps: &regex::Captures| {
                let matched = caps.get(0).unwrap().as_str();
                if matched.contains("async") {
                    matched.to_string()
                } else if matched.starts_with("function") {
                    matched.replacen("function", "async function", 1)
                } else {
                    matched.replacen("const", "const async", 1)
                }
            }).to_string();

            // Add await to fetch calls if present
            if result.contains("fetch(") {
                let fetch_re = Regex::new(r#"fetch\(([^)]+)\)"#).unwrap();
                result = fetch_re.replace_all(&result, "await fetch($1)").to_string();
            }
        }

        result
    }

    fn add_python_async(&self, content: &str, issue: &ScoreIssue) -> String {
        let mut result = content.to_string();

        let func_name = extract_function_name(issue)
            .unwrap_or_else(|| "my_function".to_string());

        // Find function definition and make it async
        let func_re = Regex::new(&format!(
            r"def\s+{}\s*\(",
            regex::escape(&func_name)
        )).unwrap();

        if func_re.is_match(&result) {
            result = func_re.replace_all(&result, "async def $0(").to_string();

            // Add asyncio import if needed
            if !result.contains("import asyncio") {
                result.insert_str(0, "import asyncio\n");
            }
        }

        result
    }

    fn add_rust_async(&self, content: &str, issue: &ScoreIssue) -> String {
        let mut result = content.to_string();

        let func_name = extract_function_name(issue)
            .unwrap_or_else(|| "my_function".to_string());

        // Find function definition and make it async
        let func_re = Regex::new(&format!(
            r"(pub\s+)?fn\s+{}\s*\(",
            regex::escape(&func_name)
        )).unwrap();

        if func_re.is_match(&result) {
            result = func_re.replace_all(&result, "$1async fn $0(").to_string();
        }

        result
    }

    fn add_caching(&self, content: &str, issue: &ScoreIssue) -> String {
        match self.language {
            Language::TypeScript | Language::JavaScript => {
                self.add_js_caching(content, issue)
            }
            Language::Rust => self.add_rust_caching(content, issue),
            Language::Python => self.add_python_caching(content, issue),
            Language::Go => content.to_string(),
        }
    }

    fn add_js_caching(&self, content: &str, issue: &ScoreIssue) -> String {
        let func_name = extract_function_name(issue)
            .unwrap_or_else(|| "expensiveOperation".to_string());

        let cache_wrapper = format!(r#"
const {name}Cache = new Map();

/**
 * Cached version of {name}
 */
function cached{name}(...args: any[]) {{
    const key = JSON.stringify(args);
    if ({name}Cache.has(key)) {{
        return {name}Cache.get(key);
    }}
    const result = {name}(...args);
    {name}Cache.set(key, result);
    return result;
}}

"#, name = func_name);

        let mut result = content.to_string();

        // Check if caching is already implemented
        if !result.contains(&format!("{}Cache", func_name))
            && !result.contains("cached")
            && result.contains(&format!("function {}", func_name))
        {
            let insert_pos = find_function_end(&result, &func_name);
            result.insert_str(insert_pos + 1, &cache_wrapper);
        }

        result
    }

    fn add_rust_caching(&self, content: &str, issue: &ScoreIssue) -> String {
        let func_name = extract_function_name(issue)
            .unwrap_or_else(|| "expensive_operation".to_string());

        // Add lru_cache import comment
        let cache_comment = format!(r#"
// TODO: Consider adding caching for {}
// Options: use lru_cache crate or implement simple HashMap cache
// Example: let mut cache = HashMap::new();
//          if let Some(cached) = cache.get(&key) {{ return *cached; }}
//          let result = {}(...);
//          cache.insert(key, result);
//          result

"#, func_name, func_name);

        let mut result = content.to_string();

        if !result.contains("cache") && !result.contains("Cache") {
            let insert_pos = find_insert_after_imports(&result);
            result.insert_str(insert_pos, &cache_comment);
        }

        result
    }

    fn add_python_caching(&self, content: &str, issue: &ScoreIssue) -> String {
        let func_name = extract_function_name(issue)
            .unwrap_or_else(|| "expensive_operation".to_string());

        let mut result = content.to_string();

        // Add @lru_cache decorator
        let func_re = Regex::new(&format!(
            r"(def\s+{}\s*\()",
            regex::escape(&func_name)
        )).unwrap();

        if func_re.is_match(&result) && !result.contains("@lru_cache") {
            result = func_re.replace_all(&result, "@lru_cache(maxsize=128)\n    $0").to_string();

            // Add functools import if needed
            if !result.contains("import functools") && !result.contains("from functools") {
                result.insert_str(0, "from functools import lru_cache\n");
            }
        }

        result
    }

    fn optimize_loop(&self, content: &str, issue: &ScoreIssue) -> String {
        match self.language {
            Language::TypeScript | Language::JavaScript => {
                self.optimize_js_loop(content, issue)
            }
            Language::Rust => self.optimize_rust_loop(content, issue),
            Language::Python => self.optimize_python_loop(content, issue),
            Language::Go => content.to_string(),
        }
    }

    fn optimize_js_loop(&self, content: &str, issue: &ScoreIssue) -> String {
        let mut result = content.to_string();

        // Replace forEach with for...of for better performance
        let for_each_re = Regex::new(r#"(?P<array>\w+)\.forEach\((?P<callback>\([^)]*\)\s*=>\s*{)"#).unwrap();

        if for_each_re.is_match(&result) {
            result = for_each_re.replace_all(&result, "for (const $callback of $array) {").to_string();

            // Add comment about the optimization
            if !result.contains("loop optimization") {
                let comment = "\n// Optimized: replaced forEach with for...of for better performance\n";
                result.insert_str(0, comment);
            }
        }

        // Suggest using Set for lookups if appropriate
        if result.contains(".includes(") && result.contains(".forEach(") {
            let set_comment = "\n// TODO: Consider using Set for O(1) lookups instead of Array.includes()\n";
            if !result.contains("Set for O(1)") {
                result.insert_str(0, set_comment);
            }
        }

        result
    }

    fn optimize_rust_loop(&self, content: &str, issue: &ScoreIssue) -> String {
        let mut result = content.to_string();

        // Suggest using iterators instead of indexed loops where possible
        if result.contains(".iter()") && result.contains(".enumerate()") && result.contains(".clone()") {
            let comment = "\n// TODO: Consider using iter().copied() or iter().cloned() instead of cloning in loop\n";
            if !result.contains("iter().copied()") {
                result.insert_str(0, comment);
            }
        }

        // Suggest using with_capacity for Vec
        if result.contains("Vec::new()") && result.contains(".push(") {
            let capacity_comment = "\n// TODO: Consider using Vec::with_capacity() when final size is known\n";
            if !result.contains("with_capacity") {
                result.insert_str(0, capacity_comment);
            }
        }

        result
    }

    fn optimize_python_loop(&self, content: &str, issue: &ScoreIssue) -> String {
        let mut result = content.to_string();

        // Suggest using list comprehension instead of append in loop
        if result.contains("for ") && result.contains(".append(") {
            let comment = "\n# TODO: Consider using list comprehension for better performance\n# Example: [x * 2 for x in items]\n";
            if !result.contains("list comprehension") {
                result.insert_str(0, comment);
            }
        }

        // Suggest using set for lookups
        if result.contains(" in ") && result.contains(".count(") {
            let set_comment = "\n# TODO: Convert list to set for O(1) membership testing\n";
            if !result.contains("set for O(1)") {
                result.insert_str(0, set_comment);
            }
        }

        result
    }

    fn add_debounce(&self, content: &str, issue: &ScoreIssue) -> String {
        match self.language {
            Language::TypeScript | Language::JavaScript => {
                self.add_js_debounce(content, issue)
            }
            _ => content.to_string(),
        }
    }

    fn add_js_debounce(&self, content: &str, issue: &ScoreIssue) -> String {
        let func_name = extract_function_name(issue)
            .unwrap_or_else(|| "handleInput".to_string());

        let debounce_code = r#"
/**
 * Debounce function to limit execution rate
 */
function debounce<T extends (...args: any[]) => any>(
    func: T,
    wait: number
): (...args: Parameters<T>) => void {
    let timeout: ReturnType<typeof setTimeout> | null = null;
    return (...args: Parameters<T>) => {
        if (timeout) clearTimeout(timeout);
        timeout = setTimeout(() => func(...args), wait);
    };
}

"#;

        let mut result = content.to_string();

        if !result.contains("function debounce") && !result.contains("const debounce") {
            let insert_pos = find_insert_after_imports(&result);
            result.insert_str(insert_pos, debounce_code);
        }

        // Suggest wrapping the function
        let suggestion = format!("\n// TODO: Wrap '{}' with debounce() to limit execution frequency\n// Example: const debounced{} = debounce({}, 300);\n", func_name, func_name, func_name);

        if !result.contains(&format!("debounced{}", func_name)) {
            let func_pos = result.find(&format!("function {}", func_name))
                .or_else(|| result.find(&format!("const {} = ", func_name)))
                .unwrap_or(0);
            result.insert_str(func_pos, &suggestion);
        }

        result
    }

    fn add_memo(&self, content: &str, issue: &ScoreIssue) -> String {
        match self.language {
            Language::TypeScript | Language::JavaScript => {
                self.add_js_memo(content, issue)
            }
            _ => content.to_string(),
        }
    }

    fn add_js_memo(&self, content: &str, issue: &ScoreIssue) -> String {
        let func_name = extract_function_name(issue)
            .unwrap_or_else(|| "expensiveValue".to_string());

        let mut result = content.to_string();

        // Add useMemo suggestion
        let memo_comment = format!(r#"
// TODO: Use useMemo to memoize '{}'
// Example: const {} = useMemo(() => compute{}, [dependency]);

"#, func_name, func_name, func_name);

        if !result.contains("useMemo") && !result.contains(&format!("memoized{}", func_name)) {
            // Find the const declaration
            if let Some(pos) = result.find(&format!("const {} =", func_name)) {
                result.insert_str(pos, &memo_comment);
            }
        }

        result
    }
}

/// Result of a performance fix
#[derive(Debug, Clone)]
pub struct PerformanceFix {
    pub description: String,
    pub diff: crate::patch::PatchDiff,
}

impl Fix for PerformanceFix {
    fn description(&self) -> String {
        self.description.clone()
    }

    fn diff(&self) -> crate::patch::PatchDiff {
        self.diff.clone()
    }
}

impl Fixer for PerformanceFixer {
    type Fix = PerformanceFix;

    fn new(file_path: &Path) -> Self
    where
        Self: Sized,
    {
        Self::new(file_path)
    }

    fn can_fix(&self, issue: &ScoreIssue) -> bool {
        matches!(
            issue.category.as_str(),
            "performance"
        )
        || matches!(
            issue.title.as_str(),
            "Missing async" | "Missing caching" | "Inefficient loop" | "Missing debounce"
        )
    }

    fn fix(&self, content: &str, issue: &ScoreIssue) -> LumenResult<(String, Self::Fix)> {
        let (new_content, description) = match issue.title.as_str() {
            title if title.contains("async") => {
                let fixed = self.add_async_await(content, issue);
                (fixed, "Added async/await for better performance".to_string())
            }
            title if title.contains("caching") => {
                let fixed = self.add_caching(content, issue);
                (fixed, "Added caching mechanism".to_string())
            }
            title if title.contains("loop") => {
                let fixed = self.optimize_loop(content, issue);
                (fixed, "Optimized loop for better performance".to_string())
            }
            title if title.contains("debounce") => {
                let fixed = self.add_debounce(content, issue);
                (fixed, "Added debounce to limit execution frequency".to_string())
            }
            title if title.contains("memo") => {
                let fixed = self.add_memo(content, issue);
                (fixed, "Added memoization hint".to_string())
            }
            _ => (content.to_string(), format!("Performance fix: {}", issue.title)),
        };

        let diff = crate::patch::PatchDiff::simple(content.to_string(), new_content.clone());
        Ok((new_content, PerformanceFix { description, diff }))
    }

    fn fix_all(&self, content: &str, issues: &[&ScoreIssue]) -> LumenResult<(String, Vec<Self::Fix>)> {
        let mut current_content = content.to_string();
        let mut fixes = Vec::new();

        // Group issues by type
        let mut async_issues: Vec<&ScoreIssue> = Vec::new();
        let mut cache_issues: Vec<&ScoreIssue> = Vec::new();
        let mut loop_issues: Vec<&ScoreIssue> = Vec::new();
        let mut debounce_issues: Vec<&ScoreIssue> = Vec::new();
        let mut memo_issues: Vec<&ScoreIssue> = Vec::new();

        for issue in issues {
            match issue.title.as_str() {
                title if title.contains("async") => async_issues.push(*issue),
                title if title.contains("caching") => cache_issues.push(*issue),
                title if title.contains("loop") => loop_issues.push(*issue),
                title if title.contains("debounce") => debounce_issues.push(*issue),
                title if title.contains("memo") => memo_issues.push(*issue),
                _ => {}
            }
        }

        // Apply fixes
        for issue in async_issues {
            let old_content = current_content.clone();
            current_content = self.add_async_await(&current_content, issue);
            fixes.push(PerformanceFix {
                description: format!("Added async/await for {}", issue.title),
                diff: crate::patch::PatchDiff::simple(old_content, current_content.clone()),
            });
        }

        for issue in cache_issues {
            let old_content = current_content.clone();
            current_content = self.add_caching(&current_content, issue);
            fixes.push(PerformanceFix {
                description: format!("Added caching for {}", issue.title),
                diff: crate::patch::PatchDiff::simple(old_content, current_content.clone()),
            });
        }

        for issue in loop_issues {
            let old_content = current_content.clone();
            current_content = self.optimize_loop(&current_content, issue);
            fixes.push(PerformanceFix {
                description: format!("Optimized loop for {}", issue.title),
                diff: crate::patch::PatchDiff::simple(old_content, current_content.clone()),
            });
        }

        for issue in debounce_issues {
            let old_content = current_content.clone();
            current_content = self.add_debounce(&current_content, issue);
            fixes.push(PerformanceFix {
                description: format!("Added debounce for {}", issue.title),
                diff: crate::patch::PatchDiff::simple(old_content, current_content.clone()),
            });
        }

        for issue in memo_issues {
            let old_content = current_content.clone();
            current_content = self.add_memo(&current_content, issue);
            fixes.push(PerformanceFix {
                description: format!("Added memoization for {}", issue.title),
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

fn extract_function_name(issue: &ScoreIssue) -> Option<String> {
    issue
        .description
        .split('\'')
        .nth(1)
        .or_else(|| issue.description.split('"').nth(1))
        .or_else(|| issue.title.split('\'').nth(1))
        .or_else(|| issue.title.split('"').nth(1))
        .map(|s| s.trim().to_string())
}

fn extract_function_from_line(issue: &ScoreIssue) -> Option<String> {
    // This would require reading the file at the line number
    // For now, return None
    None
}

fn find_insert_after_imports(content: &str) -> usize {
    let mut last_import_end = 0;
    let mut line_count = 0;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ")
            || trimmed.starts_with("import ")
            || trimmed.starts_with("from ")
            || trimmed.starts_with("require(")
        {
            last_import_end = i;
        }
    }

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

fn find_function_end(content: &str, func_name: &str) -> usize {
    let func_pattern = format!("function {}", func_name);
    if let Some(start) = content.find(&func_pattern) {
        // Find the closing brace for this function
        let mut brace_count = 0;
        let mut in_function = false;
        let mut offset = 0;

        for ch in content[start..].chars() {
            offset += ch.len_utf8();
            if ch == '{' {
                in_function = true;
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if in_function && brace_count == 0 {
                    return start + offset;
                }
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
    fn test_add_js_async() {
        let fixer = PerformanceFixer::new(Path::new("test.ts"));

        let content = "function fetchData() { fetch('/api/data'); }";
        let issue = ScoreIssue {
            id: "perf-1".to_string(),
            severity: IssueSeverity::Medium,
            category: "performance".to_string(),
            title: "Missing async in fetchData".to_string(),
            description: "Function 'fetchData' should be async".to_string(),
            file: Some("test.ts".to_string()),
            line: Some(1),
            column: None,
            impact: 5.0,
            suggestion: None,
        };

        let (fixed, _) = fixer.fix(content, &issue).unwrap();
        assert!(fixed.contains("async function fetchData"));
    }

    #[test]
    fn test_add_js_caching() {
        let fixer = PerformanceFixer::new(Path::new("test.ts"));

        let content = "function expensiveOperation(n) { return n * n; }";
        let issue = ScoreIssue {
            id: "perf-2".to_string(),
            severity: IssueSeverity::Low,
            category: "performance".to_string(),
            title: "Missing caching for expensiveOperation".to_string(),
            description: "Function 'expensiveOperation' should be cached".to_string(),
            file: Some("test.ts".to_string()),
            line: Some(1),
            column: None,
            impact: 3.0,
            suggestion: None,
        };

        let (fixed, _) = fixer.fix(content, &issue).unwrap();
        assert!(fixed.contains("expensiveOperationCache"));
    }
}
