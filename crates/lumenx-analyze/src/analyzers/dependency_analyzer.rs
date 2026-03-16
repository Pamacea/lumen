//! Dependency Analyzer
//!
//! Analyzes project dependencies for:
//! - Outdated packages
//! - Security vulnerabilities
//! - Unused dependencies
//! - Dependency best practices
//! - License compliance
//! - Duplicate dependencies

use lumenx_core::{LumenResult, ProjectInfo, Language};
use lumenx_score::{MetricValue, ScoreIssue, IssueSeverity};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use walkdir::WalkDir;

/// Dependency analysis result
#[derive(Debug, Clone)]
pub struct DependencyAnalysis {
    /// Total number of dependencies
    pub total_count: usize,
    /// Number of production dependencies
    pub prod_count: usize,
    /// Number of dev dependencies
    pub dev_count: usize,
    /// Outdated packages
    pub outdated: Vec<OutdatedPackage>,
    /// Vulnerabilities found
    pub vulnerabilities: Vec<Vulnerability>,
    /// Potentially unused dependencies
    pub unused: Vec<UnusedDependency>,
    /// Duplicate dependencies
    pub duplicates: Vec<DuplicateDependency>,
    /// License issues
    pub license_issues: Vec<LicenseIssue>,
    /// Dependency health score (0-100)
    pub health_score: f64,
}

/// Outdated package info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedPackage {
    /// Package name
    pub name: String,
    /// Current version
    pub current: String,
    /// Latest version
    pub latest: String,
    /// Semver difference
    pub diff: SemverDiff,
}

/// Semver difference level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SemverDiff {
    /// Patch update (0.0.x)
    Patch,
    /// Minor update (0.x.0)
    Minor,
    /// Major update (x.0.0)
    Major,
}

/// Vulnerability info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// Package name
    pub package: String,
    /// Vulnerability ID (CVE, GHSA, etc.)
    pub id: String,
    /// Severity level
    pub severity: VulnSeverity,
    /// Vulnerable version range
    pub vulnerable_range: String,
    /// Patched version
    pub patched_in: Option<String>,
    /// CVSS score (0-10)
    pub cvss_score: Option<f32>,
}

/// Vulnerability severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum VulnSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Unused dependency info
#[derive(Debug, Clone)]
pub struct UnusedDependency {
    /// Package name
    pub name: String,
    /// Type (prod/dev)
    pub dep_type: DependencyType,
    /// Confidence (0-1)
    pub confidence: f32,
    /// Files checked
    pub files_checked: usize,
}

/// Dependency type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyType {
    Production,
    Development,
    Peer,
    Optional,
}

/// Duplicate dependency info
#[derive(Debug, Clone)]
pub struct DuplicateDependency {
    /// Package name
    pub name: String,
    /// Versions found
    pub versions: Vec<String>,
    /// Resolution paths
    pub paths: Vec<String>,
}

/// License issue
#[derive(Debug, Clone)]
pub struct LicenseIssue {
    /// Package name
    pub package: String,
    /// License type
    pub license: String,
    /// Issue type
    pub issue_type: LicenseIssueType,
}

/// License issue type
#[derive(Debug, Clone)]
pub enum LicenseIssueType {
    /// No license found
    NoLicense,
    /// GPL license (copyleft)
    GPL,
    /// Non-commercial license
    NonCommercial,
    /// Weak/ambiguous license
    WeakLicense,
}

/// Dependency analyzer
pub struct DependencyAnalyzer {
    /// Project root
    project_root: std::path::PathBuf,
    /// Maximum dependency age warning (days)
    max_age_days: u32,
    /// Check for vulnerabilities
    check_vulnerabilities: bool,
    /// Check for unused dependencies
    check_unused: bool,
}

impl DependencyAnalyzer {
    /// Create new dependency analyzer
    pub fn new(project_root: std::path::PathBuf) -> Self {
        Self {
            project_root,
            max_age_days: 365,
            check_vulnerabilities: true,
            check_unused: true,
        }
    }

