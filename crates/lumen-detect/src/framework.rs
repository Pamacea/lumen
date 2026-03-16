//! Framework detection

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use lumen_core::{LumenResult, Project};

/// Category of framework
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FrameworkCategory {
    /// Frontend framework
    Frontend,
    /// Backend framework
    Backend,
    /// Full-stack framework
    FullStack,
    /// Mobile framework
    Mobile,
    /// Desktop framework
    Desktop,
    /// Testing framework
    Testing,
    /// Build tool
    BuildTool,
    /// Database
    Database,
    /// ORM
    ORM,
    /// Utility library
    Utility,
}

/// Detected framework
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Framework {
    /// Name of the framework
    pub name: String,

    /// Version (if detected)
    pub version: Option<String>,

    /// Category
    pub category: FrameworkCategory,

    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,
}

impl Framework {
    /// Create a new framework detection
    pub fn new(
        name: impl Into<String>,
        category: FrameworkCategory,
        confidence: f32,
    ) -> Self {
        Self {
            name: name.into(),
            version: None,
            category,
            confidence,
        }
    }

    /// With version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// Detect frameworks used in the project
pub fn detect_frameworks(project: &Project) -> lumen_core::LumenResult<Vec<Framework>> {
    let mut frameworks = Vec::new();

    // Check package.json for Node.js frameworks
    if let Ok(package_json) = project.read_file("package.json") {
        frameworks.extend(detect_from_package_json(&package_json));
    }

    // Check Cargo.toml for Rust frameworks
    if let Ok(cargo_toml) = project.read_file("Cargo.toml") {
        frameworks.extend(detect_from_cargo_toml(&cargo_toml));
    }

    // Check go.mod for Go frameworks
    if let Ok(go_mod) = project.read_file("go.mod") {
        frameworks.extend(detect_from_go_mod(&go_mod));
    }

    // Check pom.xml for Java frameworks
    if project.file_exists("pom.xml") {
        frameworks.push(Framework::new("Maven", FrameworkCategory::BuildTool, 1.0));
    }

    // Check requirements.txt or pyproject.toml for Python frameworks
    if project.file_exists("requirements.txt") || project.file_exists("pyproject.toml") {
        frameworks.extend(detect_python_frameworks(project));
    }

    Ok(frameworks)
}

fn detect_from_package_json(content: &str) -> Vec<Framework> {
    let mut frameworks = Vec::new();

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
        let dependencies = value
            .get("dependencies")
            .and_then(|d| d.as_object())
            .into_iter()
            .flatten()
            .map(|(k, _)| k.as_str());

        let dev_dependencies = value
            .get("devDependencies")
            .and_then(|d| d.as_object())
            .into_iter()
            .flatten()
            .map(|(k, _)| k.as_str());

        let all_deps: HashSet<_> = dependencies.chain(dev_dependencies).flatten().collect();

        // Detect React
        if all_deps.contains("react") {
            frameworks.push(Framework::new("React", FrameworkCategory::Frontend, 1.0));
        }

        // Detect Vue
        if all_deps.contains("vue") {
            frameworks.push(Framework::new("Vue", FrameworkCategory::Frontend, 1.0));
        }

        // Detect Next.js
        if all_deps.contains("next") {
            frameworks.push(Framework::new("Next.js", FrameworkCategory::FullStack, 1.0));
        }

        // Detect NestJS
        if all_deps.contains("@nestjs/core") {
            frameworks.push(Framework::new("NestJS", FrameworkCategory::Backend, 1.0));
        }

        // Detect Vite
        if all_deps.contains("vite") {
            frameworks.push(Framework::new("Vite", FrameworkCategory::BuildTool, 1.0));
        }

        // Detect Webpack
        if all_deps.contains("webpack") {
            frameworks.push(Framework::new("Webpack", FrameworkCategory::BuildTool, 1.0));
        }

        // Detect TypeScript
        if all_deps.contains("typescript") {
            frameworks.push(Framework::new("TypeScript", FrameworkCategory::Utility, 1.0));
        }

        // Detect Jest
        if all_deps.contains("jest") {
            frameworks.push(Framework::new("Jest", FrameworkCategory::Testing, 1.0));
        }

        // Detect Vitest
        if all_deps.contains("vitest") {
            frameworks.push(Framework::new("Vitest", FrameworkCategory::Testing, 1.0));
        }

        // Detect Playwright
        if all_deps.contains("@playwright/test") {
            frameworks.push(Framework::new("Playwright", FrameworkCategory::Testing, 1.0));
        }
    }

    frameworks
}

fn detect_from_cargo_toml(content: &str) -> Vec<Framework> {
    let mut frameworks = Vec::new();

    // Parse Cargo.toml (simplified parsing)
    if content.contains("axum") {
        frameworks.push(Framework::new("Axum", FrameworkCategory::Backend, 1.0));
    }

    if content.contains("actix-web") {
        frameworks.push(Framework::new("Actix Web", FrameworkCategory::Backend, 1.0));
    }

    if content.contains("tokio") {
        frameworks.push(Framework::new("Tokio", FrameworkCategory::Utility, 1.0));
    }

    frameworks
}

fn detect_from_go_mod(content: &str) -> Vec<Framework> {
    let mut frameworks = Vec::new();

    if content.contains("gin-gonic") {
        frameworks.push(Framework::new("Gin", FrameworkCategory::Backend, 1.0));
    }

    if content.contains("gorilla/mux") {
        frameworks.push(Framework::new("Gorilla Mux", FrameworkCategory::Backend, 1.0));
    }

    frameworks
}

fn detect_python_frameworks(project: &Project) -> Vec<Framework> {
    let mut frameworks = Vec::new();

    // Check for common Python frameworks
    if project.file_exists("manage.py") {
        frameworks.push(Framework::new("Django", FrameworkCategory::Backend, 0.9));
    }

    if project.file_exists("app.py") || project.file_exists("wsgi.py") {
        frameworks.push(Framework::new("Flask", FrameworkCategory::Backend, 0.7));
    }

    if project.read_file("pyproject.toml")
        .map(|c| c.contains("fastapi"))
        .unwrap_or(false)
    {
        frameworks.push(Framework::new("FastAPI", FrameworkCategory::Backend, 1.0));
    }

    frameworks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_creation() {
        let fw = Framework::new("React", FrameworkCategory::Frontend, 1.0);
        assert_eq!(fw.name, "React");
        assert_eq!(fw.category, FrameworkCategory::Frontend);
        assert_eq!(fw.confidence, 1.0);
        assert!(fw.version.is_none());
    }

    #[test]
    fn test_framework_with_version() {
        let fw = Framework::new("React", FrameworkCategory::Frontend, 1.0)
            .with_version("18.2.0");
        assert_eq!(fw.version, Some("18.2.0".to_string()));
    }
}
