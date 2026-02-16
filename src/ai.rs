use anyhow::{Context, Result, bail};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

use crate::commit::SYSTEM_PROMPT;

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    temperature: f32,
    messages: Vec<Message<'a>>,
}

#[derive(Debug, Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: String,
}

pub async fn generate_commit_message(api_key: &str, model: &str, diff: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let payload = ChatRequest {
        model,
        temperature: 0.2,
        messages: vec![
            Message {
                role: "system",
                content: SYSTEM_PROMPT,
            },
            Message {
                role: "user",
                content: diff,
            },
        ],
    };

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()
        .await
        .context("request to Groq failed")?
        .error_for_status()
        .context("Groq returned non-success status")?
        .json::<ChatResponse>()
        .await
        .context("failed parsing Groq response")?;

    let Some(choice) = response.choices.first() else {
        bail!("Groq response had no message choices");
    };

    Ok(choice.message.content.trim().to_owned())
}
