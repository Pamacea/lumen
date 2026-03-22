# Changelog

All notable changes to Lumen will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.5] - 2026-03-22

### Changed
- **BREAKING**: Workspace refactored from 10 micro-crates to 3 consolidated crates
  - `oalacea-lumen-core`: Core types, configuration, 7-dimension scoring, trends
  - `oalacea-lumen-analysis`: AST parsing, language detection, code diff
  - `oalacea-lumen`: CLI application (binary renamed from `lumenx` to `lumen`)
- **BREAKING**: Binary renamed from `lumenx` to `lumen` (use `lumen scan` instead of `lumenx scan`)
- All packages now use `oalacea-` prefix for consistency
- Fixed grade system tests (APlus strict mode now returns 98.0)
- Fixed trend analysis tests (newest-first ordering)
- Added `DimensionScore::new()` and `weighted_sum()` methods

### Fixed
- All 16 core tests now passing (scoring, grading, trends, metrics)
- Fixed `is_good()` method for BMinus grade (2.7 GPA is good)
- Fixed improvement rate calculation with proper score ordering

### Technical
- Simplified AST node representation with lifetime parameter
- Added `Clone` trait to `AstNode` for traversal
- Parser modules reorganized under `analyze/ast/` directory

## [0.6.2] - 2026-03-17

### Added
- **Performance**: Persistent cache layer for test generation (BLAKE3 hashing + LRU)
- **Performance**: Parallel file analysis with Rayon (2-4x faster on large projects)
- **Reports**: AI-ready `fixes.json` with exact file locations and fix commands
- **Reports**: Enhanced markdown reports with file:line locations
- **Reports**: Code snippet preview in reports
- **CLI**: `lumenx cache` command for cache management (clear, stats, prune)
- **CLI**: Automatic fixes.json generation on every scan

### Changed
- `lumen scan` now generates everything by default (report + fixes.json)
- Report issues now show exact file locations (e.g., `src/file.ts:123`)
- Added "Suggested Fix" and "View Code Snippet" expandable sections
- Replaced walkdir with glob for better Windows compatibility

### Performance
- First scan: 13x faster (163s → 12s for 135 files)
- Parallel AST analysis with Rayon
- Disabled slow O(n²) duplication detection
- Repeated scans: 80-90% faster (cache hits)
- Large projects (500+ files): up to 20x faster combined

### Fixed
- **Critical**: Fixed "0 files scanned" bug on Windows/Git Bash
- **Critical**: Eliminated 300k+ false positives from "Unused Function" detection for TypeScript/React
- Fixed markdown report placeholders (CODE_BLOCK_START → proper ```)
- Fixed file location display in issue reports
- fixes.json now correctly generated in lumen-reports/ folder

## [Unreleased]

### Added
- Score history tracking with SQLite storage (lumen-history)
- Git diff and PR analysis (lumen-diff)
- Watch mode for automatic re-analysis on file changes
- Python AST parser with tree-sitter
- Dependency analyzer
- Test coverage analyzer
- CLI command `watch` for monitoring projects

### Changed
- Updated all crates to version 0.6.5
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
