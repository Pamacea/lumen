//! Project detection and information types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Supported frameworks
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    // Frontend
    NextJs,
    Remix,
    SvelteKit,
    Nuxt,
    Astro,
    ViteReact,
    ViteVue,
    ViteSvelte,
    Angular,
    Solid,
    // Backend
    Express,
    Fastify,
    NestJS,
    Axum,
    ActixWeb,
    Rocket,
    Poem,
    // Mobile
    ReactNative,
    Flutter,
    // Unknown
    Unknown,
}

impl Framework {
    /// Get the display name
    pub fn display_name(&self) -> &str {
        match self {
            Framework::NextJs => "Next.js",
            Framework::Remix => "Remix",
            Framework::SvelteKit => "SvelteKit",
            Framework::Nuxt => "Nuxt",
            Framework::Astro => "Astro",
            Framework::ViteReact => "Vite + React",
            Framework::ViteVue => "Vite + Vue",
            Framework::ViteSvelte => "Vite + Svelte",
            Framework::Angular => "Angular",
            Framework::Solid => "Solid",
            Framework::Express => "Express",
            Framework::Fastify => "Fastify",
            Framework::NestJS => "NestJS",
            Framework::Axum => "Axum",
            Framework::ActixWeb => "Actix Web",
            Framework::Rocket => "Rocket",
            Framework::Poem => "Poem",
            Framework::ReactNative => "React Native",
            Framework::Flutter => "Flutter",
            Framework::Unknown => "Unknown",
        }
    }

    /// Check if this is a Rust framework
    pub fn is_rust(&self) -> bool {
        matches!(self, Framework::Axum | Framework::ActixWeb | Framework::Rocket | Framework::Poem)
    }

    /// Check if this is a Node.js framework
    pub fn is_nodejs(&self) -> bool {
        matches!(
            self,
            Framework::NextJs | Framework::Remix | Framework::Express | Framework::Fastify | Framework::NestJS
        )
    }
}

/// Programming language
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    TypeScript,
    JavaScript,
    Rust,
    Python,
    Go,
    Java,
    CSharp,
    Unknown,
}

impl Language {
    pub fn display_name(&self) -> &str {
        match self {
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::CSharp => "C#",
            Language::Unknown => "Unknown",
        }
    }
}

/// Test runner
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum TestRunner {
    Vitest,
    Jest,
    Mocha,
    CargoTest,
    CargoNextest,
    Pytest,
    GoTest,
    JUnit,
    Unknown,
}

impl TestRunner {
    pub fn display_name(&self) -> &str {
        match self {
            TestRunner::Vitest => "Vitest",
            TestRunner::Jest => "Jest",
            TestRunner::Mocha => "Mocha",
            TestRunner::CargoTest => "cargo test",
            TestRunner::CargoNextest => "cargo nextest",
            TestRunner::Pytest => "pytest",
            TestRunner::GoTest => "go test",
            TestRunner::JUnit => "JUnit",
            TestRunner::Unknown => "Unknown",
        }
    }
}

/// Database information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseInfo {
    pub name: String,
    pub version: Option<String>,
    pub orm: Option<String>,
}

/// Package.json content (partial)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageJson {
    pub name: String,
    pub version: Option<String>,
    pub dependencies: Option<HashMap<String, String>>,
    pub dev_dependencies: Option<HashMap<String, String>>,
    pub scripts: Option<HashMap<String, String>>,
}

/// Complete project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Project name
    pub name: String,
    /// Root directory
    pub root: PathBuf,
    /// Detected framework
    pub framework: Framework,
    /// Primary language
    pub language: Language,
    /// Test runner
    pub test_runner: TestRunner,
    /// Package manager
    pub package_manager: Option<String>,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Dev dependencies
    pub dev_dependencies: Vec<String>,
    /// Database
    pub database: Option<DatabaseInfo>,
    /// Package.json if available
    pub package_json: Option<PackageJson>,
    /// Cargo.toml dependencies if available
    pub cargo_dependencies: Option<HashMap<String, String>>,
}

/// Project analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Project info
    pub info: ProjectInfo,
    /// Source files
    pub source_files: Vec<PathBuf>,
    /// Test files
    pub test_files: Vec<PathBuf>,
    /// Config files
    pub config_files: Vec<PathBuf>,
    /// Total lines of code
    pub total_lines: usize,
    /// Test coverage (if available)
    pub coverage: Option<f64>,
}
