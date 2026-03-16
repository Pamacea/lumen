//! TypeScript/JavaScript parser using tree-sitter

use crate::ast::{AstLanguage, Parser, Query};
use crate::ast::query::QueryExecutor;
use crate::parsers::{LanguageParser, ParsedCode, FunctionInfo, ClassInfo, ImportInfo, ExportInfo, VariableInfo, ParameterInfo};
use lumenx_core::LumenResult as Result;

/// TypeScript/JavaScript parser
pub struct TypeScriptParser {
    parser: Parser,
    language: AstLanguage,
}

impl TypeScriptParser {
    /// Create a new TypeScript parser
    pub fn new() -> Result<Self> {
        Ok(Self {
            parser: Parser::new(AstLanguage::TypeScript)?,
            language: AstLanguage::TypeScript,
        })
    }

    /// Create a parser that specifically handles TSX
    pub fn tsx() -> Result<Self> {
        Ok(Self {
            parser: Parser::new(AstLanguage::Tsx)?,
            language: AstLanguage::Tsx,
        })
    }

    /// Extract functions using tree-sitter queries
    fn extract_functions(&mut self, tree: &tree_sitter::Tree, source: &str) -> Vec<FunctionInfo> {
        let query = Query::function(self.language).unwrap();
        let mut executor = QueryExecutor::new();

        let matches = executor.exec(&query, tree, source);

        matches.into_iter().map(|m| {
            let name = m.captures.get("name")
                .map(|c| c.text.clone())
                .unwrap_or_else(|| String::from("<anonymous>"));

            let start = m.captures.get("name")
                .map(|c| c.start)
                .unwrap_or((0, 0));

            FunctionInfo {
                name,
                start,
                end: (0, 0), // TODO: extract end position
                is_async: false,
                is_generator: false,
                parameters: Vec::new(),
                return_type: None,
                body: None,
            }
        }).collect()
    }

    /// Extract imports
    fn extract_imports(&mut self, tree: &tree_sitter::Tree, source: &str) -> Vec<ImportInfo> {
        // Simplified implementation
        // Full implementation would parse the import statement structure
        Vec::new()
    }
}

impl Default for TypeScriptParser {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl LanguageParser for TypeScriptParser {
    fn parse(&mut self, source: &str) -> Result<ParsedCode> {
        let tree = self.parser.parse(source)?;

        let functions = self.extract_functions(&tree, source);
        let classes = Vec::new(); // TODO: implement
        let imports = self.extract_imports(&tree, source);
        let exports = Vec::new(); // TODO: implement
        let variables = Vec::new(); // TODO: implement

        let line_count = source.lines().count();
        let blank_lines = source.lines().filter(|l| l.trim().is_empty()).count();
        let comment_lines = 0; // TODO: count comments

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
        self.language
    }
}

/// TypeScript analyzer for code patterns
pub struct TypeScriptAnalyzer;

impl TypeScriptAnalyzer {
    /// Check for TypeScript best practices
    pub fn analyze_best_practices(source: &str) -> Result<Vec<String>> {
        let mut findings = Vec::new();

        // Check for use of 'any' type
        if source.contains(": any") || source.contains(":<any>") {
            findings.push("Usage of 'any' type detected".to_string());
        }

        // Check for missing return types
        if source.matches("function").count() > 0 && !source.matches("=>").all(|s| s.contains(":")) {
            findings.push("Functions may be missing return type annotations".to_string());
        }

        Ok(findings)
    }

    /// Extract React component info
    pub fn extract_components(source: &str) -> Result<Vec<ComponentInfo>> {
        let mut components = Vec::new();

        // Simple regex-based extraction (full version would use AST)
        for line in source.lines() {
            if line.contains("export default function") || line.contains("export default const") {
                if let Some(name) = line.split("function").nth(1)
                    .or_else(|| line.split("const").nth(1)) {
                    let name = name.split('(').next()
                        .map(|s| s.trim())
                        .unwrap_or("Unknown")
                        .to_string();
                    components.push(ComponentInfo {
                        name,
                        component_type: ComponentType::FunctionComponent,
                        hooks: Vec::new(),
                        props: Vec::new(),
                    });
                }
            }
        }

        Ok(components)
    }
}

/// React component information
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub name: String,
    pub component_type: ComponentType,
    pub hooks: Vec<String>,
    pub props: Vec<PropInfo>,
}

/// Type of React component
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentType {
    FunctionComponent,
    ClassComponent,
}

/// Component prop
#[derive(Debug, Clone)]
pub struct PropInfo {
    pub name: String,
    pub type_annotation: Option<String>,
    pub is_required: bool,
    pub default_value: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_function() {
        let mut parser = TypeScriptParser::new().unwrap();
        let source = r#"
function hello(name: string): string {
    return "Hello, " + name;
}
"#;
        let result = parser.parse(source);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert!(!parsed.functions.is_empty());
    }

    #[test]
    fn test_analyze_any_usage() {
        let findings = TypeScriptAnalyzer::analyze_best_practices("let x: any = 5;").unwrap();
        assert_eq!(findings.len(), 1);
        assert!(findings[0].contains("any"));
    }
}
