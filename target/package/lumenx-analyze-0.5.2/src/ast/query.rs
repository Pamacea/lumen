//! Query builder and execution for AST pattern matching

use crate::ast::{AstNode, AstLanguage, Result};
use std::collections::HashMap;
use tree_sitter::{Query as TsQuery, QueryCursor};

/// Builder for creating tree-sitter queries
pub struct QueryBuilder {
    pattern: String,
    captures: Vec<String>,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            captures: Vec::new(),
        }
    }

    /// Add a pattern to match
    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = pattern.to_string();
        self
    }

    /// Add capture names
    pub fn captures(mut self, captures: &[&str]) -> Self {
        self.captures = captures.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Build the query string
    pub fn build(&self) -> String {
        if self.captures.is_empty() {
            self.pattern.clone()
        } else {
            // Append capture names to pattern
            let mut result = self.pattern.clone();
            for capture in &self.captures {
                result.push_str(" @");
                result.push_str(capture);
            }
            result
        }
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Compiled query for a specific language
pub struct Query {
    /// Tree-sitter query
    inner: TsQuery,
    /// Capture indices by name
    capture_indices: HashMap<String, u32>,
    /// Capture names list
    capture_names: Vec<String>,
}

impl Query {
    /// Create a new query from a pattern string
    pub fn new(language: AstLanguage, pattern: &str) -> Result<Self> {
        let ts_lang = language.tree_sitter_language()?;
        let inner = TsQuery::new(&ts_lang, pattern)
            .map_err(|e| lumenx_core::LumenError::ParseError(format!("Invalid query: {}", e)))?;

        // Get capture names from the query
        let capture_names: Vec<String> = inner.capture_names().iter().map(|s| s.to_string()).collect();

        // Build capture index map
        let mut capture_indices = HashMap::new();
        for (i, name) in capture_names.iter().enumerate() {
            capture_indices.insert(name.clone(), i as u32);
        }

        Ok(Self {
            inner,
            capture_indices,
            capture_names,
        })
    }

    /// Create a query for finding function declarations
    pub fn function(language: AstLanguage) -> Result<Self> {
        let pattern = match language {
            AstLanguage::Rust => "(function_item name: (identifier) @name)",
            AstLanguage::TypeScript | AstLanguage::JavaScript | AstLanguage::Tsx | AstLanguage::Jsx => {
                "(function_declaration name: (identifier) @name)"
            }
            AstLanguage::Python => "(function_definition name: (identifier) @name)",
            AstLanguage::Go => "(function_declaration name: (identifier) @name)",
            _ => return Ok(Self::empty(language)),
        };
        Self::new(language, pattern)
    }

    /// Create a query for finding class declarations
    pub fn class(language: AstLanguage) -> Result<Self> {
        let pattern = match language {
            AstLanguage::TypeScript | AstLanguage::JavaScript | AstLanguage::Tsx | AstLanguage::Jsx => {
                "(class_declaration name: (identifier) @name)"
            }
            AstLanguage::Rust => "(struct_item name: (type_identifier) @name)",
            AstLanguage::Python => "(class_definition name: (identifier) @name)",
            _ => return Ok(Self::empty(language)),
        };
        Self::new(language, pattern)
    }

    /// Create a query for finding imports
    pub fn imports(language: AstLanguage) -> Result<Self> {
        let pattern = match language {
            AstLanguage::TypeScript | AstLanguage::JavaScript | AstLanguage::Tsx | AstLanguage::Jsx => {
                "(import_statement source: (string) @source)"
            }
            AstLanguage::Rust => "(use_declaration argument: (scoped_identifier) @path)",
            AstLanguage::Python => "(import_statement name: (dotted_name) @module)",
            _ => return Ok(Self::empty(language)),
        };
        Self::new(language, pattern)
    }

    /// Create a query for finding function calls
    pub fn calls(language: AstLanguage) -> Result<Self> {
        let pattern = match language {
            AstLanguage::Rust => "(call_expression function: (identifier) @name)",
            AstLanguage::TypeScript | AstLanguage::JavaScript | AstLanguage::Tsx | AstLanguage::Jsx => {
                "(call_expression function: (identifier) @name)"
            }
            AstLanguage::Python => "(call function: (identifier) @name)",
            _ => return Ok(Self::empty(language)),
        };
        Self::new(language, pattern)
    }

    /// Create an empty query (returns no matches)
    fn empty(language: AstLanguage) -> Self {
        let ts_lang = language.tree_sitter_language().unwrap();
        Self {
            inner: TsQuery::new(&ts_lang, "").unwrap(),
            capture_indices: HashMap::new(),
            capture_names: Vec::new(),
        }
    }

    /// Get capture index by name
    pub fn capture_index(&self, name: &str) -> Option<u32> {
        self.capture_indices.get(name).copied()
    }

    /// Get the inner tree-sitter query
    pub fn inner(&self) -> &TsQuery {
        &self.inner
    }
}

/// A match from a query execution
#[derive(Debug, Clone)]
pub struct QueryMatch {
    /// Pattern index that matched
    pub pattern_index: u32,
    /// Captured nodes by capture name
    pub captures: HashMap<String, QueryCapture>,
}

/// A captured node from a query match
#[derive(Debug, Clone)]
pub struct QueryCapture {
    /// Capture name
    pub name: String,
    /// Node text
    pub text: String,
    /// Byte range
    pub byte_range: std::ops::Range<usize>,
    /// Start position (line, column)
    pub start: (usize, usize),
    /// End position (line, column)
    pub end: (usize, usize),
}

/// Execute queries on a parsed tree
pub struct QueryExecutor {
    cursor: QueryCursor,
}

impl QueryExecutor {
    /// Create a new executor
    pub fn new() -> Self {
        Self {
            cursor: QueryCursor::new(),
        }
    }

    /// Execute a query on a tree with source code
    pub fn exec(&mut self, query: &Query, tree: &tree_sitter::Tree, source: &str) -> Vec<QueryMatch> {
        let root = tree.root_node();
        let mut matches = Vec::new();

        let query_matches = self.cursor.matches(query.inner(), root, source.as_bytes());
        for match_ in query_matches {
            let mut captures = HashMap::new();

            for capture in match_.captures {
                let capture_index = capture.index as usize;
                let node = capture.node;

                // Get capture name from our stored list
                let name = query.capture_names.get(capture_index)
                    .cloned()
                    .unwrap_or_else(|| format!("capture_{}", capture_index));

                let capture_data = QueryCapture {
                    name: name.clone(),
                    text: source[node.byte_range()].to_string(),
                    byte_range: node.byte_range(),
                    start: (node.start_position().row as usize, node.start_position().column as usize),
                    end: (node.end_position().row as usize, node.end_position().column as usize),
                };

                captures.insert(name, capture_data);
            }

            matches.push(QueryMatch {
                pattern_index: match_.pattern_index as u32,
                captures,
            });
        }

        matches
    }

    /// Execute a query and return only captured text for a specific capture
    pub fn exec_captures(
        &mut self,
        query: &Query,
        tree: &tree_sitter::Tree,
        source: &str,
        capture_name: &str,
    ) -> Vec<String> {
        let matches = self.exec(query, tree, source);
        matches
            .into_iter()
            .filter_map(|m| m.captures.get(capture_name).map(|c| c.text.clone()))
            .collect()
    }
}

impl Default for QueryExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::new()
            .pattern("(function_declaration name: (identifier))")
            .captures(&["name"])
            .build();

        assert!(query.contains("@name"));
    }

    #[test]
    fn test_query_function_rust() {
        let query = Query::function(AstLanguage::Rust);
        assert!(query.is_ok());
    }
}
