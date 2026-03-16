# Changelog

All notable changes to LumenX will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Score history tracking with SQLite storage (lumenx-history)
- Git diff and PR analysis (lumenx-diff)
- Watch mode for automatic re-analysis on file changes
- Python AST parser with tree-sitter
- Dependency analyzer
- Test coverage analyzer
- CLI command `watch` for monitoring projects

### Changed
- Updated all crates to version 0.6.0
- Migrated to git2 0.19 API
- Migrated to notify 6.0 API
- Fixed rusqlite integration
- Updated tree-sitter parser implementations

### Fixed
- Critical: MetricValue enum - replaced Score with Percentage
- Critical: Fixed LumenError variants (Other → AnalysisFailed)
- Fixed foreach callback signature in git2 0.19
- Fixed notify recommended_watcher API changes
- Fixed tree-sitter Python parser cursor traversal

## [0.5.3] - 2025-03-XX

### Added
- Initial test generation capabilities
- Report generation in multiple formats

## [0.5.0] - 2025-03-XX

### Added
- Initial release of LumenX toolkit
- 7-dimension quality scoring
- Framework detection
- Static code analysis
- Security scanning
- Performance analysis
- SEO analysis
- UI/UX analysis
- Documentation analysis
