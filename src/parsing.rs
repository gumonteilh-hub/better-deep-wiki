use ignore::Walk;
use std::path::Path;
use regex::Regex;
use crate::types::FilterConfig;

const CODE_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "tsx", "jsx", "java", "cpp", "c", "go", "rb", "php", "cs",
];
const DOC_EXTENSIONS: &[&str] = &["md", "markdown", "rst", "txt", "adoc", "org"];

#[derive(Debug, PartialEq)]
pub enum FileType {
    Code,
    Doc,
    Other,
}

#[derive(Debug)]
pub struct FileMeta {
    pub path: String,
}

fn detect_file_type(path: &Path) -> FileType {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        if CODE_EXTENSIONS.contains(&ext) {
            return FileType::Code;
        }
        if DOC_EXTENSIONS.contains(&ext) {
            return FileType::Doc;
        }
    }
    FileType::Other
}

fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    match Regex::new(pattern) {
        Ok(regex) => regex.is_match(path),
        Err(_) => {
            eprintln!("Warning: Invalid regex pattern '{}', skipping", pattern);
            false
        }
    }
}

fn should_include_file(path: &str, filter_config: &Option<FilterConfig>) -> bool {
    match filter_config {
        None => true,
        Some(config) => {
            let matches_any_pattern = config.paths.iter().any(|pattern| path_matches_pattern(path, pattern));
            
            match config.mode.as_str() {
                "include" => matches_any_pattern,
                "exclude" => !matches_any_pattern,
                _ => true, // Default to include if mode is unknown
            }
        }
    }
}

pub fn parse_repo(path: String, filter_config: Option<FilterConfig>) -> Vec<FileMeta> {
    let mut results = Vec::new();

    for entry in Walk::new(&path) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let entry_path = entry.path();
        let file_type = detect_file_type(entry_path);

        if file_type != FileType::Code {
            continue;
        }

        let path_str = entry_path.display().to_string();
        
        // Apply filter if configured
        if !should_include_file(&path_str, &filter_config) {
            continue;
        }

        results.push(FileMeta {
            path: path_str,
        });
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_patterns() {
        // Exact match
        assert!(path_matches_pattern("src/main.rs", "^src/main\\.rs$"));
        assert!(!path_matches_pattern("src/main.rs", "^src/lib\\.rs$"));

        // Extension patterns
        assert!(path_matches_pattern("main.rs", r"\.rs$"));
        assert!(path_matches_pattern("main.rs", r"\.rs"));
        assert!(path_matches_pattern("src/main.rs", ".rs"));
        assert!(!path_matches_pattern("main.py", r"\.rs$"));

        // Directory patterns
        assert!(path_matches_pattern("src/main.rs", "^src/"));
        assert!(path_matches_pattern("src/main.rs", "src/"));
        assert!(path_matches_pattern("src/main.rs", "src"));
        assert!(path_matches_pattern("src/utils/mod.rs", "^src/"));
        assert!(!path_matches_pattern("tests/main.rs", "^src/"));

        // Complex patterns
        assert!(path_matches_pattern("src/test_utils.rs", "test"));
        assert!(path_matches_pattern("test_main.rs", "test"));
        assert!(path_matches_pattern("test_main.rs", "test*"));
        assert!(!path_matches_pattern("test_main.rs", "*test"));
        assert!(!path_matches_pattern("src/main.rs", "test"));
    }

    #[test]
    fn test_filter_config_exclude() {
        let filter_config = Some(FilterConfig {
            mode: "exclude".to_string(),
            paths: vec!["node_modules".to_string(), r"\.test\.js$".to_string()],
        });

        assert!(!should_include_file("node_modules/package.json", &filter_config));
        assert!(!should_include_file("src/main.test.js", &filter_config));
        assert!(should_include_file("src/main.js", &filter_config));
    }

    #[test]
    fn test_filter_config_include() {
        let filter_config = Some(FilterConfig {
            mode: "include".to_string(),
            paths: vec!["^src/".to_string(), r"\.rs$".to_string()],
        });

        assert!(should_include_file("src/main.js", &filter_config));
        assert!(should_include_file("main.rs", &filter_config));
        assert!(!should_include_file("docs/readme.md", &filter_config));
    }
}
