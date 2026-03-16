//! AST node wrapper with helper methods

use crate::ast::AstLanguage;
use tree_sitter::Node;

/// Wrapper around tree-sitter Node with convenience methods
#[derive(Debug, Clone, Copy)]
pub struct AstNode<'a> {
    /// Inner tree-sitter node
    inner: Node<'a>,
    /// Language of this node
    language: AstLanguage,
}

impl<'a> AstNode<'a> {
    /// Create a new AST node
    pub fn new(inner: Node<'a>, language: AstLanguage) -> Self {
        Self { inner, language }
    }

    /// Get the inner tree-sitter node
    pub fn inner(&self) -> Node<'a> {
        self.inner
    }

    /// Get the node kind (type)
    pub fn kind(&self) -> &'static str {
        self.inner.kind()
    }

    /// Check if this node is a specific kind
    pub fn is_kind(&self, kind: &str) -> bool {
        self.inner.kind() == kind
    }

    /// Get the node's text from the source
    pub fn text(&self, source: &'a str) -> &'a str {
        &source[self.inner.byte_range()]
    }

    /// Get the start position (line, column)
    pub fn start_position(&self) -> (usize, usize) {
        let pos = self.inner.start_position();
        (pos.row as usize, pos.column as usize)
    }

    /// Get the end position (line, column)
    pub fn end_position(&self) -> (usize, usize) {
        let pos = self.inner.end_position();
        (pos.row as usize, pos.column as usize)
    }

    /// Get children of this node
    pub fn children(&self, source: &'a str) -> Vec<AstNode<'a>> {
        let mut cursor = self.inner.walk();
        (0..self.inner.child_count())
            .filter_map(|i| {
                cursor.goto_first_child();
                for _ in 0..i {
                    if !cursor.goto_next_sibling() {
                        return None;
                    }
                }
                Some(AstNode::new(cursor.node(), self.language))
            })
            .collect()
    }

    /// Get named children only
    pub fn named_children(&self, source: &'a str) -> Vec<AstNode<'a>> {
        let mut cursor = self.inner.walk();
        let mut children = Vec::new();

        if cursor.goto_first_child() {
            loop {
                let node = cursor.node();
                if node.is_named() {
                    children.push(AstNode::new(node, self.language));
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        children
    }

    /// Get child by index
    pub fn child(&self, index: usize) -> Option<AstNode<'a>> {
        self.inner
            .child(index)
            .map(|child| AstNode::new(child, self.language))
    }

    /// Get named child by index
    pub fn named_child(&self, index: usize) -> Option<AstNode<'a>> {
        self.inner
            .named_child(index)
            .map(|child| AstNode::new(child, self.language))
    }

    /// Get child by field name
    pub fn child_by_field_name(&self, field_name: &str) -> Option<AstNode<'a>> {
        self.inner
            .child_by_field_name(field_name)
            .map(|child| AstNode::new(child, self.language))
    }

    /// Get all children for a field
    pub fn children_by_field_name(&self, field_name: &str) -> Vec<AstNode<'a>> {
        let mut cursor = self.inner.walk();
        self.inner
            .children_by_field_name(field_name, &mut cursor)
            .map(|child| AstNode::new(child, self.language))
            .collect()
    }

    /// Get the parent node
    pub fn parent(&self) -> Option<AstNode<'a>> {
        self.inner
            .parent()
            .map(|parent| AstNode::new(parent, self.language))
    }

    /// Get next sibling
    pub fn next_sibling(&self) -> Option<AstNode<'a>> {
        self.inner
            .next_sibling()
            .map(|sibling| AstNode::new(sibling, self.language))
    }

    /// Get previous sibling
    pub fn prev_sibling(&self) -> Option<AstNode<'a>> {
        self.inner
            .prev_sibling()
            .map(|sibling| AstNode::new(sibling, self.language))
    }

    /// Check if node is named
    pub fn is_named(&self) -> bool {
        self.inner.is_named()
    }

    /// Check if node is missing
    pub fn is_missing(&self) -> bool {
        self.inner.is_missing()
    }

    /// Check if node is extra (like parentheses)
    pub fn is_extra(&self) -> bool {
        self.inner.is_extra()
    }

    /// Get the byte range
    pub fn byte_range(&self) -> std::ops::Range<usize> {
        self.inner.byte_range()
    }

    /// Get node language
    pub fn language(&self) -> AstLanguage {
        self.language
    }
}

/// Node kind classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// Function declaration
    Function,
    /// Function call
    Call,
    /// Variable declaration
    Variable,
    /// Class declaration
    Class,
    /// Interface declaration
    Interface,
    /// Type declaration
    Type,
    /// Import statement
    Import,
    /// Export statement
    Export,
    /// Return statement
    Return,
    /// If statement
    If,
    /// Loop statement
    Loop,
    /// Try statement
    Try,
    /// Throw statement
    Throw,
    /// Identifier
    Identifier,
    /// String literal
    StringLiteral,
    /// Number literal
    NumberLiteral,
    /// Boolean literal
    BooleanLiteral,
    /// Comment
    Comment,
    /// Other
    Other,
}

impl NodeKind {
    /// Detect node kind from tree-sitter node type
    pub fn from_str(kind: &str) -> Self {
        match kind {
            "function_declaration" | "function_definition" | "function_item" => NodeKind::Function,
            "call_expression" | "call" => NodeKind::Call,
            "variable_declaration" | "let_declaration" | "const_declaration" => NodeKind::Variable,
            "class_declaration" | "class" => NodeKind::Class,
            "interface_declaration" | "interface" => NodeKind::Interface,
            "type_alias_declaration" | "type_alias" => NodeKind::Type,
            "import_statement" | "import" => NodeKind::Import,
            "export_statement" | "export" => NodeKind::Export,
            "return_statement" | "return" => NodeKind::Return,
            "if_statement" | "if" => NodeKind::If,
            "for_statement" | "while_statement" | "loop_statement" => NodeKind::Loop,
            "try_statement" | "try" => NodeKind::Try,
            "throw_statement" | "throw" => NodeKind::Throw,
            "identifier" => NodeKind::Identifier,
            "string" | "string_literal" | "template_string" => NodeKind::StringLiteral,
            "number" | "integer" | "float" => NodeKind::NumberLiteral,
            "true" | "false" => NodeKind::BooleanLiteral,
            "comment" => NodeKind::Comment,
            _ => NodeKind::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_kind_detection() {
        assert_eq!(NodeKind::from_str("function_declaration"), NodeKind::Function);
        assert_eq!(NodeKind::from_str("call_expression"), NodeKind::Call);
        assert_eq!(NodeKind::from_str("identifier"), NodeKind::Identifier);
        assert_eq!(NodeKind::from_str("unknown_type"), NodeKind::Other);
    }
}
