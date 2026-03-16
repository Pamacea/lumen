//! HTML parser using regex (tree-sitter not available for HTML)

use crate::ast::AstLanguage;
use crate::parsers::{LanguageParser, ParsedCode};
use lumenx_core::LumenResult;

/// HTML parser (uses regex-based parsing)
pub struct HtmlParser;

impl HtmlParser {
    /// Create a new HTML parser
    pub fn new() -> LumenResult<Self> {
        Ok(Self)
    }

    /// Extract HTML elements
    pub fn extract_elements(&self, source: &str) -> Vec<HtmlElement> {
        let mut elements = Vec::new();
        let mut stack: Vec<(String, usize)> = Vec::new();

        for (i, line) in source.lines().enumerate() {
            // Find opening tags
            let mut rest = line;
            while let Some(idx) = rest.find('<') {
                let tag_start = idx + 1;
                if let Some(tag_end) = rest[tag_start..].find(|c| c == ' ' || c == '>') {
                    let tag = &rest[tag_start..tag_end];
                    if !tag.starts_with('!') && !tag.starts_with('/') {
                        // Skip closing tags
                        let tag_name = tag.trim().to_string();
                        stack.push((tag_name.clone(), i + 1));

                        elements.push(HtmlElement {
                            tag_name: tag_name.clone(),
                            line: i + 1,
                            is_self_closing: rest[tag_end..].starts_with("/>"),
                            attributes: Vec::new(),
                            children_count: 0,
                        });
                    } else if tag.starts_with('/') {
                        // Closing tag - pop from stack
                        if let Some((_, start_line)) = stack.last() {
                            if let Some(elem) = elements.iter_mut().find(|e| e.line == *start_line) {
                                elem.children_count += 1;
                            }
                        }
                    }
                }
                rest = &rest[idx + 1..];
            }
        }

        elements
    }

    /// Extract attributes from a tag
    pub fn extract_attributes(tag: &str) -> Vec<HtmlAttribute> {
        let mut attrs = Vec::new();
        let mut current = tag;

        while let Some(eq_idx) = current.find('=') {
            let name = current[..eq_idx].trim().to_string();
            let after = &current[eq_idx + 1..];

            if let Some(quote_end) = after.find(|c| c == '"' || c == '\'').and_then(|i| {
                let quote = after.chars().nth(i)?;
                after[i + 1..].find(quote).map(|j| i + 1 + j + 1)
            }) {
                let value = &after[1..quote_end];
                attrs.push(HtmlAttribute {
                    name,
                    value: value.to_string(),
                });
            }

            current = after;
        }

        attrs
    }

    /// Extract meta tags
    pub fn extract_meta_tags(&self, source: &str) -> Vec<MetaTag> {
        let mut metas = Vec::new();

        for line in source.lines() {
            if line.contains("<meta") {
                let attrs = Self::extract_attributes(line);

                metas.push(MetaTag {
                    name: attrs.iter().find(|a| a.name == "name").map(|a| a.value.clone())
                        .or_else(|| attrs.iter().find(|a| a.name == "property").map(|a| a.value.clone())),
                    content: attrs.iter().find(|a| a.name == "content").map(|a| a.value.clone()),
                    charset: attrs.iter().find(|a| a.name == "charset").map(|a| a.value.clone()),
                });
            }
        }

        metas
    }

    /// Extract script tags
    pub fn extract_scripts(&self, source: &str) -> Vec<ScriptTag> {
        let mut scripts = Vec::new();

        for line in source.lines() {
            if line.contains("<script") {
                let attrs = Self::extract_attributes(line);

                scripts.push(ScriptTag {
                    src: attrs.iter().find(|a| a.name == "src").map(|a| a.value.clone()),
                    is_inline: !attrs.iter().any(|a| a.name == "src"),
                    is_async: attrs.iter().any(|a| a.name == "async"),
                    is_defer: attrs.iter().any(|a| a.name == "defer"),
                    script_type: attrs.iter().find(|a| a.name == "type").map(|a| a.value.clone()),
                });
            }
        }

        scripts
    }

