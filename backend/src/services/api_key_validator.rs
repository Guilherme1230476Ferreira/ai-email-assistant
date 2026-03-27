use anyhow::{anyhow, Context, Result};

pub async fn validate_api_key(api_key: &str, client: &reqwest::Client) -> Result<()> {
    if api_key.trim().is_empty() {
        return Err(anyhow!("api_key cannot be empty"));
    }

    let openai_result = validate_openai(api_key, client).await;
    if openai_result.is_ok() {
        return Ok(());
    }

    let gemini_result = validate_gemini(api_key, client).await;
    if gemini_result.is_ok() {
        return Ok(());
    }

    Err(anyhow!(
        "api_key validation failed for all supported providers (OpenAI/Gemini)"
    ))
}

async fn validate_openai(api_key: &str, client: &reqwest::Client) -> Result<()> {
    let response = client
        .get("https://api.openai.com/v1/models")
        .bearer_auth(api_key)
        .send()
        .await
        .context("failed to call OpenAI validation endpoint")?;

    if response.status().is_success() {
        return Ok(());
    }

    if response.status().as_u16() == 401 {
        return Err(anyhow!("OpenAI API key is invalid"));
    }

    Err(anyhow!(
        "OpenAI validation failed with status {}",
        response.status()
    ))
}

async fn validate_gemini(api_key: &str, client: &reqwest::Client) -> Result<()> {
    let response = client
        .get("https://generativelanguage.googleapis.com/v1beta/models")
        .query(&[("key", api_key)])
        .send()
        .await
        .context("failed to call Gemini validation endpoint")?;

    if response.status().is_success() {
        return Ok(());
    }

    if response.status().as_u16() == 401 || response.status().as_u16() == 403 {
        return Err(anyhow!("Gemini API key is invalid"));
    }

    Err(anyhow!(
        "Gemini validation failed with status {}",
        response.status()
    ))
}