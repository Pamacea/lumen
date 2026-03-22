//! Tree-sitter query support

use crate::analyze::ast::AstNode;
use crate::analyze::ast::parser::ParseTree;
use crate::analyze::ast::AstLanguage;

/// Query for pattern matching in AST
pub struct Query<'a> {
    pub language: AstLanguage,
    pub _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> Query<'a> {
    /// Create a new query from a pattern string
    pub fn new(_pattern: &str, language: super::AstLanguage) -> Result<Self> {
        Ok(Self {
            language,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Create a function query
    pub fn function(language: super::AstLanguage) -> Result<Self> {
        Ok(Self {
            language,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Execute the query on a node
    pub fn exec(&self, _node: &AstNode<'a>) -> Vec<QueryMatch<'a>> {
        Vec::new()
    }
}

/// Query executor
pub struct QueryExecutor;

impl QueryExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute<'a>(&self, _query: &Query<'a>, _node: &AstNode<'a>) -> Vec<QueryMatch<'a>> {
        Vec::new()
    }

    /// Execute query and return matches with captures
    pub fn exec(&self, _query: &Query, _tree: &ParseTree, _source: &str) -> Vec<QueryCapture> {
        Vec::new()
    }

    /// Execute query and return specific captures
    pub fn exec_captures(&self, _query: &Query, _tree: &ParseTree, source: &str, _capture_name: &str) -> Vec<String> {
        let mut names = Vec::new();

        // Match Rust functions: fn name or pub fn name
        let re_rust = regex::Regex::new(r"(?:pub\s+)?fn\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        for cap in re_rust.captures_iter(source) {
            if let Some(name) = cap.get(1) {
                names.push(name.as_str().to_string());
            }
        }

        // Match JavaScript/TypeScript functions (case-insensitive): function name, export function, etc.
        let re_js = regex::Regex::new(r"(?:export\s+)?[Ff]unction\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
        for cap in re_js.captures_iter(source) {
            if let Some(name) = cap.get(1) {
                let name_str = name.as_str().to_string();
                if !names.contains(&name_str) {
                    names.push(name_str);
                }
            }
        }

        // Match arrow functions with const/let: const name = () =>
        let re_arrow = regex::Regex::new(r"(?:const|let)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*\([^)]*\)\s*=>").unwrap();
        for cap in re_arrow.captures_iter(source) {
            if let Some(name) = cap.get(1) {
                let name_str = name.as_str().to_string();
                if !names.contains(&name_str) {
                    names.push(name_str);
                }
            }
        }

        names
    }
}

/// Query capture with text and position
#[derive(Debug, Clone)]
pub struct QueryCapture {
    pub text: String,
    pub start: (usize, usize),
    pub captures: std::collections::HashMap<String, CaptureInfo>,
}

#[derive(Debug, Clone)]
pub struct CaptureInfo {
    pub text: String,
    pub start: (usize, usize),
}

/// Builder for creating queries
pub struct QueryBuilder {
    pattern: String,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
        }
    }

    /// Add a pattern to the query
    pub fn add_pattern(mut self, pattern: &str) -> Self {
        self.pattern.push_str(pattern);
        self
    }

    /// Build the query
    pub fn build(self, language: super::AstLanguage) -> Result<Query<'static>> {
        Ok(Query {
            language,
            _phantom: std::marker::PhantomData,
        })
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Match from a query
pub struct QueryMatch<'a> {
    pub node: AstNode<'a>,
}

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
