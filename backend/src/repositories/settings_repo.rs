use std::sync::Arc;

use axum::http::StatusCode;
use sqlx::PgPool;

use crate::{AppError, infrastructure::crypto::CryptoService, models::domain::AppSetting};

#[derive(Clone)]
pub struct SettingsRepository {
    pool: Arc<PgPool>,
    crypto: CryptoService,
}

impl SettingsRepository {
    pub fn new(pool: Arc<PgPool>, crypto: CryptoService) -> Self {
        Self { pool, crypto }
    }

    pub async fn get_settings(&self) -> Result<AppSetting, AppError> {
        let setting = sqlx::query_as!(
            AppSetting,
            "SELECT id, llm_base_url, llm_model, llm_api_key_encrypted, updated_at FROM settings WHERE id = 'singleton'"
        )
        .fetch_optional(&*self.pool)
        .await?;

        // Return empty settings if the row somehow doesn't exist, though migration inserts it.
        if let Some(s) = setting {
            Ok(s)
        } else {
            Ok(AppSetting {
                id: "singleton".to_string(),
                llm_base_url: "https://api.openai.com/v1".to_string(),
                llm_model: "gpt-4o".to_string(),
                llm_api_key_encrypted: None,
                updated_at: None,
            })
        }
    }

    pub async fn update_settings(
        &self,
        llm_base_url: &str,
        llm_model: &str,
        llm_api_key: Option<&str>,
    ) -> Result<AppSetting, AppError> {
        let encrypted_key = if let Some(key) = llm_api_key {
            // Do not update the key if the frontend submitted the placeholder
            if key.starts_with("sk-...") {
                None
            } else {
                Some(self.crypto.encrypt(key).map_err(|_| {
                    AppError::new(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to encrypt API key",
                    )
                })?)
            }
        } else {
            None
        };

        // If encrypted_key is Some, we update the key field, otherwise preserve existing
        let setting = if let Some(enc_key) = encrypted_key {
            sqlx::query_as!(
                AppSetting,
                "UPDATE settings 
                 SET llm_base_url = $1, llm_model = $2, llm_api_key_encrypted = $3, updated_at = CURRENT_TIMESTAMP
                 WHERE id = 'singleton'
                 RETURNING id, llm_base_url, llm_model, llm_api_key_encrypted, updated_at",
                 llm_base_url,
                 llm_model,
                 enc_key
            )
            .fetch_one(&*self.pool)
            .await?
        } else {
            sqlx::query_as!(
                AppSetting,
                "UPDATE settings 
                 SET llm_base_url = $1, llm_model = $2, updated_at = CURRENT_TIMESTAMP
                 WHERE id = 'singleton'
                 RETURNING id, llm_base_url, llm_model, llm_api_key_encrypted, updated_at",
                llm_base_url,
                llm_model,
            )
            .fetch_one(&*self.pool)
            .await?
        };

        Ok(setting)
    }

    pub async fn get_decrypted_api_key(&self) -> Result<Option<String>, AppError> {
        let setting = self.get_settings().await?;
        if let Some(enc_key) = setting.llm_api_key_encrypted {
            let dec_key = self.crypto.decrypt(&enc_key).map_err(|_| {
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to decrypt API key",
                )
            })?;
            Ok(Some(dec_key))
        } else {
            Ok(None)
        }
    }
}
