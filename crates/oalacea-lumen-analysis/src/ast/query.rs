//! Query execution for AST pattern matching

use super::{Query, Match};

pub struct QueryExecutor;

impl QueryExecutor {
    pub fn new() -> Self {
        Self
    }

    pub fn execute<'a>(&self, _code: &str, _query: &Query<'a>) -> Vec<Match<'a>> {
        Vec::new()
    }
}