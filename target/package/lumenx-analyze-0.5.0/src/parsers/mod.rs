//! Specialized parsers for different languages

pub mod typescript;
pub mod rust;
pub mod css;
pub mod html;

pub use typescript::{TypeScriptParser, TypeScriptAnalyzer};
pub use rust::{RustParser, RustAnalyzer};
pub use css::{CssParser, CssAnalyzer};
pub use html::{HtmlParser, HtmlAnalyzer};

use lumenx_core::LumenResult;

/// Trait for language-specific parsers
pub trait LanguageParser {
    /// Parse source code and extract semantic information
    fn parse(&mut self, source: &str) -> LumenResult<ParsedCode>;

    /// Get the language this parser handles
    fn language(&self) -> crate::ast::AstLanguage;
}

/// Result of parsing source code
#[derive(Debug, Clone)]
pub struct ParsedCode {
    /// Function declarations found
    pub functions: Vec<FunctionInfo>,
    /// Class declarations found
    pub classes: Vec<ClassInfo>,
    /// Import statements found
    pub imports: Vec<ImportInfo>,
    /// Export statements found
    pub exports: Vec<ExportInfo>,
    /// Variables declared at top level
    pub variables: Vec<VariableInfo>,
    /// Number of lines
    pub line_count: usize,
    /// Number of blank lines
    pub blank_lines: usize,
    /// Number of comment lines
    pub comment_lines: usize,
}

/// Information about a function
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    /// Function name
    pub name: String,
    /// Start position (line, column)
    pub start: (usize, usize),
    /// End position (line, column)
    pub end: (usize, usize),
    /// Whether it's async
    pub is_async: bool,
    /// Whether it's a generator
    pub is_generator: bool,
    /// Parameters
    pub parameters: Vec<ParameterInfo>,
    /// Return type annotation
    pub return_type: Option<String>,
    /// Body text
    pub body: Option<String>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    /// Parameter name
    pub name: String,
    /// Type annotation
    pub type_annotation: Option<String>,
    /// Default value
    pub default_value: Option<String>,
}

/// Information about a class
#[derive(Debug, Clone)]
pub struct ClassInfo {
    /// Class name
    pub name: String,
    /// Start position
    pub start: (usize, usize),
    /// End position
    pub end: (usize, usize),
    /// Base class names
    pub extends: Vec<String>,
    /// Implemented interfaces
    pub implements: Vec<String>,
    /// Methods
    pub methods: Vec<FunctionInfo>,
    /// Properties
    pub properties: Vec<PropertyInfo>,
}

/// Class property
#[derive(Debug, Clone)]
pub struct PropertyInfo {
    /// Property name
    pub name: String,
    /// Type annotation
    pub type_annotation: Option<String>,
    /// Whether it's static
    pub is_static: bool,
    /// Visibility (public, private, protected)
    pub visibility: Option<String>,
}

/// Information about an import
#[derive(Debug, Clone)]
pub struct ImportInfo {
    /// Module path being imported
    pub module: String,
    /// Names being imported
    pub names: Vec<String>,
    /// Whether it's a default import
    pub is_default: bool,
    /// Whether it's a wildcard import
    pub is_wildcard: bool,
    /// Alias if any
    pub alias: Option<String>,
    /// Position in file
    pub position: (usize, usize),
}

/// Information about an export
#[derive(Debug, Clone)]
pub struct ExportInfo {
    /// Name being exported
    pub name: String,
    /// Whether it's a default export
    pub is_default: bool,
    /// Whether it's exported from another module (re-export)
    pub is_reexport: bool,
    /// Source module if re-export
    pub source_module: Option<String>,
    /// Position in file
    pub position: (usize, usize),
}

/// Variable declaration
#[derive(Debug, Clone)]
pub struct VariableInfo {
    /// Variable name
    pub name: String,
    /// Type annotation
    pub type_annotation: Option<String>,
    /// Initial value
    pub initial_value: Option<String>,
    /// Whether it's constant
    pub is_const: bool,
    /// Visibility
    pub visibility: Option<String>,
    /// Position
    pub position: (usize, usize),
}
