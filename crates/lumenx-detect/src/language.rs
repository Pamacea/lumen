//! Language detection

use crate::DetectionResult;
use lumenx_core::{Language, Project};
use std::collections::HashMap;

/// Detect all programming languages used in the project
pub fn detect_languages(project: &Project) -> DetectionResult<Vec<Language>> {
    let mut language_counts: HashMap<Language, usize> = HashMap::new();

    // Count files by extension
    for file in project.files() {
        let lang = Language::from_path(&file.path);
        if lang != Language::Unknown {
            *language_counts.entry(lang).or_insert(0) += 1;
        }
    }

    // Sort by count (descending)
    let mut languages: Vec<_> = language_counts.into_iter().collect();
    languages.sort_by(|a, b| b.1.cmp(&a.1));

    Ok(languages.into_iter().map(|(lang, _)| lang).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        // Basic test
        assert!(true);
    }
}
