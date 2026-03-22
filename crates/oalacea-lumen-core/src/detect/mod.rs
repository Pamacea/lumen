//! # Detection Module
//!
//! Framework and project detection for Oalacea Lumen.

pub mod detectors;

// Re-export from core module
pub use crate::core::{Framework, Language, PackageJson, ProjectInfo, TestRunner};

// Import Result type alias
use crate::core::LumenResult;
use std::path::{Path, PathBuf};

/// Detection result
#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub framework: Framework,
    pub language: Language,
    pub test_runner: TestRunner,
    pub package_manager: Option<String>,
}

/// Framework detector
pub struct FrameworkDetector {
    root: PathBuf,
}

impl FrameworkDetector {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self { root: root.as_ref().to_path_buf() }
    }

    pub fn detect(&self) -> LumenResult<ProjectInfo> {
        let framework = self.detect_framework()?;
        let language = self.detect_language(&framework)?;
        let test_runner = self.detect_test_runner(&framework)?;
        let package_manager = self.detect_package_manager(&framework);

        let (dependencies, dev_dependencies, package_json, cargo_deps) =
            self.load_dependencies(&framework)?;

        Ok(ProjectInfo {
            name: self.root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            root: self.root.clone(),
            framework,
            language,
            test_runner,
            package_manager,
            dependencies,
            dev_dependencies,
            database: None,
            package_json,
            cargo_dependencies: cargo_deps,
        })
    }

    fn detect_framework(&self) -> LumenResult<Framework> {
        use detectors::*;

        if check_nextjs(&self.root) { return Ok(Framework::NextJs); }
        if check_nestjs(&self.root) { return Ok(Framework::NestJS); }
        if check_axum(&self.root) { return Ok(Framework::Axum); }
        if check_actix(&self.root) { return Ok(Framework::ActixWeb); }
        if check_rocket(&self.root) { return Ok(Framework::Rocket); }

        Ok(Framework::Unknown)
    }

    fn detect_language(&self, framework: &Framework) -> LumenResult<Language> {
        Ok(if framework.is_rust() {
            Language::Rust
        } else if self.root.join("tsconfig.json").exists() {
            Language::TypeScript
        } else {
            Language::JavaScript
        })
    }

    fn detect_test_runner(&self, framework: &Framework) -> LumenResult<TestRunner> {
        Ok(if framework.is_rust() {
            TestRunner::CargoTest
        } else if self.root.join("vitest.config.ts").exists() {
            TestRunner::Vitest
        } else {
            TestRunner::Jest
        })
    }

    fn detect_package_manager(&self, framework: &Framework) -> Option<String> {
        if framework.is_rust() {
            Some("cargo".to_string())
        } else if self.root.join("pnpm-lock.yaml").exists() {
            Some("pnpm".to_string())
        } else if self.root.join("yarn.lock").exists() {
            Some("yarn".to_string())
        } else if self.root.join("package-lock.json").exists() {
            Some("npm".to_string())
        } else {
            None
        }
    }

    fn load_dependencies(&self, _framework: &Framework) -> LumenResult<(Vec<String>, Vec<String>, Option<PackageJson>, Option<std::collections::HashMap<String, String>>)> {
        Ok((vec![], vec![], None, None))
    }
}
