//! Core AST parser using tree-sitter

use crate::ast::{AstLanguage, AstNode, Result};
use std::path::Path;
use tree_sitter::{Parser as TsParser, Tree, Node};

/// AST parser for source code
pub struct Parser {
    /// Tree-sitter parser
    parser: TsParser,
    /// Language being parsed
    language: AstLanguage,
}

impl Parser {
    /// Create a new parser for the given language
    pub fn new(language: AstLanguage) -> Result<Self> {
        let mut parser = TsParser::new();
        let ts_lang = language.tree_sitter_language()?;
        parser.set_language(&ts_lang)
            .map_err(|e| lumen_core::LumenError::ConfigError(format!("Failed to set language: {}", e)))?;

        Ok(Self {
            parser,
            language,
        })
    }

    /// Create a parser from file extension
    pub fn from_extension(ext: &str) -> Result<Self> {
        Self::new(AstLanguage::from_extension(ext))
    }

    /// Parse source code string
    pub fn parse(&mut self, source: &str) -> Result<Tree> {
        self.parser
            .parse(source, None)
            .ok_or_else(|| lumen_core::LumenError::ParseError("Failed to parse source code".to_string()))
    }

    /// Parse a file
    pub fn parse_file(&mut self, path: &Path) -> Result<(Tree, String)> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| lumen_core::LumenError::Io(e))?;
        let tree = self.parse(&source)?;
        Ok((tree, source))
    }

    /// Get the language
    pub fn language(&self) -> AstLanguage {
        self.language
    }

    /// Get the root node of the last parsed tree
    pub fn root_node<'a>(&self, tree: &'a Tree) -> AstNode<'a> {
        AstNode::new(tree.root_node(), self.language)
    }
}

impl Clone for Parser {
    fn clone(&self) -> Self {
        Self::new(self.language).unwrap_or_else(|_| {
            // Fallback to a simple parser if cloning fails
            let mut parser = TsParser::new();
            let rust_lang = tree_sitter_rust::language();
            let _ = parser.set_language(&rust_lang);
            Self {
                parser,
                language: AstLanguage::Rust,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = Parser::new(AstLanguage::Rust);
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parse_simple_rust() {
        let mut parser = Parser::new(AstLanguage::Rust).unwrap();
        let source = "fn main() { println!(\"Hello\"); }";
        let tree = parser.parse(source);
        assert!(tree.is_ok());
    }

    #[test]
    fn test_language_from_extension() {
        assert_eq!(AstLanguage::from_extension("rs"), AstLanguage::Rust);
        assert_eq!(AstLanguage::from_extension("ts"), AstLanguage::TypeScript);
        assert_eq!(AstLanguage::from_extension("tsx"), AstLanguage::Tsx);
        assert_eq!(AstLanguage::from_extension("css"), AstLanguage::Css);
        assert_eq!(AstLanguage::from_extension("html"), AstLanguage::Html);
    }
}
