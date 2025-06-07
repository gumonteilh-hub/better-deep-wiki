use std::fs;
use std::path::Path;
use tiktoken_rs::cl100k_base;
use crate::utils;
use crate::types::Chunk;

// Méthode de découpe disponible
pub enum SplitBy {
    // Word,     // Découpe par mot
    // Sentence, // Découpe par phrase (basé sur ".")
    Line,     // Découpe par ligne ("\n")
    // Passage,  // Découpe par paragraphe ("\n\n")
}

// Structure de configuration du splitter
pub struct TextSplitter {
    pub split_by: SplitBy,    // Méthode de découpe
    pub chunk_size: usize,    // Taille maximale d'un chunk en tokens estimés
    pub chunk_overlap: usize, // Recouvrement entre chunks en tokens estimés
}

impl TextSplitter {
    // Méthode pour traiter un fichier sur disque
    pub fn split_file(&self, path: &Path) -> Result<Vec<Chunk>, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        self.split_text(&content, path.to_string_lossy().to_string())
    }

    // Découpe un texte brut en chunks
    pub fn split_text(&self, text: &str, path: String) -> Result<Vec<Chunk>, String> {
        // Étape 1 : découpe naïve du texte en unités selon split_by
        let units: Vec<String> = match self.split_by {
            // SplitBy::Word => text.split_whitespace().map(|s| s.to_string()).collect(),
            SplitBy::Line => text.split('\n').map(|l| l.to_string()).collect(),
            // SplitBy::Sentence => text.split('.').map(|s| format!("{}.", s.trim())).collect(),
            // SplitBy::Passage => text.split("\n\n").map(|s| s.to_string()).collect(),
        };

        // Initialisation du tokenizer GPT (cl100k_base)
        let enc = cl100k_base().map_err(|e| e.to_string())?;

        // Étape 2 : pour chaque unité, estimer le nombre de tokens
        let token_counts: Vec<usize> = units.iter().map(|s| enc.encode_ordinary(s).len()).collect();

        // Étape 3 : sliding window pour créer les chunks avec overlap
        let mut chunks = Vec::new();
        let mut chunk_index = 0;

        let mut start_index = 0;

        while start_index < units.len() {
            let mut end_index = start_index;

            // Avancer tant que le chunk reste sous la limite de tokens
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
                chunk_index: chunk_index,
                chunk_end_line: end_index,
                chunk_start_line: start_index,
                path: path.clone(),
                text: utils::prepare_chunk(&path, chunk_index, &chunk_text),
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_empty_input() {
//         let splitter = TextSplitter {
//             split_by: SplitBy::Line,
//             chunk_size: 10,
//             chunk_overlap: 2,
//         };
//         let chunks = splitter.split_text("", "".into()).unwrap();
//         assert!(chunks.is_empty());
//     }

//     #[test]
//     fn test_single_unit_exceeds_chunk_size() {
//         let long_word = "a".repeat(1000);
//         let splitter = TextSplitter {
//             split_by: SplitBy::Line,
//             chunk_size: 1,
//             chunk_overlap: 0,
//         };
//         let chunks = splitter.split_text(&long_word, "".into()).unwrap();
//         assert_eq!(chunks.len(), 1);
//     }

//     #[test]
//     fn test_exact_fit_no_overlap() {
//         let text = "one two three four";
//         let splitter = TextSplitter {
//             split_by: SplitBy::Word,
//             chunk_size: 10,
//             chunk_overlap: 0,
//         };
//         let chunks = splitter.split_text(text).unwrap();
//         assert_eq!(chunks.len(), 1);
//     }

//     #[test]
//     fn test_overlap_behavior() {
//         let text = "a b c d e f g h i j k";
//         let splitter = TextSplitter {
//             split_by: SplitBy::Word,
//             chunk_size: 4,
//             chunk_overlap: 2,
//         };
//         let chunks = splitter.split_text(text).unwrap();
//         assert!(chunks.len() > 1);
//     }

//     #[test]
//     fn test_passage_split() {
//         let text = "para1\n\npara2\n\npara3";
//         let splitter = TextSplitter {
//             split_by: SplitBy::Line,
//             chunk_size: 100,
//             chunk_overlap: 10,
//         };
//         let chunks = splitter.split_text(text, "".into()).unwrap();
//         assert_eq!(chunks.len(), 1); // All 3 should fit in one chunk
//     }

//     #[test]
//     fn test_sentence_split() {
//         let text = "This is one. This is two. This is three.";
//         let splitter = TextSplitter {
//             split_by: SplitBy::Line,
//             chunk_size: 10,
//             chunk_overlap: 2,
//         };
//         let chunks = splitter.split_text(text, "").unwrap();
//         assert!(chunks.len() >= 1);
//     }
// }
