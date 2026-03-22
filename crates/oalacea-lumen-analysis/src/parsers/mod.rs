//! Parsers module for various languages

pub mod html;
pub mod rust;
pub mod typescript;
pub mod javascript;
pub mod python;
pub mod go;
pub mod css;

// Common types
pub struct ParsedCode {
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<ImportInfo>,
    pub exports: Vec<ExportInfo>,
    pub variables: Vec<VariableInfo>,
}

pub struct FunctionInfo {
    pub name: String,
    pub params: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub line: usize,
}

pub struct ClassInfo {
    pub name: String,
    pub methods: Vec<FunctionInfo>,
    pub line: usize,
}

pub struct ImportInfo {
    pub module: String,
    pub items: Vec<String>,
    pub line: usize,
}

pub struct ExportInfo {
    pub name: String,
    pub kind: ExportKind,
    pub line: usize,
}

pub enum ExportKind {
    Function,
    Class,
    Variable,
}

pub struct VariableInfo {
    pub name: String,
    pub type_name: Option<String>,
    pub line: usize,
}

pub struct ParameterInfo {
    pub name: String,
    pub type_name: Option<String>,
}

pub trait LanguageParser {
    fn parse(&self, code: &str) -> Result<ParsedCode, Box<dyn std::error::Error>>;
}

// Placeholder implementations
pub use html::HtmlParser;
pub use rust::RustAnalyzer;
pub use typescript::TypeScriptAnalyzer;
