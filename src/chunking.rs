use crate::types::{Chunk, ChunkType};
use std::fs;
use std::path::Path;
use tiktoken_rs::{cl100k_base, CoreBPE};

pub struct TextSplitter {
    pub chunk_size: usize,    // Taille maximale d'un chunk en tokens estimés
    pub chunk_overlap: usize, // Recouvrement entre chunks en tokens estimés
}

impl TextSplitter {
    pub fn split_file(&self, path: &Path) -> Result<Vec<Chunk>, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;

        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

        let enc = cl100k_base().map_err(|e| e.to_string())?;

        let chunks = match extension {
            "rs" => crate::intelligent_chunking::chunk_rust(&content, path, self.chunk_size, &enc)?,
            "js" | "jsx" => crate::intelligent_chunking::chunk_javascript(
                &content,
                path,
                self.chunk_size,
                &enc,
            )?,
            "ts" | "tsx" => {
                crate::intelligent_chunking::chunk_tsx(&content, path, self.chunk_size, &enc)?
            }
            "java" => {
                crate::intelligent_chunking::chunk_java(&content, path, self.chunk_size, &enc)?
            }
            _ => {
                // Fallback to line chunking for unsupported extensions
                return self.split_text(&content, path.to_string_lossy().to_string(), &enc);
            }
        };

        if chunks.is_empty() {
            // Fallback if tree-sitter parsing failed
            self.split_text(&content, path.to_string_lossy().to_string(), &enc)
        } else {
            Ok(chunks)
        }
    }

    pub fn split_text(
        &self,
        text: &str,
        path: String,
        enc: &CoreBPE,
    ) -> Result<Vec<Chunk>, String> {
        let units: Vec<String> = text.split('\n').map(|l| l.to_string()).collect();
        let token_counts: Vec<usize> = units.iter().map(|s| enc.encode_ordinary(s).len()).collect();

        let mut chunks = Vec::new();
        let mut chunk_index = 0;
        let mut start_index = 0;

        while start_index < units.len() {
            let mut end_index = start_index;

            let mut total_tokens: usize = 0;
            while end_index < units.len()
                && total_tokens + token_counts[end_index] <= self.chunk_size
            {
                total_tokens += token_counts[end_index];
                end_index += 1;
            }

            // Cas limite : l'unité seule dépasse chunk_size → on l'inclut quand même
            if start_index == end_index {
                end_index += 1;
            }

            // Fusion des unités sélectionnées en un chunk
            let chunk_text = units[start_index..end_index].join(" ");
            chunks.push(Chunk {
                chunk_index: chunk_index.to_string(),
                chunk_end_line: end_index,
                chunk_start_line: start_index,
                path: path.clone(),
                text: chunk_text,
                function_name: None,
                chunk_type: ChunkType::LineChunk,
            });

            chunk_index += 1;
            if end_index >= units.len() {
                break;
            }
            let mut overlap_tokens = 0;
            let mut backtrack = end_index;

            while backtrack > start_index {
                backtrack -= 1;
                overlap_tokens += token_counts[backtrack];
                if overlap_tokens >= self.chunk_overlap {
                    break;
                }
            }
            start_index = backtrack;
        }

        Ok(chunks)
    }
}
