use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use crate::{app_error::AppError, models::domain::Email};
use axum::http::StatusCode;
use pgvector::Vector;

#[async_trait]
pub trait LlmService: Send + Sync {
    async fn generate_reply(
        &self,
        email: &Email,
        context: &str,
        base_url: &str,
        model: &str,
        api_key: &str,
    ) -> Result<String, AppError>;

    async fn generate_embedding(
        &self,
        text: &str,
        base_url: &str,
        model: &str,
        api_key: &str,
    ) -> Result<Vector, AppError>;
}

pub struct UnifiedLlmService {
    client: Client,
}

impl UnifiedLlmService {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmService for UnifiedLlmService {
    async fn generate_reply(
        &self,
        email: &Email,
        context: &str,
        base_url: &str,
        model: &str,
        api_key: &str,
    ) -> Result<String, AppError> {
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        let system_prompt = if context.is_empty() {
            "You are a professional and helpful AI email assistant. Draft a polite, concise, and appropriate reply to the following received email from a client/colleague. Return only the direct text of the response you draft.".to_string()
        } else {
            format!(
                "You are an AI email assistant. You learn from the user's past email replies to mimic their tone and utilize their knowledge. \
                \n\nHere is how the user previously replied to similar emails:\n{}\n\n\
                Use the style and facts defined above to generate a polite and concise drafted reply to the following new incoming email. Return only the response text.",
                context
            )
        };

        let payload = json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": &email.original_content
                }
            ],
            "max_tokens": 1000
        });

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(response) => {
                if !response.status().is_success() {
                    let status = response.status();
                    let body = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    return Err(AppError::new(
                        StatusCode::BAD_GATEWAY,
                        format!("LLM API returned an error ({}): {}", status, body),
                    ));
                }

                let response_json: serde_json::Value = response.json().await.map_err(|e| {
                    AppError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to parse LLM JSON response: {}", e),
                    )
                })?;

                if let Some(content) = response_json["choices"][0]["message"]["content"].as_str() {
                    Ok(content.to_string())
                } else {
                    Err(AppError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Unexpected response structure from LLM".to_string(),
                    ))
                }
            }
            Err(e) => Err(AppError::new(
                StatusCode::BAD_GATEWAY,
                format!("Failed to reach LLM API: {}", e),
            )),
        }
    }

    async fn generate_embedding(
        &self,
        text: &str,
        base_url: &str,
        model: &str,
        api_key: &str,
    ) -> Result<Vector, AppError> {
        let url = format!("{}/embeddings", base_url.trim_end_matches('/'));

        let payload = json!({
            "model": model,
            "input": text
        });

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(response) => {
                if !response.status().is_success() {
                    let status = response.status();
                    let body = response
                        .text()
                        .await
                        .unwrap_or_else(|_| "Unknown error".to_string());
                    return Err(AppError::new(
                        StatusCode::BAD_GATEWAY,
                        format!("Embedding API returned an error ({}): {}", status, body),
                    ));
                }

                let response_json: serde_json::Value = response.json().await.map_err(|e| {
                    AppError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to parse LLM Embedding JSON response: {}", e),
                    )
                })?;

                if let Some(data) = response_json["data"][0]["embedding"].as_array() {
                    let floats: Vec<f32> = data
                        .iter()
                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                        .collect();
                    Ok(Vector::from(floats))
                } else {
                    Err(AppError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Unexpected response structure from Embedding API".to_string(),
                    ))
                }
            }
            Err(e) => Err(AppError::new(
                StatusCode::BAD_GATEWAY,
                format!("Failed to reach Embedding API: {}", e),
            )),
        }
    }
}
