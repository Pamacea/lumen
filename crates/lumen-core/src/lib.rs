//! # Lumen Core
//!
//! Core types and utilities for the Lumen toolkit.

pub mod config;
pub mod error;
pub mod project;

pub use config::{Config, LumenConfig, ScoringConfig};
pub use error::{LumenError, LumenResult};
pub use project::{DatabaseInfo, Framework, Language, PackageJson, Project, ProjectInfo, TestRunner};

/// Lumen version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
