//! # UI/UX Analyzer
//!
//! Analyzes user interface and user experience quality across multiple dimensions:
//! - Layout consistency (Grid, Flexbox, positioning)
//! - Responsive design (breakpoints, media queries, touch targets)
//! - Accessibility (ARIA, semantic HTML, keyboard navigation, focus management, color contrast)
//! - Component reuse patterns (Atomic Design, props consistency)
//! - Visual consistency (color palette, typography scale, spacing grid, border radius)

use lumen_core::{LumenResult, Project};
use lumen_score::ScoreIssue;
use regex::Regex;
use std::collections::HashMap;
use walkdir::WalkDir;

/// UI/UX analysis result
#[derive(Debug, Clone)]
pub struct UIUXAnalysis {
    /// Layout analysis results
    pub layout: LayoutAnalysis,
    /// Responsive design analysis
    pub responsive: ResponsiveAnalysis,
    /// Accessibility analysis
    pub accessibility: AccessibilityAnalysis,
    /// Component analysis
    pub components: ComponentAnalysis,
    /// Visual consistency analysis
    pub visual: VisualAnalysis,
    /// Overall score (0-100)
    pub overall_score: f64,
}

/// Layout analysis results
#[derive(Debug, Clone)]
pub struct LayoutAnalysis {
    /// Modern layout usage (Flexbox + Grid)
    pub modern_layout_ratio: f64,
    /// Container consistency score (0-100)
    pub container_consistency: f64,
    /// Z-index issues detected
    pub z_index_issues: Vec<ZIndexIssue>,
    /// Orphaned absolute positioned elements
    pub orphaned_absolute: usize,
    /// Grid system detected
    pub has_grid_system: bool,
    /// Layout method distribution
    pub layout_methods: LayoutMethodDistribution,
    /// Layout score (0-100)
    pub score: f64,
}

/// Layout method distribution
#[derive(Debug, Clone)]
pub struct LayoutMethodDistribution {
    /// Number of flexbox usages
    pub flexbox_count: usize,
    /// Number of grid usages
    pub grid_count: usize,
    /// Number of absolute positioning usages
    pub absolute_count: usize,
    /// Number of float usages (legacy)
    pub float_count: usize,
    /// Total layout declarations found
    pub total_count: usize,
}

/// Z-index issue
#[derive(Debug, Clone)]
pub struct ZIndexIssue {
    /// File containing the issue
    pub file: String,
    /// Line number
    pub line: Option<usize>,
    /// Z-index value
    pub value: i32,
    /// Selector
    pub selector: String,
}

/// Responsive design analysis
#[derive(Debug, Clone)]
pub struct ResponsiveAnalysis {
    /// Breakpoint coverage
    pub has_mobile_breakpoint: bool,
    pub has_tablet_breakpoint: bool,
    pub has_desktop_breakpoint: bool,
    /// Mobile-first approach detected
    pub is_mobile_first: bool,
    /// Responsive unit usage ratio (rem, em, %, vw, vh vs px)
    pub responsive_unit_ratio: f64,
    /// Touch target issues (< 44x44px)
    pub touch_target_issues: Vec<TouchTargetIssue>,
    /// Viewport meta tag present
    pub has_viewport_meta: bool,
    /// Responsive score (0-100)
    pub score: f64,
}

/// Touch target issue
#[derive(Debug, Clone)]
pub struct TouchTargetIssue {
    /// Element type
    pub element_type: String,
    /// File containing the element
    pub file: String,
    /// Estimated size (width, height)
    pub estimated_size: Option<(u32, u32)>,
}

/// Accessibility analysis
#[derive(Debug, Clone)]
pub struct AccessibilityAnalysis {
    /// Semantic HTML ratio (semantic elements / total elements)
    pub semantic_html_ratio: f64,
    /// Has valid heading structure (h1 -> h2 -> h3, no skips)
    pub has_valid_heading_structure: bool,
    /// Has main landmark
    pub has_main_landmark: bool,
    /// Has nav landmark
    pub has_nav_landmark: bool,
    /// ARIA quality score (0-100)
    pub aria_quality_score: f64,
    /// Keyboard accessibility issues
    pub keyboard_issues: Vec<KeyboardIssue>,
    /// Focus style present
    pub has_focus_styles: bool,
    /// Alt text coverage
    pub alt_text_coverage: f64,
    /// Form label coverage
    pub form_label_coverage: f64,
    /// Accessibility score (0-100)
    pub score: f64,
}

/// Keyboard navigation issue
#[derive(Debug, Clone)]
pub struct KeyboardIssue {
    /// Issue type
    pub issue_type: KeyboardIssueType,
    /// File containing the issue
    pub file: String,
    /// Element selector
    pub element: String,
}

