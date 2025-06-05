use reqwest::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::fmt;

use tiktoken_rs::cl100k_base;

const MAX_BATCH_SIZE: usize = 128;
const MAX_TOTAL_TOKENS: usize = 16384;
const MAX_SEQUENCE_LENGTH: usize = 8192;

pub type Embedding = Vec<f32>;

#[derive(Debug)]
pub struct EmbedResult {
    pub input: Vec<String>,
    pub embeddings: Option<Vec<Embedding>>,
    pub error: Option<String>,
}

impl fmt::Display for EmbedResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(err) = &self.error {
            write!(f, "EmbedResult: ERREUR: {err}")
        } else {
            write!(
                f,
                "EmbedResult: {} textes, {} embeddings",
                self.input.len(),
                self.embeddings.as_ref().map(|e| e.len()).unwrap_or(0)
            )
        }
    }
}

fn truncate_to_max_tokens(text: &str, max_tokens: usize) -> String {
    let enc = cl100k_base().expect("error instantiation tiktoken_rs");
    let tokens = enc.encode_ordinary(text);
    if tokens.len() <= max_tokens {
        text.to_string()
    } else {
        let truncated = enc
            .decode(tokens[..max_tokens].to_vec())
            .unwrap_or_default();
        truncated
    }
}

fn make_batches(texts: &[String]) -> Vec<Vec<String>> {
    let mut batches = Vec::new();
    let mut current_batch = Vec::new();
    let mut current_batch_tokens = 0;
    let enc = cl100k_base().expect("error instantiation tiktoken_rs");
    for text in texts {
        let tokens = enc.encode_ordinary(text);
        let t = if tokens.len() > MAX_SEQUENCE_LENGTH {
            enc.decode(tokens[..MAX_SEQUENCE_LENGTH].to_vec())
                .unwrap_or_default()
        } else {
            text.clone()
        };
        let n_tokens = enc.encode_ordinary(&t).len();
        if current_batch.len() >= MAX_BATCH_SIZE
            || current_batch_tokens + n_tokens > MAX_TOTAL_TOKENS
        {
            if !current_batch.is_empty() {
                batches.push(current_batch);
                current_batch = Vec::new();
                current_batch_tokens = 0;
            }
        }
        current_batch.push(t);
        current_batch_tokens += n_tokens;
    }
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }
    batches
}

#[async_trait::async_trait]
pub trait Embedder {
    async fn embed_batch(&self, inputs: &[String]) -> Result<Vec<Embedding>, String>;
}

pub struct MistralEmbedder {
    pub dim: usize,
    pub api_key: String,
    pub endpoint: String,
    pub model: String,
    pub client: Client,
}

#[derive(Serialize)]
struct MistralEmbeddingRequest<'a> {
    model: &'a str,
    inputs: &'a [String],
}

#[derive(Deserialize)]
struct MistralEmbeddingResponse {
    data: Vec<MistralEmbeddingData>,
}

#[derive(Deserialize)]
struct MistralEmbeddingData {
    embedding: Vec<f32>,
}

#[async_trait::async_trait]
impl Embedder for MistralEmbedder {
    async fn embed_batch(&self, inputs: &[String]) -> Result<Vec<Embedding>, String> {
        let req_body = MistralEmbeddingRequest {
            model: &self.model,
            inputs,
        };
        let res = self
            .client
            .post(&self.endpoint)
            .header(AUTHORIZATION, format!("Bearer {}", self.api_key))
            .header(CONTENT_TYPE, "application/json")
            .json(&req_body)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {e}"))?;
        if !res.status().is_success() {
            return Err(format!("Mistral API error: status {}", res.status()));
        }
        let parsed: MistralEmbeddingResponse = res
            .json()
            .await
            .map_err(|e| format!("Error parsing response: {e}"))?;
        if parsed.data.len() != inputs.len() {
            return Err(format!(
                "API returned {} embeddings for {} inputs",
                parsed.data.len(),
                inputs.len()
            ));
        }
        Ok(parsed.data.into_iter().map(|d| d.embedding).collect())
    }
}

pub async fn embed_all<E: Embedder + Sync + Send>(embedder: &E, texts: Vec<String>) -> EmbedResult {
    let mut all_embeddings = Vec::new();
    let batches = make_batches(&texts);
    for batch in batches {
        match embedder.embed_batch(&batch).await {
            Ok(embs) => {
                all_embeddings.extend(embs);
            }
            Err(e) => {
                all_embeddings.extend((0..batch.len()).map(|_| Vec::new()));
                return EmbedResult {
                    input: texts,
                    embeddings: Some(all_embeddings),
                    error: Some(format!("Erreur embedding batch : {e}")),
                };
            }
        }
    }
    EmbedResult {
        input: texts,
        embeddings: Some(all_embeddings),
        error: None,
    }
}
