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

        // Check Rust frameworks first (higher priority)
        if check_axum(&self.root) { return Ok(Framework::Axum); }
        if check_actix(&self.root) { return Ok(Framework::ActixWeb); }
        if check_rocket(&self.root) { return Ok(Framework::Rocket); }
        if check_poem(&self.root) { return Ok(Framework::Poem); }

        // Check if it's a basic Rust project (has Cargo.toml with src/ directory)
        if self.root.join("Cargo.toml").exists() {
            let src_dir = self.root.join("src");
            let has_rs_files = src_dir.exists() && (
                src_dir.join("main.rs").exists() ||
                src_dir.join("lib.rs").exists()
            );
            if has_rs_files {
                return Ok(Framework::Unknown); // Rust project but no specific framework
            }
        }

        // Check JS/TS frameworks - priority order (most specific first)
        if check_nextjs(&self.root) { return Ok(Framework::NextJs); }
        if check_nestjs(&self.root) { return Ok(Framework::NestJS); }
        if check_remix(&self.root) { return Ok(Framework::Remix); }
        if check_sveltekit(&self.root) { return Ok(Framework::SvelteKit); }
        if check_nuxt(&self.root) { return Ok(Framework::Nuxt); }
        if check_astro(&self.root) { return Ok(Framework::Astro); }
        if check_vite_react(&self.root) { return Ok(Framework::ViteReact); }
        if check_vite_vue(&self.root) { return Ok(Framework::ViteVue); }
        if check_vite_svelte(&self.root) { return Ok(Framework::ViteSvelte); }

        // Check Angular (angular.json or ng folder)
        if self.root.join("angular.json").exists() || self.root.join("ng").exists() {
            return Ok(Framework::Angular);
        }

        // Check SolidJS
        if package_has_dep(&self.root, "solid-js") {
            return Ok(Framework::Solid);
        }

        // Check Express
        if check_express(&self.root) { return Ok(Framework::Express); }
        if check_fastify(&self.root) { return Ok(Framework::Fastify); }

        // Check for generic Vite (React fallback)
        if self.root.join("vite.config.js").exists() ||
           self.root.join("vite.config.ts").exists() {
            return Ok(Framework::ViteReact); // Assume React as default
        }

        Ok(Framework::Unknown)
    }

    fn detect_language(&self, framework: &Framework) -> LumenResult<Language> {
        // First check if Rust (by framework or Cargo.toml + rs files)
        if framework.is_rust() {
            return Ok(Language::Rust);
        }

        // Check for Rust projects (including workspaces)
        if self.root.join("Cargo.toml").exists() {
            // Check if it's a workspace
            if let Ok(cargo_content) = std::fs::read_to_string(self.root.join("Cargo.toml")) {
                if cargo_content.contains("[workspace]") || cargo_content.contains("[workspace.package]") {
                    // It's a Rust workspace - check for .rs files in crates/*/src/
                    let crates_dir = self.root.join("crates");
                    if crates_dir.exists() {
                        if let Ok(crates_entries) = std::fs::read_dir(&crates_dir) {
                            for crate_entry in crates_entries.flatten() {
                                let crate_src = crate_entry.path().join("src");
                                if crate_src.exists() {
                                    if let Ok(src_entries) = std::fs::read_dir(&crate_src) {
                                        if src_entries.filter_map(|e| e.ok())
                                            .any(|e| e.path().extension().map(|e| e == "rs").unwrap_or(false)) {
                                            return Ok(Language::Rust);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Still Rust even if no .rs files found yet
                    return Ok(Language::Rust);
                }
            }

            // Check for standard Rust project with src/ directory
            let src_dir = self.root.join("src");
            if src_dir.exists() {
                if std::fs::read_dir(&src_dir).map(|entries| {
                    entries.filter_map(|e| e.ok())
                        .any(|e| e.path().extension().map(|e| e == "rs").unwrap_or(false))
                }).unwrap_or(false) {
                    return Ok(Language::Rust);
                }
            }
        }

        // Check for TypeScript
        if self.root.join("tsconfig.json").exists() {
            return Ok(Language::TypeScript);
        }

        // Check for Go
        if self.root.join("go.mod").exists() {
            return Ok(Language::Go);
        }

        // Check for Python
        if self.root.join("requirements.txt").exists() ||
           self.root.join("pyproject.toml").exists() ||
           self.root.join("setup.py").exists() {
            return Ok(Language::Python);
        }

        // Check by scanning for source files
        if let Ok(entries) = std::fs::read_dir(&self.root.join("src")) {
            let extensions: Vec<String> = entries.filter_map(|e| e.ok())
                .filter_map(|e| e.path().extension().and_then(|e| e.to_str()).map(|s| s.to_string()))
                .collect();

            if extensions.iter().any(|e| e == "ts" || e == "tsx") {
                return Ok(Language::TypeScript);
            }
            if extensions.iter().any(|e| e == "js" || e == "jsx") {
                return Ok(Language::JavaScript);
            }
            if extensions.iter().any(|e| e == "rs") {
                return Ok(Language::Rust);
            }
            if extensions.iter().any(|e| e == "go") {
                return Ok(Language::Go);
            }
            if extensions.iter().any(|e| e == "py") {
                return Ok(Language::Python);
            }
        }

        // Default to JavaScript (most common for Node projects)
        Ok(Language::JavaScript)
    }

    fn detect_test_runner(&self, framework: &Framework) -> LumenResult<TestRunner> {
        Ok(if framework.is_rust() || self.root.join("Cargo.toml").exists() {
            TestRunner::CargoTest
        } else if self.root.join("vitest.config.ts").exists() {
            TestRunner::Vitest
        } else {
            TestRunner::Jest
        })
    }

    fn detect_package_manager(&self, framework: &Framework) -> Option<String> {
        if framework.is_rust() || self.root.join("Cargo.toml").exists() {
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
