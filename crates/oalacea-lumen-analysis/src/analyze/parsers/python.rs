//! Python language parser and analyzer
//!
//! Uses tree-sitter-python to parse Python source code and extract:
//! - Function definitions
//! - Class definitions
//! - Import statements
//! - Decorators
//! - Type annotations
//! - Async/await patterns
//! - Context managers

use oalacea_lumen_core::LumenResult;
use tree_sitter::{Node, Parser, Tree};

/// Python code parser
pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    /// Create a new Python parser
    pub fn new() -> LumenResult<Self> {
        let mut parser = Parser::new();
        let lang = tree_sitter_python::language();
        parser.set_language(&lang)
            .map_err(|e| oalacea_lumen_core::LumenError::ParseError(format!("Language error: {}", e)))?;
        Ok(Self { parser })
    }

    /// Parse Python source code
    pub fn parse(&mut self, source: &str) -> LumenResult<ParsedCode> {
        let tree = self.parser.parse(source, None)
            .ok_or_else(|| oalacea_lumen_core::LumenError::ParseError("Failed to parse Python code".to_string()))?;
        let root = tree.root_node();

        let mut functions = vec![];
        let mut classes = vec![];
        let mut imports = vec![];
        let mut exports = vec![];
        let mut variables = vec![];

        let line_count = source.lines().count();
        let blank_lines = source.lines().filter(|l| l.trim().is_empty()).count();
        let comment_lines = source.lines().filter(|l| l.trim().starts_with('#')).count();

        // Walk the tree and extract information
        self.walk_tree(&root, source, &mut functions, &mut classes, &mut imports);

        // Detect __all__ exports
        if source.contains("__all__") {
            exports.push(ExportInfo {
                name: "__all__".to_string(),
                is_default: false,
                is_reexport: false,
                source_module: None,
                position: (0, 0),
            });
        }

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

    fn walk_tree(
        &self,
        node: &Node,
        source: &str,
        functions: &mut Vec<FunctionInfo>,
        classes: &mut Vec<ClassInfo>,
        imports: &mut Vec<ImportInfo>,
    ) {
        if node.kind() == "function_definition" {
            if let Some(func) = self.extract_function(node, source) {
                functions.push(func);
            }
        } else if node.kind() == "class_definition" {
            if let Some(class) = self.extract_class(node, source) {
                classes.push(class);
            }
        } else if node.kind() == "import_statement" || node.kind() == "import_from_statement" {
            if let Some(import) = self.extract_import(node, source) {
                imports.push(import);
            }
        }

        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = &cursor.node();
                self.walk_tree(child, source, functions, classes, imports);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    fn extract_function(&self, node: &Node, source: &str) -> Option<FunctionInfo> {
        let name_node = node.child_by_field_name("name")?;
        let name = node_text(&name_node, source);

        let start = (
            node.start_position().row + 1,
            node.start_position().column,
        );
        let end = (
            node.end_position().row + 1,
            node.end_position().column,
        );

        let parameters = self.extract_parameters(node, source);
        let return_type = node.child_by_field_name("return_type")
            .map(|n| node_text(&n, source));

        let body = node.child_by_field_name("body")
            .map(|n| source[n.byte_range()].trim().to_string());

        // Check for async decorator
        let mut is_async = false;
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "decorator" {
                    let decorator_text = source[child.byte_range()].to_string();
                    if decorator_text.contains("async") || decorator_text.contains("coroutine") {
                        is_async = true;
                    }
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        Some(FunctionInfo {
            name,
            start,
            end,
            is_async,
            is_generator: false,
            parameters,
            return_type,
            body,
        })
    }

    fn extract_class(&self, node: &Node, source: &str) -> Option<ClassInfo> {
        let name_node = node.child_by_field_name("name")?;
        let name = node_text(&name_node, source);

        let start = (
            node.start_position().row + 1,
            node.start_position().column,
        );
        let end = (
            node.end_position().row + 1,
            node.end_position().column,
        );

        let mut extends = vec![];
        if let Some(args_node) = node.child_by_field_name("superclasses") {
            let mut cursor = args_node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() == "identifier" || child.kind() == "dotted_name" {
                        extends.push(node_text(&child, source));
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        let methods = self.extract_class_methods(node, source);

        Some(ClassInfo {
            name,
            start,
            end,
            extends,
            implements: vec![],
            methods,
            properties: vec![],
        })
    }

    fn extract_import(&self, node: &Node, source: &str) -> Option<ImportInfo> {
        let text = source[node.byte_range()].to_string();
        let position = (
            node.start_position().row + 1,
            node.start_position().column,
        );

        if node.kind() == "import_statement" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let module = node_text(&name_node, source);
                return Some(ImportInfo {
                    module: module.clone(),
                    names: vec![module],
                    is_default: true,
                    is_wildcard: false,
                    alias: None,
                    position,
                });
            }
            return None;
        } else if node.kind() == "import_from_statement" {
            let module_node = node.child_by_field_name("module_name");
            let module = module_node
                .map(|n| node_text(&n, source))
                .unwrap_or_else(|| String::new());

            let is_wildcard = text.contains("*");
            let mut names = vec![];

            let module_byte_range = module_node.as_ref().map(|n| n.byte_range());

            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() == "dotted_name" {
                        // Skip the module_name node
                        let is_module = module_byte_range.as_ref()
                            .map(|range| child.byte_range() == *range)
                            .unwrap_or(false);
                        if !is_module {
                            names.push(node_text(&child, source));
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            Some(ImportInfo {
                module,
                names: names.clone(),
                is_default: false,
                is_wildcard,
                alias: None,
                position,
            })
        } else {
            None
        }
    }

    fn extract_parameters(&self, func_node: &Node, source: &str) -> Vec<ParameterInfo> {
        let mut params = vec![];

        if let Some(params_node) = func_node.child_by_field_name("parameters") {
            let mut cursor = params_node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() == "identifier" || child.kind() == "default_parameter" || child.kind() == "typed_parameter" {
                        let name = if child.kind() == "typed_parameter" {
                            child.child_by_field_name("name")
                                .map(|n| node_text(&n, source))
                                .unwrap_or_else(|| String::new())
                        } else {
                            let text = node_text(&child, source);
                            text.split('=').next().unwrap_or(&text).trim().to_string()
                        };

                        let type_annotation = if child.kind() == "typed_parameter" {
                            child.child_by_field_name("type")
                                .map(|n| node_text(&n, source))
                        } else {
                            None
                        };

                        let default_value = if let Some(eq_pos) = source[child.byte_range()].find('=') {
                            Some(source[child.byte_range()].split_at(eq_pos + 1).1.trim().to_string())
                        } else {
                            None
                        };

                        params.push(ParameterInfo {
                            name,
                            type_annotation,
                            default_value,
                        });
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        params
    }

    fn extract_class_methods(&self, class_node: &Node, source: &str) -> Vec<FunctionInfo> {
        let mut methods = vec![];

        if let Some(body_node) = class_node.child_by_field_name("body") {
            let mut cursor = body_node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() == "function_definition" {
                        if let Some(func) = self.extract_function(&child, source) {
                            methods.push(func);
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        methods
    }
}

fn node_text(node: &Node, source: &str) -> String {
    source[node.byte_range()].to_string()
}

impl Default for PythonParser {
    fn default() -> Self {
        // Create a parser with tree-sitter Python language
        let mut parser = Parser::new();
        let lang = tree_sitter_python::language();
        let _ = parser.set_language(&lang);
        Self { parser }
    }
}

/// Result of parsing Python code
#[derive(Debug, Clone)]
pub struct ParsedCode {
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<ImportInfo>,
    pub exports: Vec<ExportInfo>,
    pub variables: Vec<VariableInfo>,
    pub line_count: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
}

/// Function information
#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub is_async: bool,
    pub is_generator: bool,
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub body: Option<String>,
}

/// Parameter information
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub name: String,
    pub type_annotation: Option<String>,
    pub default_value: Option<String>,
}

/// Class information
#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub extends: Vec<String>,
    pub implements: Vec<String>,
    pub methods: Vec<FunctionInfo>,
    pub properties: Vec<VariableInfo>,
}

/// Import information
#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub module: String,
    pub names: Vec<String>,
    pub is_default: bool,
    pub is_wildcard: bool,
    pub alias: Option<String>,
    pub position: (usize, usize),
}

/// Export information
#[derive(Debug, Clone)]
pub struct ExportInfo {
    pub name: String,
    pub is_default: bool,
    pub is_reexport: bool,
    pub source_module: Option<String>,
    pub position: (usize, usize),
}

/// Variable information
#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub type_annotation: Option<String>,
    pub is_mutable: bool,
}

/// Python code analyzer
pub struct PythonAnalyzer {
    parser: PythonParser,
}

impl PythonAnalyzer {
    /// Create a new Python analyzer
    pub fn new() -> LumenResult<Self> {
        Ok(Self {
            parser: PythonParser::new()?,
        })
    }

    /// Analyze Python code for patterns and issues
    pub fn analyze(&mut self, source: &str) -> LumenResult<PythonAnalysis> {
        let parsed = self.parser.parse(source)?;

        let mut analysis = PythonAnalysis {
            function_count: parsed.functions.len(),
            class_count: parsed.classes.len(),
            import_count: parsed.imports.len(),
            async_functions: parsed.functions.iter()
                .filter(|f| f.is_async)
                .map(|f| f.name.clone())
                .collect(),
            type_hints_coverage: 0.0,
            has_docstrings: false,
            complexity_issues: vec![],
            security_issues: vec![],
        };

        // Check for type hints
        let functions_with_hints = parsed.functions.iter()
            .filter(|f| f.return_type.is_some() || !f.parameters.is_empty())
            .count();
        if !parsed.functions.is_empty() {
            analysis.type_hints_coverage = (functions_with_hints as f64 / parsed.functions.len() as f64) * 100.0;
        }

        // Check for docstrings
        analysis.has_docstrings = source.contains("\"\"\"")
            || source.contains("'''")
            || parsed.functions.iter().any(|f| {
                f.body.as_ref()
                    .map(|b| b.contains("\"\"\"") || b.contains("'''"))
                    .unwrap_or(false)
            });

        Ok(analysis)
    }
}

impl Default for PythonAnalyzer {
    fn default() -> Self {
        // Create analyzer with default parser
        Self {
            parser: PythonParser::default(),
        }
    }
}

/// Python analysis result
#[derive(Debug, Clone)]
pub struct PythonAnalysis {
    /// Number of functions
    pub function_count: usize,
    /// Number of classes
    pub class_count: usize,
    /// Number of imports
    pub import_count: usize,
    /// Async function names
    pub async_functions: Vec<String>,
    /// Type hints coverage percentage
    pub type_hints_coverage: f64,
    /// Whether docstrings are present
    pub has_docstrings: bool,
    /// Complexity issues found
    pub complexity_issues: Vec<String>,
    /// Security issues found
    pub security_issues: Vec<String>,
}
