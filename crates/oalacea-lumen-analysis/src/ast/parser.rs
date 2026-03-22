//! AST parser for various languages

use super::{AstLanguage, AstNode, Result};

pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse<'a>(&self, code: &'a str, _language: AstLanguage) -> Result<AstNode<'a>> {
        Ok(AstNode::from_code(code))
    }
}
