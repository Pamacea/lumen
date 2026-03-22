//! CSS parser using tree-sitter

use crate::analyze::ast::{AstLanguage, Parser};
use crate::analyze::parsers::{LanguageParser, ParsedCode};
use oalacea_lumen_core::LumenResult;

/// CSS parser
pub struct CssParser {
    parser: Parser,
}

impl CssParser {
    /// Create a new CSS parser
    pub fn new() -> LumenResult<Self> {
        Ok(Self {
            parser: Parser::new(AstLanguage::Css)?,
        })
    }

    /// Extract CSS rules
    pub fn extract_rules(&self, source: &str) -> Vec<CssRule> {
        let mut rules = Vec::new();
        let mut current_selector = String::new();
        let mut properties = Vec::new();

        for (i, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with("/*") || trimmed.starts_with("//") {
                continue;
            }

            // Check for selector (ends with {)
            if trimmed.ends_with('{') {
                current_selector = trimmed.trim_end_matches('{').trim().to_string();
            }
            // Check for closing brace
            else if trimmed == "}" {
                if !current_selector.is_empty() {
                    rules.push(CssRule {
                        selector: current_selector.clone(),
                        properties: properties.clone(),
                        line: i,
                    });
                    current_selector.clear();
                    properties.clear();
                }
            }
            // Extract properties
            else if !current_selector.is_empty() && trimmed.contains(':') {
                if let Some((name, value)) = trimmed.split_once(':') {
                    let value = value.trim_end_matches(';').trim();
                    properties.push(CssProperty {
                        name: name.trim().to_string(),
                        value: value.to_string(),
                        important: value.ends_with("!important"),
                    });
                }
            }
            // Handle single-line CSS like ".foo { color: #fff; }"
            else if trimmed.contains('{') && trimmed.contains('}') && trimmed.contains(':') {
                // Parse single-line rule: selector { property: value; }
                if let Some(selector_part) = trimmed.split('{').next() {
                    let selector = selector_part.trim();
                    let rest = trimmed.split('{').nth(1).unwrap_or("");
                    let content = rest.trim_end_matches('}').trim();

                    let mut rule_properties = Vec::new();
                    for prop in content.split(';') {
                        let prop = prop.trim();
                        if prop.contains(':') {
                            if let Some((name, value)) = prop.split_once(':') {
                                let value = value.trim();
                                rule_properties.push(CssProperty {
                                    name: name.trim().to_string(),
                                    value: value.to_string(),
                                    important: value.ends_with("!important"),
                                });
                            }
                        }
                    }

                    rules.push(CssRule {
                        selector: selector.to_string(),
                        properties: rule_properties,
                        line: i,
                    });
                }
            }
        }

        rules
    }

    /// Extract class names
    pub fn extract_classes(&self, source: &str) -> Vec<String> {
        let mut classes = Vec::new();

        for line in source.lines() {
            // Match .class-name pattern
            let mut current = line;
            while let Some(idx) = current.find('.') {
                let rest = &current[idx + 1..];
                // Find the end of the class name
                let class_name: String = rest.chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect();

                if !class_name.is_empty() && !class_name.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
                    classes.push(class_name);
                }

                current = rest;
            }
        }

        classes.sort();
        classes.dedup();
        classes
    }

    /// Extract media queries
    pub fn extract_media_queries(&self, source: &str) -> Vec<MediaQuery> {
        let mut queries = Vec::new();

        for line in source.lines() {
            if line.trim().starts_with("@media") {
                queries.push(MediaQuery {
                    query: line.trim().to_string(),
                    is_print: line.contains("print"),
                    is_screen: line.contains("screen"),
                    min_width: extract_width(line, "min-width"),
                    max_width: extract_width(line, "max-width"),
                });
            }
        }

        queries
    }
}

impl Default for CssParser {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl LanguageParser for CssParser {
    fn parse(&mut self, source: &str) -> LumenResult<ParsedCode> {
        let tree = self.parser.parse(source)?;

        // CSS doesn't have functions/classes in the traditional sense
        let functions = Vec::new();
        let classes = Vec::new();
        let imports = Vec::new();
        let exports = Vec::new();
        let variables = Vec::new();

        let line_count = source.lines().count();
        let blank_lines = source.lines().filter(|l| l.trim().is_empty()).count();
        let comment_lines = source.lines().filter(|l| l.trim().starts_with("/*")).count();

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

    fn language(&self) -> crate::analyze::ast::AstLanguage {
        AstLanguage::Css
    }
}

/// CSS analyzer
pub struct CssAnalyzer;

impl CssAnalyzer {
    /// Analyze CSS quality
    pub fn analyze(source: &str) -> LumenResult<CssAnalysis> {
        let parser = CssParser::new()?;
        let rules = parser.extract_rules(source);

        let total_selectors = rules.len();
        let complex_selectors = rules.iter()
            .filter(|r| r.selector.split(' ').count() > 3)
            .count();

        let uses_important = rules.iter()
            .flat_map(|r| r.properties.iter())
            .filter(|p| p.important)
            .count();

        Ok(CssAnalysis {
            rule_count: total_selectors,
            complex_selector_ratio: if total_selectors > 0 {
                complex_selectors as f64 / total_selectors as f64
            } else {
                0.0
            },
            important_count: uses_important,
            has_color_hardcode: source.contains("#fff") || source.contains("#000"),
        })
    }
}

/// CSS rule
#[derive(Debug, Clone)]
pub struct CssRule {
    pub selector: String,
    pub properties: Vec<CssProperty>,
    pub line: usize,
}

/// CSS property
#[derive(Debug, Clone)]
pub struct CssProperty {
    pub name: String,
    pub value: String,
    pub important: bool,
}

/// Media query
#[derive(Debug, Clone)]
pub struct MediaQuery {
    pub query: String,
    pub is_print: bool,
    pub is_screen: bool,
    pub min_width: Option<u32>,
    pub max_width: Option<u32>,
}

/// CSS analysis results
#[derive(Debug, Clone)]
pub struct CssAnalysis {
    pub rule_count: usize,
    pub complex_selector_ratio: f64,
    pub important_count: usize,
    pub has_color_hardcode: bool,
}

fn extract_width(line: &str, prefix: &str) -> Option<u32> {
    line.find(prefix)
        .and_then(|i| {
            let rest = &line[i..];
            rest.find(':')
                .and_then(|j| {
                    let value = &rest[j + 1..];
                    value.trim()
                        .trim_end_matches("px")
                        .trim_end_matches("em")
                        .trim()
                        .parse::<u32>()
                        .ok()
                })
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_classes() {
        let parser = CssParser::new().unwrap();
        let source = r#"
.button { }
.nav-item { }
.btn-primary { }
"#;
        let classes = parser.extract_classes(source);
        assert!(classes.contains(&"button".to_string()));
        assert!(classes.contains(&"nav-item".to_string()));
    }

    #[test]
    fn test_css_analysis() {
        let analysis = CssAnalyzer::analyze(".foo { color: #fff; }").unwrap();
        assert!(analysis.has_color_hardcode);
        assert_eq!(analysis.rule_count, 1);
    }
}