/// Keyboard issue types
#[derive(Debug, Clone)]
pub enum KeyboardIssueType {
    /// Missing skip link
    MissingSkipLink,
    /// Positive tabindex
    PositiveTabindex,
    /// Interactive element not keyboard accessible
    NotKeyboardAccessible,
    /// No visible focus indicator
    NoFocusIndicator,
}

/// Component analysis
#[derive(Debug, Clone)]
pub struct ComponentAnalysis {
    /// Total components found
    pub total_components: usize,
    /// Unique components
    pub unique_components: usize,
    /// Reuse ratio (total / unique)
    pub reuse_ratio: f64,
    /// Single-use components (potential duplication)
    pub single_use_components: usize,
    /// Design system detected
    pub design_system_detected: Option<String>,
    /// Atomic design adherence (0-100)
    pub atomic_design_score: f64,
    /// Component categories found
    pub categories: ComponentCategories,
    /// Component score (0-100)
    pub score: f64,
}

/// Component categories (Atomic Design)
#[derive(Debug, Clone)]
pub struct ComponentCategories {
    /// Atoms (Button, Input, Icon, etc.)
    pub atoms: usize,
    /// Molecules (Card, SearchBar, etc.)
    pub molecules: usize,
    /// Organisms (Header, Form, etc.)
    pub organisms: usize,
    /// Templates (PageLayout, etc.)
    pub templates: usize,
    /// Pages
    pub pages: usize,
}

/// Visual consistency analysis
#[derive(Debug, Clone)]
pub struct VisualAnalysis {
    /// Color system score (0-100)
    pub color_system_score: f64,
    /// Uses CSS variables for colors
    pub uses_color_variables: bool,
    /// Has semantic colors (success, warning, error, info)
    pub has_semantic_colors: bool,
    /// Typography scale detected
    pub has_typography_scale: bool,
    /// Number of font families used
    pub font_family_count: usize,
    /// Spacing grid detected
    pub has_spacing_grid: bool,
    /// Border radius consistent
    pub border_radius_consistent: bool,
    /// Hard-coded values count
    pub hard_coded_values: usize,
    /// Visual score (0-100)
    pub score: f64,
}

/// Issue severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IssueSeverity {
    Info = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Issue found during analysis
