//! Rust parser using tree-sitter

use crate::ast::{AstLanguage, Parser, Query};
use crate::ast::query::QueryExecutor;
use crate::parsers::{LanguageParser, ParsedCode, FunctionInfo, ImportInfo, ParameterInfo};
use lumen_core::LumenResult as Result;

/// Rust parser
pub struct RustParser {
    parser: Parser,
}

impl RustParser {
    /// Create a new Rust parser
    pub fn new() -> Result<Self> {
        Ok(Self {
            parser: Parser::new(AstLanguage::Rust)?,
        })
    }

    /// Extract functions from Rust code
    fn extract_functions(&mut self, tree: &tree_sitter::Tree, source: &str) -> Vec<FunctionInfo> {
        let query = Query::function(AstLanguage::Rust).unwrap();
        let mut executor = QueryExecutor::new();

        let names = executor.exec_captures(&query, tree, source, "name");

        names.into_iter().map(|name| {
            // Find the position of this function
            let pos = source.find(&name)
                .and_then(|i| source[..i].chars().filter(|&c| c == '\n').count().checked_add(1))
                .unwrap_or(0);

            FunctionInfo {
                name,
                start: (pos, 0),
                end: (0, 0),
                is_async: false,
                is_generator: false,
                parameters: Vec::new(),
                return_type: None,
                body: None,
            }
        }).collect()
    }

    /// Extract struct declarations
    pub fn extract_structs(&mut self, tree: &tree_sitter::Tree, source: &str) -> Vec<StructInfo> {
        let mut structs = Vec::new();

        // Simple pattern matching for struct declarations
        for (i, line) in source.lines().enumerate() {
            if line.starts_with("pub struct ") || line.starts_with("struct ") {
                let rest = line.trim_start_matches("pub struct ").trim_start_matches("struct ");
                if let Some(name) = rest.split('<').next().or_else(|| rest.split('{').next()) {
                    structs.push(StructInfo {
                        name: name.trim().to_string(),
                        line: i + 1,
                        is_pub: line.starts_with("pub struct "),
                        fields: Vec::new(),
                    });
                }
            }
        }

        structs
    }

    /// Extract use declarations
    pub fn extract_uses(&mut self, tree: &tree_sitter::Tree, source: &str) -> Vec<UseInfo> {
        let mut uses = Vec::new();

        for (i, line) in source.lines().enumerate() {
            if line.starts_with("use ") {
                let rest = line.trim_start_matches("use ");
                let path = rest.split(';').next().unwrap_or(rest).trim();
                uses.push(UseInfo {
                    path: path.to_string(),
                    is_wildcard: path.contains("::*"),
                    line: i + 1,
                });
            }
        }

        uses
    }

    /// Check for unsafe blocks
    pub fn find_unsafe(&self, source: &str) -> Vec<UnsafeBlock> {
        let mut unsafes = Vec::new();

        for (i, line) in source.lines().enumerate() {
            if line.contains("unsafe") {
                unsafes.push(UnsafeBlock {
                    line: i + 1,
                    contains_raw_pointer: line.contains("*mut") || line.contains("*const"),
                    contains_external_call: line.contains("extern"),
                });
            }
        }

        unsafes
    }
}

impl Default for RustParser {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl LanguageParser for RustParser {
    fn parse(&mut self, source: &str) -> Result<ParsedCode> {
        let tree = self.parser.parse(source)?;

        let functions = self.extract_functions(&tree, source);
        let classes = Vec::new(); // Rust uses structs instead of classes
        let imports = Vec::new(); // Rust uses 'use' instead of 'import'
        let exports = Vec::new();
        let variables = Vec::new();

        let line_count = source.lines().count();
        let blank_lines = source.lines().filter(|l| l.trim().is_empty()).count();
        let comment_lines = source.lines().filter(|l| l.trim().starts_with("//")).count();

        Ok(ParsedCode {
            functions,
            classes,
            imports,
            exports,
            variables,
            line_count,
            blank_lines,
            comment_lines,
        })
    }

    fn language(&self) -> crate::ast::AstLanguage {
        AstLanguage::Rust
    }
}

/// Rust-specific analyzer
pub struct RustAnalyzer;

impl RustAnalyzer {
    /// Analyze Rust code quality
    pub fn analyze(source: &str) -> Result<RustAnalysis> {
        let mut analysis = RustAnalysis::default();

        // Count unsafe blocks
        let mut parser = RustParser::new()?;
        let tree = parser.parser.parse(source)?;
        analysis.unsafe_count = parser.find_unsafe(source).len();

        // Check for common patterns
        analysis.has_derive_debug = source.contains("#[derive(Debug)]");
        analysis.has_tests = source.contains("#[cfg(test)]");

        // Count public items
        analysis.public_items = source.matches("pub fn").count()
            + source.matches("pub struct").count()
            + source.matches("pub enum").count();

        // Check for documentation
        let doc_lines = source.lines().filter(|l| l.trim().starts_with("///")).count();
        analysis.doc_coverage = if doc_lines > 0 {
            (doc_lines as f64 / source.lines().count() as f64) * 100.0
        } else {
            0.0
        };

        Ok(analysis)
    }
}

/// Information about a Rust struct
#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub line: usize,
    pub is_pub: bool,
    pub fields: Vec<FieldInfo>,
}

/// Struct field
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub type_annotation: Option<String>,
    pub is_pub: bool,
}

/// Information about a use declaration
#[derive(Debug, Clone)]
pub struct UseInfo {
    pub path: String,
    pub is_wildcard: bool,
    pub line: usize,
}

/// Unsafe block information
#[derive(Debug, Clone)]
pub struct UnsafeBlock {
    pub line: usize,
    pub contains_raw_pointer: bool,
    pub contains_external_call: bool,
}

/// Rust code analysis results
#[derive(Debug, Clone, Default)]
pub struct RustAnalysis {
    pub unsafe_count: usize,
    pub has_derive_debug: bool,
    pub has_tests: bool,
    pub public_items: usize,
    pub doc_coverage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rust_function() {
        let mut parser = RustParser::new().unwrap();
        let source = r#"
pub fn hello() -> String {
    "Hello".to_string()
}
"#;
        let result = parser.parse(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.functions.len(), 1);
    }

    #[test]
    fn test_find_unsafe() {
        let parser = RustParser::new().unwrap();
        let source = r#"
fn main() {
    unsafe { println!("Hello"); }
}
"#;
        let unsafes = parser.find_unsafe(source);
        assert_eq!(unsafes.len(), 1);
    }

    #[test]
    fn test_extract_structs() {
        let mut parser = RustParser::new().unwrap();
        let tree = parser.parser.parse("pub struct Test;").unwrap();
        let structs = parser.extract_structs(&tree, "pub struct Test;");
        assert_eq!(structs.len(), 1);
        assert_eq!(structs[0].name, "Test");
        assert!(structs[0].is_pub);
    }
}
