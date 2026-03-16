//! AST traversal utilities

use crate::ast::AstNode;

/// Traversal order for tree walking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalOrder {
    /// Pre-order (parent before children)
    Pre,
    /// Post-order (children before parent)
    Post,
    /// Level-order (breadth-first)
    Level,
}

/// AST traversal iterator
pub struct Traversal<'a> {
    nodes: Vec<AstNode<'a>>,
    index: usize,
    order: TraversalOrder,
}

impl<'a> Traversal<'a> {
    /// Create a new traversal from a root node
    pub fn new(root: AstNode<'a>, order: TraversalOrder) -> Self {
        let mut nodes = Vec::new();

        match order {
            TraversalOrder::Pre => {
                Self::collect_pre(root, &mut nodes);
            }
            TraversalOrder::Post => {
                Self::collect_post(root, &mut nodes);
            }
            TraversalOrder::Level => {
                Self::collect_level(root, &mut nodes);
            }
        }

        Self {
            nodes,
            index: 0,
            order,
        }
    }

    fn collect_pre(node: AstNode<'a>, acc: &mut Vec<AstNode<'a>>) {
        acc.push(node);
        for child in node.named_children("") {
            Self::collect_pre(child, acc);
        }
    }

    fn collect_post(node: AstNode<'a>, acc: &mut Vec<AstNode<'a>>) {
        for child in node.named_children("") {
            Self::collect_post(child, acc);
        }
        acc.push(node);
    }

    fn collect_level(root: AstNode<'a>, acc: &mut Vec<AstNode<'a>>) {
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(root);

        while let Some(node) = queue.pop_front() {
            acc.push(node);
            for child in node.named_children("") {
                queue.push_back(child);
            }
        }
    }
}

impl<'a> Iterator for Traversal<'a> {
    type Item = AstNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.nodes.len() {
            let node = self.nodes[self.index];
            self.index += 1;
            Some(node)
        } else {
            None
        }
    }
}

/// Find nodes matching a predicate
pub struct Finder<'a, F> {
    traversal: Traversal<'a>,
    predicate: F,
}

impl<'a, F> Finder<'a, F>
where
    F: Fn(&AstNode<'a>) -> bool,
{
    /// Create a new finder
    pub fn new(root: AstNode<'a>, predicate: F) -> Self {
        Self {
            traversal: Traversal::new(root, TraversalOrder::Pre),
            predicate,
        }
    }
}

impl<'a, F> Iterator for Finder<'a, F>
where
    F: Fn(&AstNode<'a>) -> bool,
{
    type Item = AstNode<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(node) = self.traversal.next() {
            if (self.predicate)(&node) {
                return Some(node);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traversal_creation() {
        // Test that traversal can be created
        let source = "fn main() {}";
        let mut parser = crate::ast::Parser::new(crate::ast::AstLanguage::Rust).unwrap();
        let tree = parser.parse(source).unwrap();
        let root = parser.root_node(&tree);

        let traversal = Traversal::new(root, TraversalOrder::Pre);
        assert!(traversal.count() > 0);
    }
}
