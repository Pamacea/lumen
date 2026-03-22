//! AST traversal

use super::{AstNode, TraversalOrder};

pub struct Traversal;

impl Traversal {
    pub fn new() -> Self {
        Self
    }

    pub fn traverse<'a>(&self, _node: &AstNode<'a>, _order: TraversalOrder) -> Vec<AstNode<'a>> {
        Vec::new()
    }
}