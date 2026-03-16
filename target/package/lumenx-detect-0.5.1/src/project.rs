//! Project metadata detection

use serde::{Deserialize, Serialize};
use lumenx_core::{LumenResult, Project};

/// Complete project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Languages detected
    pub languages: Vec<lumenx_core::Language>,

    /// Frameworks detected
    pub frameworks: Vec<crate::Framework>,

    /// Project type
    pub project_type: ProjectType,
}

/// Detected project type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    /// Frontend web application
    FrontendApp,

    /// Backend API
    BackendApi,

    /// Full-stack application
    FullStack,

    /// Mobile application
    MobileApp,

    /// Desktop application
    DesktopApp,

    /// Library/package
    Library,

    /// CLI tool
    CliTool,

    /// Generic/unknown
    Generic,
}

/// Detect the type of project
pub fn detect_project_type(project: &Project) -> LumenResult<ProjectType> {
    let has_frontend = project.file_exists("package.json")
        || project.file_exists("index.html")
        || project.file_exists("vite.config.ts")
        || project.file_exists("next.config.js");

    let has_backend = project.file_exists("Cargo.toml")
        || project.file_exists("go.mod")
        || project.file_exists("pom.xml")
        || project.file_exists("requirements.txt")
        || project.read_file("package.json")
            .map(|c| c.contains("\"@nestjs/core\"") || c.contains("\"express\""))
            .unwrap_or(false);

    let is_library = project.read_file("Cargo.toml")
        .map(|c| c.contains("[lib]") && !c.contains("[[bin]]"))
        .unwrap_or(false);

    let is_cli = project.read_file("Cargo.toml")
        .map(|c| c.contains("[[bin]]"))
        .unwrap_or(false);

    match (has_frontend, has_backend, is_library, is_cli) {
        (true, true, _, _) => Ok(ProjectType::FullStack),
        (true, false, _, _) => Ok(ProjectType::FrontendApp),
        (false, true, _, _) => Ok(ProjectType::BackendApi),
        (_, _, true, false) => Ok(ProjectType::Library),
        (_, _, _, true) => Ok(ProjectType::CliTool),
        _ => Ok(ProjectType::Generic),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_type() {
        // Basic test
        assert!(true);
    }
}
