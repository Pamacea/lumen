//! HTML parser

pub struct HtmlParser;

impl HtmlParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self, _html: &str) -> HtmlDocument {
        HtmlDocument
    }
}

pub struct HtmlDocument;
pub struct HtmlNode;
pub struct MetaTag;
pub struct ScriptTag;
pub struct LinkTag;