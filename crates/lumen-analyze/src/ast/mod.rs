//! # AST Infrastructure
//!
//! Abstract Syntax Tree parsing and analysis infrastructure using tree-sitter.

pub mod language;
pub mod parser;
pub mod query;
pub mod node;
pub mod traversal;

pub use parser::Parser;
pub use query::{Query, QueryBuilder, QueryMatch};
pub use node::{AstNode, NodeKind};
pub use traversal::{Traversal, TraversalOrder};

pub use lumen_core::LumenResult as Result;

/// Supported languages for AST parsing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AstLanguage {
    TypeScript,
    JavaScript,
    Tsx,
    Jsx,
    Rust,
    Css,
    Html,
    Python,
    Go,
    Unknown,
}

impl AstLanguage {
    /// Get the tree-sitter language for this AST language
    pub fn tree_sitter_language(&self) -> Result<tree_sitter::Language> {
        match self {
            AstLanguage::TypeScript => Ok(tree_sitter_typescript::language_typescript()),
            AstLanguage::JavaScript => Ok(tree_sitter_javascript::language()),
            AstLanguage::Tsx => Ok(tree_sitter_typescript::language_tsx()),
            AstLanguage::Jsx => Ok(tree_sitter_javascript::language()),
            AstLanguage::Rust => Ok(tree_sitter_rust::language()),
            AstLanguage::Css => Ok(tree_sitter_css::language()),
            AstLanguage::Html => {
                // HTML is handled via regex for now
                Err(lumen_core::LumenError::ConfigError("HTML - use regex parser".to_string()))
            }
            AstLanguage::Python => Ok(tree_sitter_python::language()),
            AstLanguage::Go => Ok(tree_sitter_go::language()),
            AstLanguage::Unknown => Err(lumen_core::LumenError::ConfigError("Unknown language".to_string())),
        }
    }

    /// Get file extensions for this language
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            AstLanguage::TypeScript => &["ts", "mts", "cts"],
            AstLanguage::JavaScript => &["js", "mjs", "cjs"],
            AstLanguage::Tsx => &["tsx"],
            AstLanguage::Jsx => &["jsx"],
            AstLanguage::Rust => &["rs"],
            AstLanguage::Css => &["css", "scss", "sass", "less"],
            AstLanguage::Html => &["html", "htm"],
            AstLanguage::Python => &["py", "pyi"],
            AstLanguage::Go => &["go"],
            AstLanguage::Unknown => &[],
        }
    }

    /// Detect AST language from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "ts" | "mts" | "cts" => AstLanguage::TypeScript,
            "js" | "mjs" | "cjs" => AstLanguage::JavaScript,
            "tsx" => AstLanguage::Tsx,
            "jsx" => AstLanguage::Jsx,
            "rs" => AstLanguage::Rust,
            "css" | "scss" | "sass" | "less" => AstLanguage::Css,
            "html" | "htm" => AstLanguage::Html,
            "py" | "pyi" => AstLanguage::Python,
            "go" => AstLanguage::Go,
            _ => AstLanguage::Unknown,
        }
    }

    /// Get the display name
    pub fn name(&self) -> &'static str {
        match self {
            AstLanguage::TypeScript => "TypeScript",
            AstLanguage::JavaScript => "JavaScript",
            AstLanguage::Tsx => "TSX",
            AstLanguage::Jsx => "JSX",
            AstLanguage::Rust => "Rust",
            AstLanguage::Css => "CSS",
            AstLanguage::Html => "HTML",
            AstLanguage::Python => "Python",
            AstLanguage::Go => "Go",
            AstLanguage::Unknown => "Unknown",
        }
    }
}

impl std::fmt::Display for AstLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
