use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub vector_dimension: usize,
    pub top_k: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            chunk_size: 350,
            chunk_overlap: 100,
            vector_dimension: 1536,
            top_k: 10,
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        let default = Self::default();
        
        Self {
            chunk_size: env::var("CHUNK_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default.chunk_size),
            
            chunk_overlap: env::var("CHUNK_OVERLAP")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default.chunk_overlap),
            
            vector_dimension: env::var("VECTOR_DIMENSION")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default.vector_dimension),
            
            top_k: env::var("TOP_K")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default.top_k),
        }
    }
}