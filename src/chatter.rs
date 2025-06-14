use dotenvy::var;
use futures_util::StreamExt;
use reqwest::{Client, StatusCode};
use serde::Serialize;
use tokio::sync::mpsc::Sender;

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

pub async fn chat_mistral_stream(prompt: String, tx: Sender<String>) -> Result<(), String> {
    let api_key =
        var("MISTRAL_API_KEY").map_err(|_| "Missing MISTRAL_API_KEY in .env".to_string())?;

    let endpoint = "https://api.mistral.ai/v1/chat/completions";
    let client = Client::new();

    let req_body = ChatRequest {
        model: "mistral-large-latest".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
        stream: true,
    };

    let response = client
        .post(endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .json(&req_body)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    if response.status() != StatusCode::OK {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Mistral API error: status {status} — body: {text}"));
    }

    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Stream error: {e}"))?;
        let chunk_str = String::from_utf8_lossy(&chunk);

        for line in chunk_str.lines() {
            if line.starts_with("data: ") {
                let data = &line[6..];
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(content) = value["choices"]
                        .get(0)
                        .and_then(|c| c["delta"]["content"].as_str())
                    {
                        tx.send(content.to_string()).await.ok();
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn chat_openai_stream(prompt: String, tx: Sender<String>) -> Result<(), String> {
    let api_key =
        var("OPENAI_API_KEY").map_err(|_| "Missing OPENAI_API_KEY in .env".to_string())?;

    let endpoint = "https://api.openai.com/v1/chat/completions";
    let client = Client::new();

    let req_body = ChatRequest {
        model: "gpt-4o-mini".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
        stream: true,
    };

    let response = client
        .post(endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .json(&req_body)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    if response.status() != StatusCode::OK {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("OpenAI API error: status {status} — body: {text}"));
    }

    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| format!("Stream error: {e}"))?;
        let chunk_str = String::from_utf8_lossy(&chunk);

        for line in chunk_str.lines() {
            if line.starts_with("data: ") {
                let data = &line[6..];
                if data.trim() == "[DONE]" {
                    break;
                }
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(data) {
                    if let Some(content) = value["choices"]
                        .get(0)
                        .and_then(|c| c["delta"]["content"].as_str())
                    {
                        tx.send(content.to_string()).await.ok();
                    }
                }
            }
        }
    }

    Ok(())
}