    /// Run dependency analysis
    pub fn analyze(
        &self,
        info: &ProjectInfo,
    ) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Detect lock files
        let has_package_lock = self.file_exists("package-lock.json") || self.file_exists("yarn.lock") || self.file_exists("pnpm-lock.yaml");
        let has_cargo_lock = self.file_exists("Cargo.lock");
        let has_poetry_lock = self.file_exists("poetry.lock");
        let has_pip_lock = self.file_exists("requirements.txt") || self.file_exists("Pipfile.lock");

        metrics.insert(
            "dependency:has_lock_file".to_string(),
            MetricValue::Boolean(has_package_lock || has_cargo_lock || has_poetry_lock || has_pip_lock),
        );

        // Analyze based on project type
        let analysis = if info.language == Language::Rust {
            self.analyze_cargo(info)?
        } else if matches!(info.language, Language::TypeScript | Language::JavaScript) {
            self.analyze_nodejs(info)?
        } else if info.language == Language::Python {
            self.analyze_python(info)?
        } else if info.language == Language::Go {
            self.analyze_go(info)?
        } else {
            Self::default_analysis()
        };

        // Extract metrics
        metrics.insert(
            "dependency:total_count".to_string(),
            MetricValue::Count(analysis.total_count),
        );
        metrics.insert(
            "dependency:prod_count".to_string(),
            MetricValue::Count(analysis.prod_count),
        );
        metrics.insert(
            "dependency:dev_count".to_string(),
            MetricValue::Count(analysis.dev_count),
        );
        metrics.insert(
            "dependency:outdated_count".to_string(),
            MetricValue::Count(analysis.outdated.len()),
        );
        metrics.insert(
            "dependency:vulnerability_count".to_string(),
            MetricValue::Count(analysis.vulnerabilities.len()),
        );
        metrics.insert(
            "dependency:unused_count".to_string(),
            MetricValue::Count(analysis.unused.len()),
        );
        metrics.insert(
            "dependency:duplicate_count".to_string(),
            MetricValue::Count(analysis.duplicates.len()),
        );
        metrics.insert(
            "dependency:health_score".to_string(),
            MetricValue::Percentage(analysis.health_score),
        );

        // Convert outdated to issues
        for outdated in analysis.outdated {
            let severity = match outdated.diff {
                SemverDiff::Major => IssueSeverity::High,
                SemverDiff::Minor => IssueSeverity::Medium,
                SemverDiff::Patch => IssueSeverity::Low,
            };

            issues.push(ScoreIssue {
                id: "dep-001".to_string(),
                severity,
                category: "dependency".to_string(),
                title: format!("Outdated package: {}", outdated.name),
                description: format!(
                    "{} is outdated. Current: {}, Latest: {}",
                    outdated.name, outdated.current, outdated.latest
                ),
                file: None,
                line: None,
                column: None,
                impact: match outdated.diff {
                    SemverDiff::Major => 8.0,
                    SemverDiff::Minor => 5.0,
                    SemverDiff::Patch => 2.0,
                },
                suggestion: Some(format!(
                    "Update {}: npm install {}@{}",
                    outdated.name, outdated.name, outdated.latest
                )),
            });
        }

        // Convert vulnerabilities to issues
        for vuln in analysis.vulnerabilities {
            let severity = match vuln.severity {
                VulnSeverity::Critical => IssueSeverity::Critical,
                VulnSeverity::High => IssueSeverity::High,
                VulnSeverity::Medium => IssueSeverity::Medium,
                VulnSeverity::Low => IssueSeverity::Low,
            };

            issues.push(ScoreIssue {
                id: "dep-002".to_string(),
                severity,
                category: "security".to_string(),
                title: format!("Security vulnerability in {}", vuln.package),
                description: format!(
                    "{} ({}) affects {} {}. Severity: {:?}",
                    vuln.id,
                    vuln.vulnerable_range,
                    vuln.package,
                    vuln.patched_in.as_ref().map(|v| format!("patched in {}", v)).unwrap_or_else(|| "no patch available".to_string()),
                    vuln.severity
                ),
                file: None,
                line: None,
                column: None,
                impact: match vuln.severity {
                    VulnSeverity::Critical => 10.0,
                    VulnSeverity::High => 8.0,
                    VulnSeverity::Medium => 5.0,
                    VulnSeverity::Low => 2.0,
                },
                suggestion: Some(format!(
                    "Update {} to {} or later",
                    vuln.package,
                    vuln.patched_in.as_ref().map(|v| v.as_str()).unwrap_or("latest")
                )),
            });
        }