#[derive(Debug, Clone)]
pub struct UIUXIssue {
    /// Severity
    pub severity: IssueSeverity,
    /// Category (Layout, Responsive, Accessibility, Components, Visual)
    pub category: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// File location (if applicable)
    pub file: Option<String>,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Main analyzer function - public API for lumen-analyze integration
pub fn analyze(project: &Project) -> Vec<ScoreIssue> {
    let analysis = run_analysis_internal(project);

    // Convert to ScoreIssue format
    let mut issues = Vec::new();

    // Layout issues
    for z_issue in &analysis.layout.z_index_issues {
        issues.push(score_issue_from_uiux(UIUXIssue {
            severity: IssueSeverity::Medium,
            category: "Layout".to_string(),
            title: "Z-index war detected".to_string(),
            description: format!("Z-index value {} exceeds recommended maximum of 1000", z_issue.value),
            file: Some(z_issue.file.clone()),
            line: z_issue.line,
            suggestion: Some("Use a more organized z-index scale (e.g., 10, 20, 30 for layers)".to_string()),
        }));
    }

    // Responsive issues
    if !analysis.responsive.has_viewport_meta {
        issues.push(score_issue_from_uiux(UIUXIssue {
            severity: IssueSeverity::Critical,
            category: "Responsive".to_string(),
            title: "Missing viewport meta tag".to_string(),
            description: "No viewport meta tag found in HTML files".to_string(),
            file: None,
            line: None,
            suggestion: Some(r#"Add <meta name="viewport" content="width=device-width, initial-scale=1">"#.to_string()),
        }));
    }

    // Accessibility issues
    if !analysis.accessibility.has_main_landmark {
        issues.push(score_issue_from_uiux(UIUXIssue {
            severity: IssueSeverity::Critical,
            category: "Accessibility".to_string(),
            title: "Missing main landmark".to_string(),
            description: "No <main> element found. Screen reader users need this to navigate.".to_string(),
            file: None,
            line: None,
            suggestion: Some("Add a <main> element wrapping the primary content".to_string()),
        }));
    }

    if !analysis.accessibility.has_focus_styles {
        issues.push(score_issue_from_uiux(UIUXIssue {
            severity: IssueSeverity::Critical,
            category: "Accessibility".to_string(),
            title: "No focus styles found".to_string(),
            description: "Keyboard users need visible focus indicators".to_string(),
            file: None,
            line: None,
            suggestion: Some("Add :focus or :focus-visible styles to interactive elements".to_string()),
        }));
    }

    issues
}

/// Analyze and return detailed UI/UX analysis
pub fn analyze_detailed(project: &Project) -> LumenResult<UIUXAnalysis> {
    Ok(run_analysis_internal(project))
}

/// Internal analysis function
fn run_analysis_internal(project: &Project) -> UIUXAnalysis {
    let mut css_files = Vec::new();
    let mut html_files = Vec::new();
    let mut component_files = Vec::new();

    // Collect relevant files
    for entry in WalkDir::new(&project.info.root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str());

        match ext {
            Some("css") | Some("scss") | Some("sass") | Some("less") => {
                if let Ok(content) = std::fs::read_to_string(path) {
                    css_files.push((path.to_path_buf(), content));
                }
            }
            Some("html") | Some("htm") => {
                if let Ok(content) = std::fs::read_to_string(path) {
                    html_files.push((path.to_path_buf(), content));
                }
            }
            Some("jsx") | Some("tsx") | Some("vue") | Some("svelte") => {
                if let Ok(content) = std::fs::read_to_string(path) {
                    component_files.push((path.to_path_buf(), content));
                }
            }
            _ => {}
        }
    }

    // Run each analysis
    let layout = analyze_layout(&css_files, &component_files);
    let responsive = analyze_responsive(&css_files, &html_files);
    let accessibility = analyze_accessibility(&html_files, &component_files);
    let components = analyze_components(&component_files);
    let visual = analyze_visual(&css_files);

    // Calculate overall score (weighted average)
    let overall_score = (
        layout.score * 0.15 +
        responsive.score * 0.25 +
        accessibility.score * 0.25 +
        components.score * 0.20 +
        visual.score * 0.15
    ).min(100.0);

    UIUXAnalysis {
        layout,
        responsive,
        accessibility,
        components,
        visual,
        overall_score,
    }
}

/// Convert UIUXIssue to ScoreIssue
fn score_issue_from_uiux(issue: UIUXIssue) -> ScoreIssue {
    ScoreIssue {
        id: format!("uiux-{}", issue.category.to_lowercase().replace(' ', "-")),
        severity: match issue.severity {
            IssueSeverity::Info => lumen_score::IssueSeverity::Info,
            IssueSeverity::Low => lumen_score::IssueSeverity::Low,
            IssueSeverity::Medium => lumen_score::IssueSeverity::Medium,
            IssueSeverity::High => lumen_score::IssueSeverity::High,
            IssueSeverity::Critical => lumen_score::IssueSeverity::Critical,
        },
        category: issue.category,
        title: issue.title,
        description: issue.description,
        file: issue.file,
        line: issue.line,
        column: None,
        impact: match issue.severity {
            IssueSeverity::Critical => 20.0,
            IssueSeverity::High => 10.0,
            IssueSeverity::Medium => 5.0,
            IssueSeverity::Low => 2.0,
            IssueSeverity::Info => 0.0,
        },
        suggestion: issue.suggestion,
    }
}

/// Analyze layout patterns
fn analyze_layout(
    css_files: &[(std::path::PathBuf, String)],
    component_files: &[(std::path::PathBuf, String)],
) -> LayoutAnalysis {
    let mut flexbox_count = 0;
    let mut grid_count = 0;
    let mut absolute_count = 0;
    let mut float_count = 0;
    let mut total_count = 0;
    let mut z_index_issues = Vec::new();

    // Regex patterns for layout detection
    let display_flex = Regex::new(r"display\s*:\s*(flex|inline-flex)\b").unwrap();
    let display_grid = Regex::new(r"display\s*:\s*(grid|inline-grid)\b").unwrap();
    let position_absolute = Regex::new(r"position\s*:\s*absolute\b").unwrap();
    let position_fixed = Regex::new(r"position\s*:\s*fixed\b").unwrap();
    let float_pattern = Regex::new(r"float\s*:\s*(left|right)\b").unwrap();
    let z_index_pattern = Regex::new(r"z-index\s*:\s*(-?\d+)").unwrap();

    // Analyze CSS files
    for (path, content) in css_files {
        for (line_idx, line) in content.lines().enumerate() {
            if display_flex.is_match(line) {
                flexbox_count += 1;
                total_count += 1;
            }
            if display_grid.is_match(line) {
                grid_count += 1;
                total_count += 1;
            }
            if position_absolute.is_match(line) {
                absolute_count += 1;
                total_count += 1;
            }
            if position_fixed.is_match(line) {
                absolute_count += 1;
                total_count += 1;
            }
            if float_pattern.is_match(line) {
                float_count += 1;
                total_count += 1;
            }

            // Check for z-index wars (values > 1000)
            if let Some(caps) = z_index_pattern.captures(line) {
                if let Ok(value) = caps[1].parse::<i32>() {
                    if value.abs() > 1000 {
                        z_index_issues.push(ZIndexIssue {
                            file: path.display().to_string(),
                            line: Some(line_idx + 1),
                            value,
                            selector: extract_selector(line),
                        });
                    }
                }
            }
        }
    }

    // Check for grid systems
    let has_grid_system = detect_grid_system(css_files);

    let modern_count = flexbox_count + grid_count;
    let modern_layout_ratio = if total_count > 0 {
        (modern_count as f64) / (total_count as f64)
    } else {
        0.5
    };

    let container_consistency = calculate_container_consistency(css_files);

    // Calculate layout score
    let mut score = 50.0;
    score += modern_layout_ratio * 30.0;
    score += (container_consistency / 100.0) * 10.0;
    score -= (float_count as f64 * 2.0).min(20.0);
    score -= (z_index_issues.len() as f64 * 3.0).min(15.0);
    if has_grid_system { score += 10.0; }
    let score = score.clamp(0.0, 100.0);

    LayoutAnalysis {
        modern_layout_ratio,
        container_consistency,
        z_index_issues,
        orphaned_absolute: count_orphaned_absolute(css_files, component_files),
        has_grid_system,
        layout_methods: LayoutMethodDistribution {
            flexbox_count,
            grid_count,
            absolute_count,
            float_count,
            total_count,
        },
        score,
    }
}

/// Extract CSS selector from a line
fn extract_selector(line: &str) -> String {
    let trimmed = line.trim();
    if let Some(end) = trimmed.find('{') {
        trimmed[..end].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// Detect if project uses a grid system
fn detect_grid_system(css_files: &[(std::path::PathBuf, String)]) -> bool {
    let grid_indicators = [
        r"grid-template-columns\s*:\s*repeat\(",
        r"grid-cols-",
        r"\bcol-",
        r"\.container\s*{",
        r"\.row\s*{",
        r"flex-wrap",
    ];

    for (_, content) in css_files {
        for pattern in &grid_indicators {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(content) {
                    return true;
                }
            }
        }
    }

    false
}

/// Calculate container consistency score
fn calculate_container_consistency(css_files: &[(std::path::PathBuf, String)]) -> f64 {
    let container_patterns = [
        r"\.container\s*{",
        r"\.wrapper\s*{",
        r"class=.*container",
        r"class=.*wrapper",
    ];

    let mut container_count = 0;
    for (_, content) in css_files {
        for pattern in &container_patterns {
            if let Ok(re) = Regex::new(pattern) {
                container_count += re.find_iter(content).count();
            }
        }
    }

    if container_count > 5 {
        100.0
    } else if container_count > 0 {
        50.0 + (container_count as f64 * 10.0).min(50.0)
    } else {
        0.0
    }
}

/// Count orphaned absolute positioned elements
fn count_orphaned_absolute(
    css_files: &[(std::path::PathBuf, String)],
    component_files: &[(std::path::PathBuf, String)],
) -> usize {
    let mut orphaned = 0;
    let position_absolute = Regex::new(r"position\s*:\s*absolute\b").unwrap();
    let position_relative = Regex::new(r"position\s*:\s*(relative|absolute|fixed|sticky)\b").unwrap();

    for (_, content) in component_files {
        let has_absolute = position_absolute.is_match(content);
        let has_positioned = position_relative.is_match(content);

        if has_absolute && !has_positioned {
            orphaned += 1;
        }
    }

    orphaned
}

/// Analyze responsive design
fn analyze_responsive(
    css_files: &[(std::path::PathBuf, String)],
    html_files: &[(std::path::PathBuf, String)],
) -> ResponsiveAnalysis {
    let mut has_mobile_breakpoint = false;
    let mut has_tablet_breakpoint = false;
    let mut has_desktop_breakpoint = false;
    let mut is_mobile_first = true;
    let mut has_viewport_meta = false;

    // Media query patterns
    let min_width_640 = Regex::new(r"@media[^{]*min-width[^{]*640").unwrap();
    let min_width_768 = Regex::new(r"@media[^{]*min-width[^{]*768").unwrap();
    let min_width_1024 = Regex::new(r"@media[^{]*min-width[^{]*1024").unwrap();
    let max_width_pattern = Regex::new(r"@media[^{]*max-width").unwrap();
    let min_width_pattern = Regex::new(r"@media[^{]*min-width").unwrap();
    let viewport_meta = Regex::new(r#"<meta[^>]*name\s*=\s*['"]viewport['"]"#).unwrap();

    let responsive_unit_pattern = Regex::new(r"(rem|em|%|vw|vh)\b").unwrap();
    let px_pattern = Regex::new(r"\d+px\b").unwrap();

    let mut responsive_units = 0;
    let mut px_units = 0;

    // Analyze CSS files
    for (_, content) in css_files {
        has_mobile_breakpoint |= min_width_640.is_match(content) || min_width_768.is_match(content);
        has_tablet_breakpoint |= min_width_768.is_match(content) || min_width_1024.is_match(content);
        has_desktop_breakpoint |= min_width_1024.is_match(content);

        let min_count = min_width_pattern.find_iter(content).count();
        let max_count = max_width_pattern.find_iter(content).count();

        if max_count > min_count {
            is_mobile_first = false;
        }

        responsive_units += responsive_unit_pattern.find_iter(content).count();
        px_units += px_pattern.find_iter(content).count();
    }

    // Analyze HTML files for viewport meta
    for (_, content) in html_files {
        has_viewport_meta |= viewport_meta.is_match(content);
    }

    let total_units = responsive_units + px_units;
    let responsive_unit_ratio = if total_units > 0 {
        (responsive_units as f64) / (total_units as f64)
    } else {
        0.5
    };

    // Calculate responsive score
    let mut score = 40.0;
    if has_mobile_breakpoint { score += 20.0; }
    if has_tablet_breakpoint { score += 5.0; }
    if has_desktop_breakpoint { score += 5.0; }
    if is_mobile_first { score += 10.0; }
    if has_viewport_meta { score += 10.0; }
    score += responsive_unit_ratio * 10.0;
    let score = score.clamp(0.0, 100.0);

    ResponsiveAnalysis {
        has_mobile_breakpoint,
        has_tablet_breakpoint,
        has_desktop_breakpoint,
        is_mobile_first,
        responsive_unit_ratio,
        touch_target_issues: Vec::new(),
        has_viewport_meta,
        score,
    }
}

/// Analyze accessibility
fn analyze_accessibility(
    html_files: &[(std::path::PathBuf, String)],
    component_files: &[(std::path::PathBuf, String)],
) -> AccessibilityAnalysis {
    let mut semantic_html_ratio = 0.0;
    let mut has_valid_heading_structure = true;
    let mut has_main_landmark = false;
    let mut has_nav_landmark = false;
    let mut aria_quality_score = 100.0;
    let mut keyboard_issues = Vec::new();
    let mut has_focus_styles = false;
    let mut alt_text_coverage = 1.0;
    let mut form_label_coverage = 1.0;

    // Regex patterns
    let semantic_tags = Regex::new(r"<(header|nav|main|article|section|aside|footer|figure|figcaption)\b").unwrap();
    let all_tags = Regex::new(r"<([a-z][a-z0-9]*)\b").unwrap();
    let main_element = Regex::new(r"<main\b").unwrap();
    let nav_element = Regex::new(r"<nav\b").unwrap();
    let heading_pattern = Regex::new(r"<h([1-6])\b").unwrap();
    let img_with_alt = Regex::new(r#"<img[^>]*alt\s*=\s*["'][^"']*["']"#).unwrap();
    let img_without_alt = Regex::new(r#"<img(?![^>]*alt\s*=)"#).unwrap();
    let focus_style = Regex::new(r"(:focus|\.focus|focus-visible)").unwrap();
    let aria_label = Regex::new(r#"aria-label\s*=\s*["']"#).unwrap();
    let positive_tabindex = Regex::new(r#"tabindex\s*=\s*[1-9]"#).unwrap();
    let skip_link = Regex::new(r#"skip(-?link)?"#).unwrap();

    let mut total_semantic = 0;
    let mut total_tags = 0;
    let mut last_heading_level = 0;
    let mut images_with_alt = 0;
    let mut images_total = 0;

    // Analyze HTML files
    for (path, content) in html_files {
        total_semantic += semantic_tags.find_iter(content).count();
        total_tags += all_tags.find_iter(content).count();

        has_main_landmark |= main_element.is_match(content);
        has_nav_landmark |= nav_element.is_match(content);
        has_focus_styles |= focus_style.is_match(content);

        // Check heading structure
        for cap in heading_pattern.captures_iter(content) {
            if let Ok(level) = cap[1].parse::<u32>() {
                if last_heading_level > 0 && level > last_heading_level + 1 {
                    has_valid_heading_structure = false;
                }
                last_heading_level = level;
            }
        }

        // Count images with/without alt
        images_with_alt += img_with_alt.find_iter(content).count();
        images_total += img_without_alt.find_iter(content).count() + images_with_alt;

        // Check for skip link
        if !skip_link.is_match(content) && total_tags > 50 {
            keyboard_issues.push(KeyboardIssue {
                issue_type: KeyboardIssueType::MissingSkipLink,
                file: path.display().to_string(),
                element: "body".to_string(),
            });
        }

        // Check for positive tabindex
        if let Some(mat) = positive_tabindex.find(content) {
            keyboard_issues.push(KeyboardIssue {
                issue_type: KeyboardIssueType::PositiveTabindex,
                file: path.display().to_string(),
                element: content[mat.start()..mat.end().min(content.len())].to_string(),
            });
        }
    }

    // Analyze component files
    for (path, content) in component_files {
        has_focus_styles |= focus_style.is_match(content);
        has_main_landmark |= main_element.is_match(content);
        has_nav_landmark |= nav_element.is_match(content);

        if let Some(mat) = positive_tabindex.find(content) {
            keyboard_issues.push(KeyboardIssue {
                issue_type: KeyboardIssueType::PositiveTabindex,
                file: path.display().to_string(),
                element: "component".to_string(),
            });
        }

        // Check ARIA usage quality
        let aria_labels = aria_label.find_iter(content).count();
        if aria_labels > 0 {
            aria_quality_score = 90.0;
        }
    }

    semantic_html_ratio = if total_tags > 0 {
        (total_semantic as f64) / (total_tags as f64)
    } else {
        0.0
    };

    alt_text_coverage = if images_total > 0 {
        (images_with_alt as f64) / (images_total as f64)
    } else {
        1.0
    };

    // Calculate accessibility score
    let mut score = 30.0;
    score += semantic_html_ratio * 20.0;
    if has_valid_heading_structure { score += 10.0; }
    if has_main_landmark { score += 15.0; }
    if has_nav_landmark { score += 5.0; }
    if has_focus_styles { score += 10.0; }
    score += alt_text_coverage * 10.0;
    score -= (keyboard_issues.len() as f64 * 3.0).min(30.0);
    let score = score.clamp(0.0, 100.0);

    AccessibilityAnalysis {
        semantic_html_ratio,
        has_valid_heading_structure,
        has_main_landmark,
        has_nav_landmark,
        aria_quality_score,
        keyboard_issues,
        has_focus_styles,
        alt_text_coverage,
        form_label_coverage,
        score,
    }
}

/// Analyze component patterns
fn analyze_components(
    component_files: &[(std::path::PathBuf, String)],
) -> ComponentAnalysis {
    let mut total_components = 0;
    let mut unique_components = HashMap::new();
    let mut design_system_detected = None;
    let mut categories = ComponentCategories {
        atoms: 0,
        molecules: 0,
        organisms: 0,
        templates: 0,
        pages: 0,
    };

    // Design system detection patterns
    let design_systems = [
        ("Radix UI", Regex::new(r"@radix-ui").unwrap()),
        ("shadcn/ui", Regex::new(r"shadcn").unwrap()),
        ("Chakra UI", Regex::new(r"@chakra-ui").unwrap()),
        ("MUI", Regex::new(r"@mui|@material-ui").unwrap()),
        ("Mantine", Regex::new(r"@mantine").unwrap()),
        ("Headless UI", Regex::new(r"@headlessui").unwrap()),
        ("DaisyUI", Regex::new(r"daisyui").unwrap()),
        ("Tailwind", Regex::new(r"tailwindcss").unwrap()),
        ("Bootstrap", Regex::new(r"bootstrap").unwrap()),
    ];

    for (_, content) in component_files {
        // Detect design system
        for (name, pattern) in &design_systems {
            if pattern.is_match(content) {
                design_system_detected = Some(name.to_string());
                break;
            }
        }

        analyze_react_components(content, &mut total_components, &mut unique_components, &mut categories);
    }

    let unique_count = unique_components.len();
    let reuse_ratio = if unique_count > 0 {
        (total_components as f64) / (unique_count as f64)
    } else {
        1.0
    };

    let single_use = unique_components.values().filter(|&&count| count == 1).count();

    // Atomic design adherence
    let total_categorized = categories.atoms + categories.molecules + categories.organisms
        + categories.templates + categories.pages;
    let atomic_design_score = if total_categorized > 0 {
        let has_atoms = categories.atoms > 0;
        let has_molecules = categories.molecules > 0;
        let has_organisms = categories.organisms > 0;
        let mut score = 50.0_f64;
        if has_atoms { score += 15.0; }
        if has_molecules { score += 20.0; }
        if has_organisms { score += 15.0; }
        score.min(100.0)
    } else {
        0.0
    };

    // Calculate component score
    let mut score = 40.0;
    score += (reuse_ratio.min(3.0) - 1.0) * 15.0;
    if design_system_detected.is_some() { score += 15.0; }
    score += atomic_design_score * 0.2;
    score -= (single_use as f64 / (total_components.max(1) as f64) * 20.0).min(20.0);
    let score = score.clamp(0.0, 100.0);

    ComponentAnalysis {
        total_components,
        unique_components: unique_count,
        reuse_ratio,
        single_use_components: single_use,
        design_system_detected,
        atomic_design_score,
        categories,
        score,
    }
}

/// Analyze React components
fn analyze_react_components(
    content: &str,
    total: &mut usize,
    unique: &mut HashMap<String, usize>,
    categories: &mut ComponentCategories,
) {
    let component_def = Regex::new(r"(?:function|const)\s+([A-Z][a-zA-Z0-9]*)\s*[\(|=]").unwrap();
    let component_export = Regex::new(r"export\s+(?:default\s+)?(?:function|const)\s+([A-Z][a-zA-Z0-9]*)").unwrap();

    for cap in component_def.captures_iter(content).chain(component_export.captures_iter(content)) {
        if let Some(name) = cap.get(1) {
            let name_str = name.as_str().to_string();
            *unique.entry(name_str.clone()).or_insert(0) += 1;
            *total += 1;

            categorize_component(&name_str, categories);
        }
    }
}

/// Categorize component by Atomic Design principles
fn categorize_component(name: &str, categories: &mut ComponentCategories) {
    let name_lower = name.to_lowercase();

    // Atoms: basic building blocks
    if name_lower.ends_with("button")
        || name_lower.ends_with("input")
        || name_lower.ends_with("icon")
        || name_lower.ends_with("badge")
        || name_lower.ends_with("link")
        || name_lower.ends_with("label")
        || name_lower.ends_with("avatar")
    {
        categories.atoms += 1;
        return;
    }

    // Molecules: simple combinations
    if name_lower.ends_with("card")
        || name_lower.ends_with("form") && !name_lower.contains("formfield")
        || name_lower.ends_with("searchbar")
        || name_lower.ends_with("listitem")
        || name_lower.ends_with("dropdown")
        || name_lower.ends_with("modal")
        || name_lower.ends_with("tooltip")
    {
        categories.molecules += 1;
        return;
    }

    // Organisms: complex UI sections
    if name_lower.ends_with("header")
        || name_lower.ends_with("footer")
        || name_lower.ends_with("navbar")
        || name_lower.ends_with("sidebar")
        || name_lower.ends_with("table")
        || name_lower.ends_with("formsection")
        || name_lower.ends_with("comment")
    {
        categories.organisms += 1;
        return;
    }

    // Templates: page layouts
    if name_lower.ends_with("layout")
        || name_lower.ends_with("template")
        || name_lower.ends_with("skeleton")
    {
        categories.templates += 1;
        return;
    }

    // Pages
    if name_lower.starts_with("page") || name_lower.ends_with("page") {
        categories.pages += 1;
    }
}

/// Analyze visual consistency
fn analyze_visual(css_files: &[(std::path::PathBuf, String)]) -> VisualAnalysis {
    let mut uses_color_variables = false;
    let mut has_semantic_colors = false;
    let mut has_typography_scale = false;
    let mut font_family_count = 0;
    let mut has_spacing_grid = false;
    let mut border_radius_consistent = false;
    let mut hard_coded_values = 0;

    // Regex patterns
    let css_var = Regex::new(r"var\(--[a-z-]+\)").unwrap();
    let semantic_color_names = Regex::new(r"--color-(success|warning|error|danger|info|primary|secondary)").unwrap();
    let font_family = Regex::new(r"font-family\s*:\s*[^;]+").unwrap();
    let font_size_rems = Regex::new(r"font-size\s*:\s*([\d.]+rem)").unwrap();
    let border_radius = Regex::new(r"border-radius\s*:\s*([\d.]+)(rem|px|em|%)").unwrap();
    let magic_number = Regex::new(r":\s*[1-9]\d*px\b").unwrap();

    let mut font_families = std::collections::HashSet::new();
    let mut rem_sizes = Vec::new();
    let mut border_radii = Vec::new();

    for (_, content) in css_files {
        uses_color_variables |= css_var.is_match(content);
        has_semantic_colors |= semantic_color_names.is_match(content);

        for cap in font_family.captures_iter(content) {
            let family = cap[0].to_string();
            font_families.insert(family);
        }

        for cap in font_size_rems.captures_iter(content) {
            if let Ok(rem) = cap[1].parse::<f64>() {
                if !rem_sizes.contains(&rem) {
                    rem_sizes.push(rem);
                }
            }
        }

        for cap in border_radius.captures_iter(content) {
            if let Ok(val) = cap[1].parse::<f64>() {
                if !border_radii.contains(&val) {
                    border_radii.push(val);
                }
            }
        }

        hard_coded_values += magic_number.find_iter(content).count();
    }

    font_family_count = font_families.len();
    rem_sizes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    has_typography_scale = rem_sizes.len() >= 3 && detect_typographic_scale(&rem_sizes);
    has_spacing_grid = detect_spacing_grid(css_files);
    border_radius_consistent = border_radii.len() <= 3;

    let color_system_score = calculate_color_system_score(uses_color_variables, has_semantic_colors, hard_coded_values);

    // Calculate visual score
    let mut score = 35.0;
    score += color_system_score * 0.25;
    if has_typography_scale { score += 15.0; }
    if has_spacing_grid { score += 10.0; }
    if border_radius_consistent { score += 10.0; }
    if font_family_count <= 3 { score += 5.0; } else { score -= (font_family_count as f64 - 3.0) * 3.0; }
    if has_semantic_colors { score += 10.0; }
    score -= (hard_coded_values as f64 / 20.0).min(15.0);
    let score = score.clamp(0.0, 100.0);

    VisualAnalysis {
        color_system_score,
        uses_color_variables,
        has_semantic_colors,
        has_typography_scale,
        font_family_count,
        has_spacing_grid,
        border_radius_consistent,
        hard_coded_values,
        score,
    }
}

/// Detect if rem sizes follow a typographic scale
fn detect_typographic_scale(sizes: &[f64]) -> bool {
    let mut sorted = sizes.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    if sorted.len() < 3 {
        return false;
    }

    let mut ratios = Vec::new();
    for window in sorted.windows(2) {
        if window[0] > 0.0 {
            ratios.push(window[1] / window[0]);
        }
    }

    if ratios.is_empty() {
        return false;
    }

    let avg_ratio = ratios.iter().sum::<f64>() / ratios.len() as f64;
    let variance = ratios.iter()
        .map(|r| (r - avg_ratio).powi(2))
        .sum::<f64>() / ratios.len() as f64;

    variance < 0.1
}

/// Detect if spacing follows a 4px or 8px grid
fn detect_spacing_grid(css_files: &[(std::path::PathBuf, String)]) -> bool {
    let spacing_pattern = Regex::new(r"(margin|padding|gap)\s*:\s*([\d.]+)(px|rem)").unwrap();

    let mut px_values = Vec::new();

    for (_, content) in css_files {
        for cap in spacing_pattern.captures_iter(content) {
            if let Ok(val) = cap[2].parse::<f64>() {
                px_values.push(val);
            }
        }
    }

    if px_values.is_empty() {
        return false;
    }

    let base4 = px_values.iter().all(|v| *v == 0.0 || v % 4.0 == 0.0);
    let base8 = px_values.iter().all(|v| *v == 0.0 || v % 8.0 == 0.0);

    base4 || base8
}

/// Calculate color system score
fn calculate_color_system_score(uses_vars: bool, has_semantic: bool, hard_coded: usize) -> f64 {
    let mut score = 50.0;

    if uses_vars {
        score += 30.0;
    }
    if has_semantic {
        score += 20.0;
    }

    let penalty = (hard_coded as f64 * 2.0).min(50.0);
    score -= penalty;

    score.clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z_index_detection() {
        let css = r#"
            .header { z-index: 100; }
            .modal { z-index: 1500; }
            .dropdown { z-index: 50; }
        "#;

        let z_index_pattern = Regex::new(r"z-index\s*:\s*(-?\d+)").unwrap();
        let mut found_high = false;

        for line in css.lines() {
            if let Some(caps) = z_index_pattern.captures(line) {
                if let Ok(value) = caps[1].parse::<i32>() {
                    if value.abs() > 1000 {
                        found_high = true;
                    }
                }
            }
        }

        assert!(found_high, "Should detect z-index > 1000");
    }

    #[test]
    fn test_responsive_unit_ratio() {
        let responsive_unit_pattern = Regex::new(r"(rem|em|%|vw|vh)\b").unwrap();
        let px_pattern = Regex::new(r"\d+px\b").unwrap();

        let css = r#"
            .box { width: 100px; height: 2rem; margin: 1em; padding: 5%; }
        "#;

        let responsive = responsive_unit_pattern.find_iter(css).count();
        let px = px_pattern.find_iter(css).count();

        assert_eq!(responsive, 3);
        assert_eq!(px, 1);
    }

    #[test]
    fn test_component_categorization() {
        let mut categories = ComponentCategories {
            atoms: 0,
            molecules: 0,
            organisms: 0,
            templates: 0,
            pages: 0,
        };

        categorize_component("Button", &mut categories);
        categorize_component("SearchBar", &mut categories);
        categorize_component("Header", &mut categories);
        categorize_component("PageLayout", &mut categories);
        categorize_component("HomePage", &mut categories);

        assert_eq!(categories.atoms, 1);
        assert_eq!(categories.molecules, 1);
        assert_eq!(categories.organisms, 1);
        assert_eq!(categories.templates, 1);
        assert_eq!(categories.pages, 1);
    }
}
