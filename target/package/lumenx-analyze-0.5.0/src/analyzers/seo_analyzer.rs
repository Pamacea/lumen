//! SEO analyzer
//!
//! Analyzes web projects for SEO best practices including:
//! - Meta tags coverage (title, description, OG tags)
//! - Structured data (JSON-LD, Schema.org)
//! - Sitemap and robots.txt
//! - Heading structure (H1-H6 hierarchy)
//! - Image alt text
//! - Canonical URLs
//! - Semantic HTML usage

use crate::parsers::html::{HtmlParser, MetaTag, ScriptTag, LinkTag};
use lumenx_core::{LumenResult, Project, ProjectInfo, Framework, Language};
use lumenx_score::{IssueSeverity, MetricValue, ScoreIssue};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

/// SEO analyzer
pub struct SeoAnalyzer {
    project_root: String,
}

impl SeoAnalyzer {
    /// Create a new SEO analyzer
    pub fn new(project_root: String) -> Self {
        Self { project_root }
    }

    /// Analyze the project for SEO issues
    pub fn analyze(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Framework-specific SEO analysis
        let seo_files = self.find_seo_files(info)?;

        // Analyze meta tags
        let (meta_metrics, meta_issues) = self.analyze_meta_tags(&seo_files)?;
        metrics.extend(meta_metrics);
        issues.extend(meta_issues);

        // Analyze structured data
        let (structured_metrics, structured_issues) = self.analyze_structured_data(&seo_files)?;
        metrics.extend(structured_metrics);
        issues.extend(structured_issues);

        // Check for sitemap and robots.txt
        let (infra_metrics, infra_issues) = self.analyze_seo_infrastructure()?;
        metrics.extend(infra_metrics);
        issues.extend(infra_issues);

        // Analyze heading structure
        let (heading_metrics, heading_issues) = self.analyze_heading_structure(&seo_files)?;
        metrics.extend(heading_metrics);
        issues.extend(heading_issues);

        // Analyze image alt text
        let (image_metrics, image_issues) = self.analyze_image_alt_text(&seo_files)?;
        metrics.extend(image_metrics);
        issues.extend(image_issues);

        // Analyze semantic HTML
        let (semantic_metrics, semantic_issues) = self.analyze_semantic_html(&seo_files)?;
        metrics.extend(semantic_metrics);
        issues.extend(semantic_issues);

        // Analyze URL structure
        let (url_metrics, url_issues) = self.analyze_url_structure(info)?;
        metrics.extend(url_metrics);
        issues.extend(url_issues);

        // Calculate overall SEO score
        let base_score = 100.0;
        let penalty: f64 = issues.iter().map(|i| match i.severity {
            IssueSeverity::Critical => 15.0,
            IssueSeverity::High => 8.0,
            IssueSeverity::Medium => 4.0,
            IssueSeverity::Low => 1.0,
            IssueSeverity::Info => 0.0,
        }).sum();

        metrics.insert(
            "seo:overall_score".to_string(),
            MetricValue::Percentage((base_score - penalty).max(0.0)),
        );

        Ok((metrics, issues))
    }

