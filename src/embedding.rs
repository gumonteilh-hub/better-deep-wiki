use reqwest::Client;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::fmt;

use tiktoken_rs::cl100k_base;

use crate::types::{Chunk, Embedding};
use crate::utils::MAX_SEQUENCE_LENGTH;

pub type EmbeddingVector = Vec<f32>;

#[derive(Debug)]
pub struct EmbedResult {
    pub input: Vec<String>,
    pub embeddings: Option<Vec<EmbeddingVector>>,
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

#[async_trait::async_trait]
pub trait Embedder {
    async fn embed_batch(&self, inputs: Vec<Chunk>) -> Result<Vec<Embedding>, String>;
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
    input: &'a [String],
}

#[derive(Deserialize)]
struct MistralEmbeddingResponse {
    data: Vec<MistralEmbeddingData>,
}

#[derive(Deserialize)]
struct MistralEmbeddingData {
    embedding: Vec<f32>,
}

impl MistralEmbedder {
    pub fn from_env(dim: usize) -> Self {
        let api_key = std::env::var("MISTRAL_API_KEY").expect("MISTRAL_API_KEY must be set");
        let endpoint = std::env::var("MISTRAL_ENDPOINT")
            .unwrap_or_else(|_| "https://api.mistral.ai/v1/embeddings".to_string());
        let model =
            std::env::var("MISTRAL_MODEL").unwrap_or_else(|_| "codestral-embed".to_string());

        let client = Client::new();

        Self {
            dim,
            api_key,
            endpoint,
            model,
            client,
        }
    }
}

#[async_trait::async_trait]
impl Embedder for MistralEmbedder {
    async fn embed_batch(&self, inputs: Vec<Chunk>) -> Result<Vec<Embedding>, String> {
        let text_inputs: Vec<_> = inputs
            .iter()
            .map(|c| c.clone().text)
            .filter(|s| {
                !s.trim().is_empty()
                    && s.len() <= MAX_SEQUENCE_LENGTH
                    && s.is_char_boundary(s.len())
            })
            .collect();
        if text_inputs.is_empty() {
            let paths: Vec<_> = inputs.iter().map(|c| &c.path).collect();
            return Err(format!("Empty input batch for files: {:?}", paths));
        }
        let req_body = MistralEmbeddingRequest {
            model: &self.model,
            input: &text_inputs,
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
            let status = res.status();
            let body = res
                .text()
                .await
                .unwrap_or_else(|e| format!("(error reading body: {e})"));
            return Err(format!(
                "Mistral API error: status {} — body: {}",
                status, body
            ));
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
        let mut result = Vec::new();

        for (index, data) in parsed.data.into_iter().enumerate() {
            result.push(Embedding::new(
                inputs[index].path.clone(),
                inputs[index].chunk_index,
                inputs[index].chunk_start_line,
                inputs[index].chunk_end_line,
                data.embedding,
            ));
        }

        Ok(result)
    }
}
