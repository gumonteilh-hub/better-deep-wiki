use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Encode, Decode, Deserialize, Serialize, Debug, Clone)]
pub struct Chunk {
    pub path: String,
    pub chunk_index: usize,
    pub chunk_start_line: usize,
    pub chunk_end_line: usize,
    pub text: String,
}

#[derive(Debug)]
pub struct Embedding {
    pub id: String, // UUID
    pub path: String,
    pub chunk_index: usize,
    pub chunk_start_line: usize,
    pub chunk_end_line: usize,
    pub vector: Vec<f32>,
}

impl Embedding {
    pub fn new(
        path: String,
        chunk_index: usize,
        chunk_start_line: usize,
        chunk_end_line: usize,
        vector: Vec<f32>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            path,
            chunk_index,
            chunk_start_line,
            chunk_end_line,
            vector,
        }
    }
}