    /// Extract link tags (CSS, favicons, etc.)
    pub fn extract_links(&self, source: &str) -> Vec<LinkTag> {
        let mut links = Vec::new();

        for line in source.lines() {
            if line.contains("<link") {
                let attrs = Self::extract_attributes(line);

                links.push(LinkTag {
                    rel: attrs.iter().find(|a| a.name == "rel").map(|a| a.value.clone()),
                    href: attrs.iter().find(|a| a.name == "href").map(|a| a.value.clone()),
                    link_type: attrs.iter().find(|a| a.name == "type").map(|a| a.value.clone()),
                });
            }
        }

        links
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl LanguageParser for HtmlParser {
    fn parse(&mut self, source: &str) -> LumenResult<ParsedCode> {
        // HTML parsing is regex-based, no tree-sitter tree
        let functions = Vec::new();
        let classes = Vec::new();
        let imports = Vec::new();
        let exports = Vec::new();
        let variables = Vec::new();

        let line_count = source.lines().count();
        let blank_lines = source.lines().filter(|l| l.trim().is_empty()).count();
        let comment_lines = source.lines().filter(|l| l.trim().starts_with("<!--")).count();

        Ok(ParsedCode {
            functions,
            classes,
            imports,
            exports,
            variables,
            line_count,
            blank_lines,
            comment_lines,
        })
    }

    fn language(&self) -> crate::ast::AstLanguage {
        AstLanguage::Html
    }
}

/// HTML analyzer
pub struct HtmlAnalyzer;

impl HtmlAnalyzer {
    /// Analyze HTML quality
    pub fn analyze(source: &str) -> LumenResult<HtmlAnalysis> {
        let parser = HtmlParser::new()?;
        let elements = parser.extract_elements(source);
        let metas = parser.extract_meta_tags(source);
        let scripts = parser.extract_scripts(source);
        let links = parser.extract_links(source);

        let has_title = source.contains("<title>");
        let has_charset = metas.iter().any(|m| m.charset.is_some());
        let has_description = metas.iter().any(|m| {
            matches!(m.name.as_deref(), Some("description"))
        });
        let has_viewport = metas.iter().any(|m| {
            matches!(m.name.as_deref(), Some("viewport"))
        });

        Ok(HtmlAnalysis {
            element_count: elements.len(),
            meta_count: metas.len(),
            script_count: scripts.len(),
            link_count: links.len(),
            has_title,
            has_charset,
            has_description,
            has_viewport,
            inline_scripts: scripts.iter().filter(|s| s.is_inline).count(),
        })
    }
}

/// HTML element
#[derive(Debug, Clone)]
pub struct HtmlElement {
    pub tag_name: String,
    pub line: usize,
    pub is_self_closing: bool,
    pub attributes: Vec<HtmlAttribute>,
    pub children_count: usize,
}

/// HTML attribute
#[derive(Debug, Clone)]
pub struct HtmlAttribute {
    pub name: String,
    pub value: String,
}

/// Meta tag
#[derive(Debug, Clone)]
pub struct MetaTag {
    pub name: Option<String>,
    pub content: Option<String>,
    pub charset: Option<String>,
}

/// Script tag
#[derive(Debug, Clone)]
pub struct ScriptTag {
    pub src: Option<String>,
    pub is_inline: bool,
    pub is_async: bool,
    pub is_defer: bool,
    pub script_type: Option<String>,
}

/// Link tag
#[derive(Debug, Clone)]
pub struct LinkTag {
    pub rel: Option<String>,
    pub href: Option<String>,
    pub link_type: Option<String>,
}

/// HTML analysis results
#[derive(Debug, Clone)]
pub struct HtmlAnalysis {
    pub element_count: usize,
    pub meta_count: usize,
    pub script_count: usize,
    pub link_count: usize,
    pub has_title: bool,
    pub has_charset: bool,
    pub has_description: bool,
    pub has_viewport: bool,
    pub inline_scripts: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_elements() {
        let parser = HtmlParser::new().unwrap();
        let source = r#"
<div>
  <p>Hello</p>
  <span>World</span>
</div>
"#;
        let elements = parser.extract_elements(source);
        assert!(elements.iter().any(|e| e.tag_name == "div"));
        assert!(elements.iter().any(|e| e.tag_name == "p"));
    }

    #[test]
    fn test_html_analysis() {
        let analysis = HtmlAnalyzer::analyze("<html><head><title>Test</title></head></html>").unwrap();
        assert!(analysis.has_title);
    }
}