    /// Find files relevant for SEO analysis
    fn find_seo_files(&self, info: &ProjectInfo) -> LumenResult<Vec<SeoFile>> {
        let mut files = Vec::new();

        let root = Path::new(&self.project_root);

        // Framework-specific file locations
        let search_dirs = match info.framework {
            Framework::NextJs => vec![
                root.join("app"),
                root.join("pages"),
                root.join("src/app"),
                root.join("src/pages"),
            ],
            Framework::Remix => vec![
                root.join("app"),
                root.join("src/app"),
            ],
            Framework::Astro => vec![
                root.join("src/pages"),
            ],
            Framework::Nuxt => vec![
                root.join("pages"),
                root.join("app"),
                root.join("src/pages"),
            ],
            Framework::SvelteKit => vec![
                root.join("src/routes"),
            ],
            _ => vec![
                root.join("src"),
                root.join("app"),
                root.join("pages"),
            ],
        };

        let extensions = ["tsx", "jsx", "ts", "js", "astro", "vue", "svelte", "html"];

        for search_dir in &search_dirs {
            if search_dir.exists() {
                for entry in WalkDir::new(search_dir)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if extensions.contains(&ext.to_str().unwrap_or("")) {
                                if let Ok(content) = std::fs::read_to_string(path) {
                                    // Check if file contains HTML/JSX patterns
                                    if content.contains('<') || content.contains("export default") {
                                        let file_type = Self::detect_file_type(&content);
                                        files.push(SeoFile {
                                            path: path.to_path_buf(),
                                            content,
                                            file_type,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Detect file type based on content
    fn detect_file_type(content: &str) -> SeoFileType {
        if content.contains("<!DOCTYPE html") || content.contains("<html") {
            SeoFileType::Html
        } else if content.contains("export default") || content.contains("export const") {
            SeoFileType::Component
        } else {
            SeoFileType::Unknown
        }
    }

    /// Analyze meta tags coverage
    fn analyze_meta_tags(&self, files: &[SeoFile]) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let parser = HtmlParser::new()?;

        let mut has_title = 0;
        let mut has_description = 0;
        let mut has_charset = 0;
        let mut has_viewport = 0;
        let mut has_og_title = 0;
        let mut has_og_description = 0;
        let mut has_og_image = 0;
        let mut has_twitter_card = 0;
        let mut has_canonical = 0;
        let mut total_pages = files.len();

        for file in files {
            let metas = parser.extract_meta_tags(&file.content);
            let links = parser.extract_links(&file.content);

            // Check for title tag
            if file.content.contains("<title>") {
                has_title += 1;
            } else if file.file_type == SeoFileType::Component {
                // Check for Next.js metadata API
                if file.content.contains("metadata") || file.content.contains("generateMetadata") {
                    has_title += 1;
                }
            }

            // Check for meta tags
            for meta in &metas {
                if let Some(name) = &meta.name {
                    match name.as_str() {
                        "description" => has_description += 1,
                        "viewport" => has_viewport += 1,
                        "og:title" => has_og_title += 1,
                        "og:description" => has_og_description += 1,
                        "og:image" => has_og_image += 1,
                        "twitter:card" => has_twitter_card += 1,
                        _ => {}
                    }
                }
                if meta.charset.is_some() {
                    has_charset += 1;
                }
            }

            // Check for canonical link
            for link in &links {
                if let Some(rel) = &link.rel {
                    if rel == "canonical" {
                        has_canonical += 1;
                    }
                }
            }
        }

        // Calculate coverage percentages
        let title_coverage = if total_pages > 0 {
            (has_title as f64 / total_pages as f64) * 100.0
        } else {
            0.0
        };
        let description_coverage = if total_pages > 0 {
            (has_description as f64 / total_pages as f64) * 100.0
        } else {
            0.0
        };
        let og_coverage = if total_pages > 0 {
            (has_og_title as f64 / total_pages as f64) * 100.0
        } else {
            0.0
        };

        metrics.insert(
            "seo:title_coverage".to_string(),
            MetricValue::Percentage(title_coverage),
        );
        metrics.insert(
            "seo:description_coverage".to_string(),
            MetricValue::Percentage(description_coverage),
        );
        metrics.insert(
            "seo:og_tags_coverage".to_string(),
            MetricValue::Percentage(og_coverage),
        );
        metrics.insert(
            "seo:has_canonical_links".to_string(),
            MetricValue::Count(has_canonical),
        );

        // Generate issues
        if title_coverage < 100.0 && total_pages > 0 {
            issues.push(ScoreIssue {
                id: "seo-001".to_string(),
                severity: IssueSeverity::Critical,
                category: "seo".to_string(),
                title: "Missing title tags".to_string(),
                description: format!("Only {:.0}% of pages have title tags. Title tags are crucial for SEO.", title_coverage),
                file: None,
                line: None,
                column: None,
                impact: 15.0,
                suggestion: Some("Add title tags to all pages:\n\n```tsx\n// Next.js App Router\nexport const metadata = {\n  title: 'Page Title - Site Name',\n}\n\n// Or in layout:\nexport default function Layout({ children }) {\n  return (\n    <html>\n      <head>\n        <title>Page Title - Site Name</title>\n      </head>\n      <body>{children}</body>\n    </html>\n  )\n}\n```".to_string()),
            });
        }

        if description_coverage < 80.0 && total_pages > 0 {
            issues.push(ScoreIssue {
                id: "seo-002".to_string(),
                severity: IssueSeverity::High,
                category: "seo".to_string(),
                title: "Missing meta descriptions".to_string(),
                description: format!("Only {:.0}% of pages have meta descriptions. Meta descriptions appear in search results and affect click-through rates.", description_coverage),
                file: None,
                line: None,
                column: None,
                impact: 8.0,
                suggestion: Some("Add meta descriptions (150-160 characters):\n\n```tsx\nexport const metadata = {\n  description: 'A compelling description of your page content that entices users to click.',\n}\n\n// Or HTML:\n<meta name=\"description\" content=\"Your description here\" />\n```".to_string()),
            });
        }

        if og_coverage < 50.0 && total_pages > 0 {
            issues.push(ScoreIssue {
                id: "seo-003".to_string(),
                severity: IssueSeverity::Medium,
                category: "seo".to_string(),
                title: "Missing Open Graph tags".to_string(),
                description: format!("Only {:.0}% of pages have Open Graph tags. OG tags control how content appears when shared on social media.", og_coverage),
                file: None,
                line: None,
                column: None,
                impact: 4.0,
                suggestion: Some("Add Open Graph tags:\n\n```tsx\nexport const metadata = {\n  openGraph: {\n    title: 'Page Title',\n    description: 'Page description',\n    url: 'https://example.com/page',\n    siteName: 'Site Name',\n    images: [{\n      url: 'https://example.com/image.jpg',\n      width: 1200,\n      height: 630,\n    }],\n    type: 'website',\n  },\n}\n```".to_string()),
            });
        }

        if has_canonical == 0 && total_pages > 0 {
            issues.push(ScoreIssue {
                id: "seo-004".to_string(),
                severity: IssueSeverity::Medium,
                category: "seo".to_string(),
                title: "No canonical URLs found".to_string(),
                description: "Canonical URLs help prevent duplicate content issues by specifying the preferred version of a page.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 4.0,
                suggestion: Some("Add canonical links:\n\n```tsx\nexport const metadata = {\n  alternates: {\n    canonical: 'https://example.com/page',\n  },\n}\n\n// Or HTML:\n<link rel=\"canonical\" href=\"https://example.com/page\" />\n```".to_string()),
            });
        }

        if has_viewport < total_pages {
            issues.push(ScoreIssue {
                id: "seo-005".to_string(),
                severity: IssueSeverity::Low,
                category: "seo".to_string(),
                title: "Missing viewport meta tag".to_string(),
                description: "Viewport meta tag is essential for mobile-friendly design, which is a ranking factor.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 2.0,
                suggestion: Some("Add viewport meta tag:\n\n```html\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Analyze structured data (JSON-LD, Schema.org)
    fn analyze_structured_data(&self, files: &[SeoFile]) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let mut has_json_ld = 0;
        let mut has_microdata = 0;
        let mut schema_types = HashSet::new();

        for file in files {
            // Check for JSON-LD
            if file.content.contains("application/ld+json") || file.content.contains(r#""@context""#) {
                has_json_ld += 1;

                // Extract schema types
                let json_ld_regex = Regex::new(r#""@type"\s*:\s*"([^"]+)""#).unwrap();
                for cap in json_ld_regex.captures_iter(&file.content) {
                    if let Some(schema_type) = cap.get(1) {
                        schema_types.insert(schema_type.as_str().to_string());
                    }
                }
            }

            // Check for Microdata
            if file.content.contains("itemscope") || file.content.contains("itemtype") {
                has_microdata += 1;
            }
        }

        let structured_data_score = if !files.is_empty() {
            ((has_json_ld + has_microdata) as f64 / files.len() as f64) * 100.0
        } else {
            0.0
        };

        metrics.insert(
            "seo:structured_data_score".to_string(),
            MetricValue::Percentage(structured_data_score),
        );
        metrics.insert(
            "seo:json_ld_count".to_string(),
            MetricValue::Count(has_json_ld),
        );
        metrics.insert(
            "seo:schema_types_found".to_string(),
            MetricValue::Count(schema_types.len()),
        );

        if has_json_ld == 0 && !files.is_empty() {
            issues.push(ScoreIssue {
                id: "seo-006".to_string(),
                severity: IssueSeverity::Medium,
                category: "seo".to_string(),
                title: "No structured data found".to_string(),
                description: "Structured data helps search engines understand your content and can lead to rich snippets in search results.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 5.0,
                suggestion: Some("Add JSON-LD structured data:\n\n```tsx\nexport default function Page() {\n  const jsonLd = {\n    '@context': 'https://schema.org',\n    '@type': 'Article',\n    headline: 'Article Title',\n    author: {\n      '@type': 'Person',\n      name: 'Author Name',\n    },\n    datePublished: '2024-01-01',\n  };\n\n  return (\n    <>\n      <script\n        type=\"application/ld+json\"\n        dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}\n      />\n      {/* ... */}\n    </>\n  );\n}\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Analyze SEO infrastructure (sitemap, robots.txt)
    fn analyze_seo_infrastructure(&self) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let root = Path::new(&self.project_root);

        // Check for robots.txt
        let robots_path = root.join("public").join("robots.txt");
        let robots_root = root.join("robots.txt");
        let has_robots = robots_path.exists() || robots_root.exists();

        metrics.insert(
            "seo:has_robots_txt".to_string(),
            MetricValue::Boolean(has_robots),
        );

        if !has_robots {
            issues.push(ScoreIssue {
                id: "seo-007".to_string(),
                severity: IssueSeverity::Medium,
                category: "seo".to_string(),
                title: "Missing robots.txt file".to_string(),
                description: "robots.txt tells search crawlers which pages they can access. This is essential for proper crawling.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 4.0,
                suggestion: Some("Create a robots.txt file in your public directory:\n\n```\n# Allow all crawlers\nUser-agent: *\nAllow: /\n\n# Disallow specific paths\nDisallow: /api/\nDisallow: /admin/\n\n# Sitemap location\nSitemap: https://example.com/sitemap.xml\n```".to_string()),
            });
        }

        // Check for sitemap
        let sitemap_path = root.join("public").join("sitemap.xml");
        let sitemap_root = root.join("sitemap.xml");
        let has_sitemap = sitemap_path.exists() || sitemap_root.exists();

        metrics.insert(
            "seo:has_sitemap".to_string(),
            MetricValue::Boolean(has_sitemap),
        );

        if !has_sitemap {
            issues.push(ScoreIssue {
                id: "seo-008".to_string(),
                severity: IssueSeverity::High,
                category: "seo".to_string(),
                title: "Missing sitemap.xml file".to_string(),
                description: "Sitemaps help search engines discover and index your pages efficiently.".to_string(),
                file: None,
                line: None,
                column: None,
                impact: 6.0,
                suggestion: Some("Generate a sitemap.xml file:\n\n```tsx\n// app/sitemap.ts (Next.js)\nimport { MetadataRoute } from 'next'\n\nexport default function sitemap(): MetadataRoute.Sitemap {\n  return [\n    {\n      url: 'https://example.com',\n      lastModified: new Date(),\n    },\n    {\n      url: 'https://example.com/about',\n      lastModified: new Date(),\n    },\n    // ... more URLs\n  ]\n}\n\n// Or manually:\n// public/sitemap.xml\n<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n  <url>\n    <loc>https://example.com/</loc>\n    <lastmod>2024-01-01</lastmod>\n  </url>\n</urlset>\n```".to_string()),
            });
        }

        // Check for favicon
        let favicon_locations = [
            root.join("public").join("favicon.ico"),
            root.join("public").join("icon.png"),
            root.join("public").join("apple-icon.png"),
        ];
        let has_favicon = favicon_locations.iter().any(|p| p.exists());

        metrics.insert(
            "seo:has_favicon".to_string(),
            MetricValue::Boolean(has_favicon),
        );

        // Check for manifest.json (PWA)
        let manifest_path = root.join("public").join("manifest.json");
        let has_manifest = manifest_path.exists();

        metrics.insert(
            "seo:has_manifest".to_string(),
            MetricValue::Boolean(has_manifest),
        );

        Ok((metrics, issues))
    }

    /// Analyze heading structure
    fn analyze_heading_structure(&self, files: &[SeoFile]) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let heading_regex = Regex::new(r"<([hH][1-6])[^>]*>([^<]*)</[hH][1-6]>").unwrap();

        let mut files_with_h1 = 0;
        let mut multiple_h1_files = 0;
        let mut total_heading_issues = 0;

        for file in files {
            let mut h1_count = 0;
            let mut last_heading_level = 0;
            let mut headings_in_file = Vec::new();

            for cap in heading_regex.captures_iter(&file.content) {
                if let (Some(level_tag), Some(text)) = (cap.get(1), cap.get(2)) {
                    let level = level_tag.as_str()[1..2].parse::<usize>().unwrap_or(0);
                    headings_in_file.push((level, text.as_str().to_string()));

                    if level == 1 {
                        h1_count += 1;
                    }
                }
            }

            // Check for multiple H1s
            if h1_count > 1 {
                multiple_h1_files += 1;
                total_heading_issues += 1;
            }

            // Check for heading hierarchy violations
            for (level, _) in &headings_in_file {
                if *level > last_heading_level + 1 && last_heading_level > 0 {
                    total_heading_issues += 1;
                    break;
                }
                last_heading_level = *level;
            }

            if h1_count >= 1 {
                files_with_h1 += 1;
            }
        }

        let h1_coverage = if !files.is_empty() {
            (files_with_h1 as f64 / files.len() as f64) * 100.0
        } else {
            0.0
        };

        metrics.insert(
            "seo:h1_coverage".to_string(),
            MetricValue::Percentage(h1_coverage),
        );
        metrics.insert(
            "seo:multiple_h1_pages".to_string(),
            MetricValue::Count(multiple_h1_files),
        );

        if multiple_h1_files > 0 {
            issues.push(ScoreIssue {
                id: "seo-009".to_string(),
                severity: IssueSeverity::Medium,
                category: "seo".to_string(),
                title: "Multiple H1 tags on same page".to_string(),
                description: format!("Found {} pages with multiple H1 tags. Each page should have exactly one H1 tag for proper heading hierarchy.", multiple_h1_files),
                file: None,
                line: None,
                column: None,
                impact: 4.0,
                suggestion: Some("Ensure each page has exactly one H1 tag:\n\n```tsx\n// Good:\n<h1>Main Page Title</h1>\n<h2>Section</h2>\n<h3>Subsection</h3>\n\n// Bad:\n<h1>Title 1</h1>\n<h1>Title 2</h1>\n\n// Convert the second H1 to H2\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Analyze image alt text
    fn analyze_image_alt_text(&self, files: &[SeoFile]) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let img_regex = Regex::new(r#"<img[^>]*?(?:alt\s*=\s*["']([^"']*)["'])?[^>]*?>"#).unwrap();

        let mut total_images = 0;
        let mut images_without_alt = 0;
        let mut images_with_empty_alt = 0;

        for file in files {
            for cap in img_regex.captures_iter(&file.content) {
                total_images += 1;

                let has_alt = cap.get(1).is_some();
                if !has_alt {
                    images_without_alt += 1;
                } else if cap.get(1).map(|m| m.as_str().is_empty()).unwrap_or(false) {
                    images_with_empty_alt += 1;
                }
            }
        }

        let alt_coverage = if total_images > 0 {
            ((total_images - images_without_alt) as f64 / total_images as f64) * 100.0
        } else {
            100.0
        };

        metrics.insert(
            "seo:image_alt_coverage".to_string(),
            MetricValue::Percentage(alt_coverage),
        );
        metrics.insert(
            "seo:images_without_alt".to_string(),
            MetricValue::Count(images_without_alt),
        );

        if images_without_alt > 5 {
            issues.push(ScoreIssue {
                id: "seo-010".to_string(),
                severity: IssueSeverity::Medium,
                category: "seo".to_string(),
                title: "Missing image alt text".to_string(),
                description: format!("Found {} images without alt text. Alt text improves accessibility and SEO.", images_without_alt),
                file: None,
                line: None,
                column: None,
                impact: 4.0,
                suggestion: Some("Add descriptive alt text to images:\n\n```tsx\n// Good:\n<img src=\"sunset.jpg\" alt=\"Orange sunset over the ocean with a lighthouse in the foreground\" />\n\n// Decorative images (empty alt is OK):\n<img src=\"decorative-pattern.jpg\" alt=\"\" />\n\n// Bad:\n<img src=\"sunset.jpg\" /> // Missing alt entirely\n<img src=\"sunset.jpg\" alt=\"image1\" /> // Non-descriptive\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Analyze semantic HTML usage
    fn analyze_semantic_html(&self, files: &[SeoFile]) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        let semantic_tags = [
            ("header", r"<header[^>]*>"),
            ("nav", r"<nav[^>]*>"),
            ("main", r"<main[^>]*>"),
            ("article", r"<article[^>]*>"),
            ("section", r"<section[^>]*>"),
            ("aside", r"<aside[^>]*>"),
            ("footer", r"<footer[^>]*>"),
        ];

        let mut semantic_tag_counts: HashMap<&str, usize> = HashMap::new();
        let mut total_divs = 0;

        for file in files {
            for (tag, pattern) in &semantic_tags {
                let regex = Regex::new(pattern).unwrap();
                let count = regex.find_iter(&file.content).count();
                *semantic_tag_counts.entry(tag).or_insert(0) += count;
            }

            let div_regex = Regex::new(r"<div[^>]*>").unwrap();
            total_divs += div_regex.find_iter(&file.content).count();
        }

        let total_semantic: usize = semantic_tag_counts.values().sum();
        let semantic_ratio = if total_divs > 0 {
            (total_semantic as f64 / (total_semantic as f64 + total_divs as f64)) * 100.0
        } else if total_semantic > 0 {
            100.0
        } else {
            0.0
        };

        metrics.insert(
            "seo:semantic_html_ratio".to_string(),
            MetricValue::Percentage(semantic_ratio),
        );

        for (tag, count) in &semantic_tag_counts {
            metrics.insert(
                format!("seo:{} count", tag),
                MetricValue::Count(*count),
            );
        }

        if semantic_ratio < 30.0 {
            issues.push(ScoreIssue {
                id: "seo-011".to_string(),
                severity: IssueSeverity::Low,
                category: "seo".to_string(),
                title: "Low semantic HTML usage".to_string(),
                description: format!("Only {:.0}% of layout elements use semantic tags. Semantic HTML helps search engines understand content structure.", semantic_ratio),
                file: None,
                line: None,
                column: None,
                impact: 2.0,
                suggestion: Some("Use semantic HTML elements:\n\n```tsx\n// Instead of:\n<div class=\"header\">...</div>\n<div class=\"nav\">...</div>\n<div class=\"main\">\n  <div class=\"article\">...</div>\n</div>\n<div class=\"footer\">...</div>\n\n// Use:\n<header>...</header>\n<nav>...</nav>\n<main>\n  <article>...</article>\n</main>\n<footer>...</footer>\n```".to_string()),
            });
        }

        Ok((metrics, issues))
    }

    /// Analyze URL structure
    fn analyze_url_structure(&self, info: &ProjectInfo) -> LumenResult<(HashMap<String, MetricValue>, Vec<ScoreIssue>)> {
        let mut metrics = HashMap::new();
        let mut issues = Vec::new();

        // Check for clean URLs (no .html, .php extensions in routes)
        let root = Path::new(&self.project_root);
        let mut clean_urls = true;

        // Check framework-specific route patterns
        match info.framework {
            Framework::NextJs | Framework::Remix => {
                // These frameworks support clean URLs by default
                metrics.insert(
                    "seo:clean_urls".to_string(),
                    MetricValue::Boolean(true),
                );
            }
            _ => {
                // For other frameworks, check file extensions
                for entry in WalkDir::new(root)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension() {
                            if ext == "html" || ext == "php" || ext == "asp" {
                                clean_urls = false;
                                break;
                            }
                        }
                    }
                }

                metrics.insert(
                    "seo:clean_urls".to_string(),
                    MetricValue::Boolean(clean_urls),
                );
            }
        }

        // Check for trailing slash consistency
        // This would require checking the framework config or router setup

        Ok((metrics, issues))
    }
}

/// SEO file with content
#[derive(Debug, Clone)]
struct SeoFile {
    path: std::path::PathBuf,
    content: String,
    file_type: SeoFileType,
}

/// SEO file type
#[derive(Debug, Clone, PartialEq)]
enum SeoFileType {
    Html,
    Component,
    Unknown,
}

/// Public function for backward compatibility with the module interface
pub fn analyze(project: &lumenx_core::Project) -> Vec<ScoreIssue> {
    let analyzer = SeoAnalyzer::new(project.info.root.to_string_lossy().to_string());
    let (_, issues) = analyzer.analyze(&project.info).unwrap_or_default();
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seo_analyzer_creation() {
        let analyzer = SeoAnalyzer::new("/tmp".to_string());
        assert_eq!(analyzer.project_root, "/tmp");
    }

    #[test]
    fn test_detect_file_type() {
        let html = SeoAnalyzer::detect_file_type("<!DOCTYPE html><html>...");
        assert_eq!(html, SeoFileType::Html);

        let component = SeoAnalyzer::detect_file_type("export default function Page() {");
        assert_eq!(component, SeoFileType::Component);
    }

    #[test]
    fn test_analyze_meta_tags() {
        let files = vec![
            SeoFile {
                path: std::path::PathBuf::from("/tmp/test.html"),
                content: "<html><head><title>Test</title><meta name=\"description\" content=\"Test page\"></head></html>".to_string(),
                file_type: SeoFileType::Html,
            },
        ];

        let analyzer = SeoAnalyzer::new("/tmp".to_string());
        let (metrics, _issues) = analyzer.analyze_meta_tags(&files).unwrap();

        assert_eq!(metrics.get("seo:title_coverage").unwrap().as_percentage(), Some(100.0));
        assert_eq!(metrics.get("seo:description_coverage").unwrap().as_percentage(), Some(100.0));
    }
}
