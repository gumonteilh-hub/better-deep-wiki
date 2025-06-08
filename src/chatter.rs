use dotenvy::var;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize}; // pour charger la clé API

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Deserialize, Debug)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize, Debug)]
struct MessageContent {
    content: String,
}

pub async fn chat_mistral(prompt: String) -> Result<String, String> {
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
    };

    let response = client
        .post(endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&req_body)
        .send()
        .await
        .map_err(|e| format!("HTTP error: {e}"))?;

    if response.status() != StatusCode::OK {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Mistral API error: status {status} — body: {text}"));
    }

    let res: ChatResponse = response
        .json()
        .await
        .map_err(|e| format!("Error parsing response JSON: {e}"))?;

    res.choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "No response from Mistral API".to_string())
}
