use crate::types::Chunk;
use tiktoken_rs::cl100k_base;

const MISTRAL_EMBEDDING_PRICE_PER_MILLION_TOKEN: f64 = 0.15;
const MISTRAL_COMPLETION_PRICE_PER_MILLION_TOKEN: f64 = 0.4;
const MISTRAL_RESONNING_COMPLETION_PRICE_PER_MILLION_TOKEN: f64 = 2.0;

const MAX_BATCH_SIZE: usize = 128;
const MAX_TOTAL_TOKENS: usize = 16384;
pub const MAX_SEQUENCE_LENGTH: usize = 8192;

pub fn calculate_cost(all_chunks: &[Chunk]) {
    let bpe = cl100k_base().unwrap();

    let total_tokens: usize = all_chunks
        .iter()
        .map(|chunk| bpe.encode_with_special_tokens(&chunk.text).len())
        .sum();

    let total_cost =
        (total_tokens as f64 / 1_000_000.0) * MISTRAL_EMBEDDING_PRICE_PER_MILLION_TOKEN;

    println!(
        "Total tokens: {}\nEstimated embedding cost: {:.2} $",
        total_tokens, total_cost
    );
}

pub fn calculate_ask_cost(input: &String) {
    let bpe = cl100k_base().unwrap();

    let total_tokens = bpe.encode_with_special_tokens(input).len();
    let total_cost_standard =
        (total_tokens as f64 / 1_000_000.0) * MISTRAL_COMPLETION_PRICE_PER_MILLION_TOKEN;
    let total_cost_premium =
        (total_tokens as f64 / 1_000_000.0) * MISTRAL_RESONNING_COMPLETION_PRICE_PER_MILLION_TOKEN;

    println!(
        "Total tokens: {}\nEstimated input cost: {:.2} $ + unpredictable reponse cost with standard model",
        total_tokens, total_cost_standard
    );
    println!(
        "Total tokens: {}\nEstimated input cost: {:.2} $ + unpredictable reponse cost with premium model",
        total_tokens, total_cost_premium
    );
}

pub fn make_batches(chunks: Vec<Chunk>) -> Vec<Vec<Chunk>> {
    let mut batches = Vec::new();
    let mut current_batch: Vec<Chunk> = Vec::new();
    let mut current_batch_tokens = 0;

    let enc = cl100k_base().expect("error instantiation tiktoken_rs");
    for chunk in chunks {
        let tokens = enc.encode_ordinary(&chunk.text);
        let t = if tokens.len() > MAX_SEQUENCE_LENGTH {
            enc.decode(tokens[..MAX_SEQUENCE_LENGTH].to_vec())
                .unwrap_or_default()
        } else {
            chunk.text.clone()
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
        let prepared_chunk = Chunk {
            path: chunk.path,
            chunk_end_line: chunk.chunk_end_line,
            chunk_index: chunk.chunk_index,
            chunk_start_line: chunk.chunk_start_line,
            text: t,
        };
        current_batch.push(prepared_chunk);
        current_batch_tokens += n_tokens;
    }
    if !current_batch.is_empty() {
        batches.push(current_batch);
    }
    batches
}

pub fn prepare_chunk(path: &str, index: usize, text: &str) -> String {
    format!("file: {} | {}\n\n{}", path, index, text.trim())
}

pub fn compute_repo_identifier(repo_path: &String) -> String {
    let trimmed = if repo_path.ends_with('/') {
        &repo_path[..repo_path.len() - 1]
    } else {
        &repo_path
    };

    trimmed.replace("./", "").replace("/", "_")
}
