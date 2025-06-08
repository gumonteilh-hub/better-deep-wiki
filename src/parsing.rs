use ignore::Walk;
use std::path::Path;

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

pub fn parse_repo(path: String) -> Vec<FileMeta> {
    let mut results = Vec::new();

    for entry in Walk::new(path) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        let file_type = detect_file_type(path);

        if file_type != FileType::Code {
            continue;
        }

        results.push(FileMeta {
            path: path.display().to_string(),
        });
    }
    results
}