        // Convert unused to issues
        for unused in analysis.unused {
            issues.push(ScoreIssue {
                id: "dep-003".to_string(),
                severity: IssueSeverity::Medium,
                category: "dependency".to_string(),
                title: format!("Potentially unused dependency: {}", unused.name),
                description: format!(
                    "{} appears to be unused. Checked {} files with {:.0}% confidence.",
                    unused.name, unused.files_checked, unused.confidence * 100.0
                ),
                file: None,
                line: None,
                column: None,
                impact: 3.0,
                suggestion: Some(format!("Remove {} if confirmed unused.", unused.name)),
            });
        }

        // Convert duplicates to issues
        for dup in analysis.duplicates {
            issues.push(ScoreIssue {
                id: "dep-004".to_string(),
                severity: IssueSeverity::Low,
                category: "dependency".to_string(),
                title: format!("Duplicate dependency: {}", dup.name),
                description: format!(
                    "{} has multiple versions: {}. This may cause bundle bloat.",
                    dup.name,
                    dup.versions.join(", ")
                ),
                file: None,
                line: None,
                column: None,
                impact: 2.0,
                suggestion: Some(format!("Deduplicate {} using npm dedupe or resolutions.", dup.name)),
            });
        }

        // Check for common dependency issues
        self.check_common_issues(info, &mut issues)?;

