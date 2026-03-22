//! CSS parser

use super::{LanguageParser, ParsedCode};

pub struct CssParser;

impl LanguageParser for CssParser {
    fn parse(&self, _code: &str) -> Result<ParsedCode, Box<dyn std::error::Error>> {
        Ok(ParsedCode {
            functions: Vec::new(),
            classes: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            variables: Vec::new(),
        })
    }
}

impl CssParser {
    pub fn new() -> Self {
        Self
    }
}