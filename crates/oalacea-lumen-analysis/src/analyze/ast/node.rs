//! AST node representation

use crate::analyze::ast::AstLanguage;

/// AST node with lifetime parameter
#[derive(Clone)]
pub struct AstNode<'a> {
    pub code: &'a str,
    pub language: AstLanguage,
    pub _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> AstNode<'a> {
    /// Create a new AST node from code
    pub fn from_code(code: &'a str, language: AstLanguage) -> Self {
        Self {
            code,
            language,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get the node kind (type)
    pub fn kind(&self) -> &'static str {
        "placeholder"
    }

    /// Get named children
    pub fn named_children(&self, _query: &str) -> Vec<AstNode<'a>> {
        Vec::new()
    }
}

/// Node kind/type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Function,
    Class,
    Variable,
    Parameter,
    Unknown,
}
