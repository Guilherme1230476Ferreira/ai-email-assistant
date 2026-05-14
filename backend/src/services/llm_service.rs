use std::pin::Pin;

use async_trait::async_trait;
use futures_util::Stream;
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

    /// Returns a stream of token chunks from the LLM.
    async fn generate_reply_stream(
        &self,
        email: &Email,
        context: &str,
        base_url: &str,
        model: &str,
        api_key: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, AppError>> + Send>>, AppError>;
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

    fn build_system_prompt(context: &str) -> String {
        if context.is_empty() {
            "You are a professional and helpful AI email assistant. Draft a polite, concise, and appropriate reply to the following received email from a client/colleague. Return only the direct text of the response you draft.".to_string()
        } else {
            format!(
                "You are an AI email assistant. You learn from the user's past email replies to mimic their tone and utilize their knowledge. \
                You also have access to a knowledge base with relevant company/domain information.\
                \n\nHere is the relevant context from past emails and the knowledge base:\n{}\n\n\
                Use the style, facts, and knowledge provided above to generate a polite and concise drafted reply to the following new incoming email. Return only the response text.",
                context
            )
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
        let system_prompt = Self::build_system_prompt(context);

        let payload = json!({
            "model": model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": &email.original_content }
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

    async fn generate_reply_stream(
        &self,
        email: &Email,
        context: &str,
        base_url: &str,
        model: &str,
        api_key: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, AppError>> + Send>>, AppError> {
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
        let system_prompt = Self::build_system_prompt(context);

        let payload = json!({
            "model": model,
            "stream": true,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": &email.original_content }
            ],
            "max_tokens": 1000
        });

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                AppError::new(
                    StatusCode::BAD_GATEWAY,
                    format!("Failed to reach LLM API: {}", e),
                )
            })?;

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

        let byte_stream = response.bytes_stream();

        let token_stream = async_stream::stream! {
            use futures_util::StreamExt;
            let mut buffer = String::new();

            futures_util::pin_mut!(byte_stream);
            while let Some(chunk_result) = byte_stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        buffer.push_str(&String::from_utf8_lossy(&bytes));

                        // Process complete lines
                        while let Some(pos) = buffer.find('\n') {
                            let line = buffer[..pos].trim().to_string();
                            buffer = buffer[pos + 1..].to_string();

                            if line.is_empty() || line == "data: [DONE]" {
                                continue;
                            }

                            if let Some(json_str) = line.strip_prefix("data: ") {
                                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
                                    if let Some(content) = parsed["choices"][0]["delta"]["content"].as_str() {
                                        if !content.is_empty() {
                                            yield Ok(content.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(AppError::new(
                            StatusCode::BAD_GATEWAY,
                            format!("Stream read error: {}", e),
                        ));
                        return;
                    }
                }
            }
        };

        Ok(Box::pin(token_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_system_prompt_empty_context() {
        let prompt = UnifiedLlmService::build_system_prompt("");
        assert!(prompt.contains("professional and helpful AI email assistant"));
        assert!(prompt.contains("Draft a polite, concise"));
    }

    #[test]
    fn test_build_system_prompt_with_context() {
        let context = "Past reply: 'Hi, I can help with that.'";
        let prompt = UnifiedLlmService::build_system_prompt(context);
        assert!(prompt.contains("learn from the user's past email replies"));
        assert!(prompt.contains(context));
    }
}
