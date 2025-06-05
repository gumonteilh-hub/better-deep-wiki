use ignore::Walk;
use std::fs;
use std::path::Path;

const CODE_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "java", "cpp", "c", "go", "rb", "php", "cs",
];
const DOC_EXTENSIONS: &[&str] = &["md", "markdown", "rst", "txt", "adoc", "org"];

#[derive(Debug)]
pub enum FileType {
    Code,
    Doc,
    Other,
}

#[derive(Debug)]
pub struct FileMeta {
    pub path: String,
    pub file_type: FileType,
    pub token_count: usize,
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

fn estimate_token_count(text: &str) -> usize {
    // TODO: Remplace par tiktoken-rs si besoin de précision
    text.split_whitespace().count()
}

pub fn parse_repo(path: String) -> Vec<FileMeta> {
    let mut results = Vec::new();

    for entry in Walk::new(path) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        // Tente de lire le fichier en texte (skip binaires)
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // 2) Type
        let file_type = detect_file_type(path);

        // 3) A-t-on envie de process ce type ?
        if let FileType::Other = file_type {
            continue;
        }

        // 4) Nombre de tokens
        let token_count = estimate_token_count(&content);

        // 5) Stockage
        results.push(FileMeta {
            path: path.display().to_string(),
            file_type,
            token_count,
        });
    }
    results

}