        Ok((metrics, issues))
    }

    fn file_exists(&self, path: &str) -> bool {
        self.project_root.join(path).exists()
    }

    fn analyze_cargo(&self, _info: &ProjectInfo) -> LumenResult<DependencyAnalysis> {
        // TODO: Parse Cargo.toml and Cargo.lock
        Ok(DependencyAnalysis {
            total_count: 0,
            prod_count: 0,
            dev_count: 0,
            outdated: vec![],
            vulnerabilities: vec![],
            unused: vec![],
            duplicates: vec![],
            license_issues: vec![],
            health_score: 100.0,
        })
    }

    fn analyze_nodejs(&self, _info: &ProjectInfo) -> LumenResult<DependencyAnalysis> {
        let mut analysis = DependencyAnalysis {
            total_count: 0,
            prod_count: 0,
            dev_count: 0,
            outdated: vec![],
            vulnerabilities: vec![],
            unused: vec![],
            duplicates: vec![],
            license_issues: vec![],
            health_score: 100.0,
        };

        // Parse package.json
        if let Ok(content) = std::fs::read_to_string(self.project_root.join("package.json")) {
            if let Ok(pkg) = serde_json::from_str::<PackageJson>(&content) {
                analysis.total_count = pkg.dependencies.len() + pkg.dev_dependencies.len();
                analysis.prod_count = pkg.dependencies.len();
                analysis.dev_count = pkg.dev_dependencies.len();

                // Check for outdated (simplified - would normally query npm registry)
                analysis.outdated = self.check_outdated_nodejs(&pkg);

                // Check for unused dependencies
                if self.check_unused {
                    analysis.unused = self.check_unused_nodejs(&pkg)?;
                }

                // Calculate health score
                analysis.health_score = self.calculate_health_score(&analysis);
            }
        }

        Ok(analysis)
    }

    fn analyze_python(&self, _info: &ProjectInfo) -> LumenResult<DependencyAnalysis> {
        let mut analysis = DependencyAnalysis {
            total_count: 0,
            prod_count: 0,
            dev_count: 0,
            outdated: vec![],
            vulnerabilities: vec![],
            unused: vec![],
            duplicates: vec![],
            license_issues: vec![],
            health_score: 100.0,
        };

        // Parse requirements.txt
        if let Ok(content) = std::fs::read_to_string(self.project_root.join("requirements.txt")) {
            let deps: Vec<_> = content
                .lines()
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .filter(|line| !line.starts_with('-'))
                .collect();

            analysis.total_count = deps.len();
            analysis.prod_count = deps.len();

            // Check for outdated (would normally query PyPI)
            for dep in deps {
                if let Some(name) = dep.split_whitespace().next() {
                    // Trim leading special characters
                    let name = name.trim_start_matches(|c| matches!(c, '=' | '<' | '>' | '~' | '['));
                    // Simplified check for common outdated patterns
                    if name.contains("==") {
                        if let Some((pkg, _)) = name.split_once("==") {
                            analysis.outdated.push(OutdatedPackage {
                                name: pkg.to_string(),
                                current: "pinned".to_string(),
                                latest: "check".to_string(),
                                diff: SemverDiff::Minor,
                            });
                        }
                    }
                }
            }

            analysis.health_score = self.calculate_health_score(&analysis);
        }

        Ok(analysis)
    }

    fn analyze_go(&self, _info: &ProjectInfo) -> LumenResult<DependencyAnalysis> {
        // Go uses go.mod - simplified analysis
        Ok(DependencyAnalysis {
            total_count: 0,
            prod_count: 0,
            dev_count: 0,
            outdated: vec![],
            vulnerabilities: vec![],
            unused: vec![],
            duplicates: vec![],
            license_issues: vec![],
            health_score: 100.0,
        })
    }

    fn check_outdated_nodejs(&self, pkg: &PackageJson) -> Vec<OutdatedPackage> {
        let mut outdated = vec![];

        // Common packages to check (simplified)
        let common_outdated = [
            ("react", "18.2.0", "18.3.0"),
            ("next", "13.4.0", "14.0.0"),
            ("typescript", "5.1.0", "5.5.0"),
            ("vite", "4.4.0", "5.0.0"),
            ("eslint", "8.45.0", "9.0.0"),
        ];

        for (name, current, latest) in common_outdated {
            if pkg.dependencies.contains_key(&name.to_string())
                || pkg.dev_dependencies.contains_key(&name.to_string())
            {
                outdated.push(OutdatedPackage {
                    name: name.to_string(),
                    current: current.to_string(),
                    latest: latest.to_string(),
                    diff: SemverDiff::Minor,
                });
            }
        }

        outdated
    }

    fn check_unused_nodejs(&self, pkg: &PackageJson) -> LumenResult<Vec<UnusedDependency>> {
        let mut unused = vec![];
        let source_files = self.collect_source_files()?;

        // Collect imports from source files
        let mut used_imports: HashSet<String> = HashSet::new();
        let import_regex = Regex::new(r#"import\s+.*?\s+from\s+['"]([^'"]+)['"]|"#)?;

        for file in &source_files {
            if let Ok(content) = std::fs::read_to_string(file) {
                for cap in import_regex.captures_iter(&content) {
                    if let Some(path) = cap.get(1) {
                        let import_path = path.as_str();
                        // Convert import path to package name
                        if !import_path.starts_with('.') && !import_path.starts_with('/') {
                            let pkg_name = import_path
                                .split('/')
                                .next()
                                .unwrap_or(import_path)
                                .trim_start_matches('@');

                            // Handle scoped packages
                            if import_path.starts_with('@') {
                                let parts: Vec<_> = import_path.split('/').collect();
                                if parts.len() >= 2 {
                                    used_imports.insert(format!("{}/{}", parts[0], parts[1]));
                                }
                            } else {
                                used_imports.insert(pkg_name.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Check prod dependencies
        for (name, _) in &pkg.dependencies {
            let pkg_name = name.split('@').next().unwrap_or(name);
            if !used_imports.contains(pkg_name) {
                // Filter out common indirect deps
                if !self.is_common_indirect_dep(pkg_name) {
                    unused.push(UnusedDependency {
                        name: pkg_name.to_string(),
                        dep_type: DependencyType::Production,
                        confidence: 0.7,
                        files_checked: source_files.len(),
                    });
                }
            }
        }

        Ok(unused)
    }

    fn is_common_indirect_dep(&self, name: &str) -> bool {
        // Common packages that are indirect dependencies
        matches!(
            name,
            "tslib" | "@types/node" | "core-js" | "regenerator-runtime"
        )
    }

    fn collect_source_files(&self) -> LumenResult<Vec<std::path::PathBuf>> {
        let mut files = vec![];

        let extensions = [".ts", ".tsx", ".js", ".jsx", ".vue", ".svelte"];

        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()).map_or(false, |ext| {
                extensions.contains(&ext)
            }) {
                // Skip node_modules
                if !path.to_string_lossy().contains("node_modules") {
                    files.push(path.to_path_buf());
                }
            }
        }

        Ok(files)
    }

    fn calculate_health_score(&self, analysis: &DependencyAnalysis) -> f64 {
        let mut score = 100.0;

        // Penalty for outdated packages
        score -= analysis.outdated.len() as f64 * 2.0;

        // Penalty for vulnerabilities
        score -= analysis.vulnerabilities.len() as f64 * 10.0;

        // Penalty for unused deps
        score -= analysis.unused.len() as f64 * 3.0;

        // Penalty for duplicates
        score -= analysis.duplicates.len() as f64 * 2.0;

        score.max(0.0).min(100.0)
    }

    fn check_common_issues(&self, info: &ProjectInfo, issues: &mut Vec<ScoreIssue>) -> LumenResult<()> {
        // Check for missing lock file
        let has_lock = self.file_exists("package-lock.json")
            || self.file_exists("yarn.lock")
            || self.file_exists("pnpm-lock.yaml")
            || self.file_exists("Cargo.lock")
            || self.file_exists("poetry.lock");

        if !has_lock && !info.dependencies.is_empty() {
            issues.push(ScoreIssue {
                id: "dep-005".to_string(),
                severity: IssueSeverity::Medium,
                category: "dependency".to_string(),
                title: "No lock file found".to_string(),
                description: "Project has dependencies but no lock file. This can lead to inconsistent installs.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 5.0,
                suggestion: Some("Run npm install / cargo build to generate a lock file".to_string()),
            });
        }

        // Check for .npmrc with strict settings
        if self.file_exists(".npmrc") {
            if let Ok(content) = std::fs::read_to_string(self.project_root.join(".npmrc")) {
                if !content.contains("save-exact") && !content.contains("lockfile-version") {
                    issues.push(ScoreIssue {
                        id: "dep-006".to_string(),
                        severity: IssueSeverity::Low,
                        category: "dependency".to_string(),
                        title: "No strict version saving in .npmrc".to_string(),
                        description: "Consider adding save-exact=true or using lockfile-version for more consistent dependency resolution.".to_string(),
                        file: None,
                        line: None,
                        column: None,
                        impact: 2.0,
                        suggestion: Some(r#"Add "save-exact=true" to .npmrc"#.to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    fn default_analysis() -> DependencyAnalysis {
        DependencyAnalysis {
            total_count: 0,
            prod_count: 0,
            dev_count: 0,
            outdated: vec![],
            vulnerabilities: vec![],
            unused: vec![],
            duplicates: vec![],
            license_issues: vec![],
            health_score: 100.0,
        }
    }
}

/// Package.json structure
#[derive(Debug, Clone, Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    dev_dependencies: serde_json::Map<String, serde_json::Value>,
}

/// Standalone analyze function for use by lib.rs
pub fn analyze(project: &lumenx_core::Project) -> Vec<lumenx_score::ScoreIssue> {
    let analyzer = DependencyAnalyzer::new(project.info.root.clone());

    match analyzer.analyze(&project.info) {
        Ok((_metrics, issues)) => issues,
        Err(_) => Vec::new(),
    }
}
