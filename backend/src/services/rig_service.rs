//! # Rig RAG Service
//!
//! Architecture:
//! - **Rig OpenAI provider** (with custom base URL) → for Groq/OpenAI generation
//! - **Direct reqwest call** → for Gemini embeddings (Rig's OpenAI-compat client
//!   cannot parse Gemini's embedding response format)
//! - **Custom `PgVectorIndex`** → bridges our pgvector tables to the RAG pipeline
//! - **`RigRagService`** → orchestrates: embed query → retrieve → generate reply

use std::sync::Arc;

use pgvector::Vector;
use rig::providers::openai;
use serde::Deserialize;
use sqlx::{PgPool, Row};
use text_splitter::TextSplitter;
use uuid::Uuid;

use crate::app_error::AppError;
use axum::http::StatusCode;

// ─── Retrieved context ───────────────────────────────────────────────────────

/// A retrieved knowledge result from the vector store
#[derive(Debug, Clone)]
pub struct RetrievedContext {
    pub title: String,
    pub text: String,
    pub score: f64,
    /// Set for KB entries so the stream handler can increment retrieval_count.
    pub entry_id: Option<Uuid>,
}

// ─── Gemini embedding response structs ───────────────────────────────────────

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f64>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

// ─── PgVectorIndex ───────────────────────────────────────────────────────────

/// Bridges our PostgreSQL pgvector tables to the RAG pipeline.
/// Uses direct reqwest calls for Gemini embeddings — Rig's OpenAI-compatible
/// client returns `"data did not match any variant of untagged enum ApiResponse"`
/// when parsing Gemini's response, so we call the endpoint ourselves.
pub struct PgVectorIndex {
    pool: Arc<PgPool>,
    embedding_base_url: String,
    embedding_api_key: String,
    embedding_model_name: String,
    http: reqwest::Client,
}

impl PgVectorIndex {
    pub fn new(
        pool: Arc<PgPool>,
        embedding_base_url: String,
        embedding_api_key: String,
        embedding_model_name: String,
    ) -> Self {
        Self {
            pool,
            embedding_base_url,
            embedding_api_key,
            embedding_model_name,
            http: reqwest::Client::new(),
        }
    }

    /// Call the Gemini OpenAI-compatible embedding endpoint directly via reqwest.
    /// Returns a float vector ready for pgvector storage.
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let url = format!(
            "{}/embeddings",
            self.embedding_base_url.trim_end_matches('/')
        );

        let body = serde_json::json!({
            "model": self.embedding_model_name,
            "input": text
        });

        let response = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.embedding_api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                AppError::new(
                    StatusCode::BAD_GATEWAY,
                    format!("Embedding API request failed: {}", e),
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            return Err(AppError::new(
                StatusCode::BAD_GATEWAY,
                format!("Embedding API returned {}: {}", status, body_text),
            ));
        }

        let parsed: EmbeddingResponse = response.json().await.map_err(|e| {
            AppError::new(
                StatusCode::BAD_GATEWAY,
                format!("Failed to parse embedding response: {}", e),
            )
        })?;

        let floats = parsed
            .data
            .into_iter()
            .next()
            .ok_or_else(|| {
                AppError::new(StatusCode::BAD_GATEWAY, "No embedding data in response")
            })?
            .embedding
            .into_iter()
            .map(|f| f as f32)
            .collect();

        Ok(floats)
    }

    /// Search both knowledge_embeddings and email_embeddings for RAG context
    pub async fn search_all_context(
        &self,
        query_embedding: &[f32],
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RetrievedContext>, AppError> {
        let vec = Vector::from(query_embedding.to_vec());
        let mut results = Vec::new();

        // 1. Search knowledge base (global entries)
        let kb_rows = sqlx::query(
            r#"
            SELECT ke.id as entry_id, ke.title, kc.chunk_text,
                   (1.0 - (kc.embedding <=> $1)) as similarity
            FROM knowledge_embeddings kc
            JOIN knowledge_entries ke ON kc.entry_id = ke.id
            WHERE kc.embedding IS NOT NULL
            ORDER BY kc.embedding <=> $1
            LIMIT $2
            "#,
        )
        .bind(&vec)
        .bind(limit)
        .fetch_all(&*self.pool)
        .await
        .unwrap_or_default();

        for row in &kb_rows {
            results.push(RetrievedContext {
                title: row.get("title"),
                text: row.get("chunk_text"),
                score: row.get::<f64, _>("similarity"),
                entry_id: row.try_get("entry_id").ok(),
            });
        }

        // 2. Search past emails (per-user)
        let email_rows = sqlx::query(
            r#"
            SELECT e.original_content, e.generated_response,
                   (1.0 - (ee.content_embedding <=> $1)) as similarity
            FROM emails e
            JOIN email_embeddings ee ON e.id = ee.email_id
            WHERE e.user_id = $2 AND ee.content_embedding IS NOT NULL
            ORDER BY ee.content_embedding <=> $1
            LIMIT $3
            "#,
        )
        .bind(&vec)
        .bind(user_id)
        .bind(limit)
        .fetch_all(&*self.pool)
        .await
        .unwrap_or_default();

        for row in &email_rows {
            let content: String = row.get("original_content");
            let response: Option<String> = row.get("generated_response");
            if let Some(resp) = response {
                results.push(RetrievedContext {
                    title: "Past Email".to_string(),
                    text: format!("Received: {}\nReplied: {}", content, resp),
                    score: row.get::<f64, _>("similarity"),
                    entry_id: None,
                });
            }
        }

        // Sort by relevance score descending, take top results
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit as usize);

        Ok(results)
    }
}

// ─── RigRagService ───────────────────────────────────────────────────────────

/// Orchestrates: embedding → retrieval → context building → LLM generation.
pub struct RigRagService {
    pub vector_index: Arc<PgVectorIndex>,
    _generation_client: openai::CompletionsClient,
    _generation_model: String,
}

impl RigRagService {
    pub fn new(
        generation_base_url: &str,
        generation_api_key: &str,
        generation_model: &str,
        embedding_base_url: &str,
        embedding_api_key: &str,
        embedding_model: &str,
        pool: Arc<PgPool>,
    ) -> Self {
        // Rig OpenAI-compatible client for generation (Groq/OpenAI)
        let generation_client = openai::Client::builder()
            .api_key(generation_api_key)
            .base_url(generation_base_url)
            .build()
            .expect("Failed to build Rig generation client")
            .completions_api();

        // Embeddings use direct reqwest — Rig's client can't parse Gemini responses
        let vector_index = Arc::new(PgVectorIndex::new(
            pool,
            embedding_base_url.to_string(),
            embedding_api_key.to_string(),
            embedding_model.to_string(),
        ));

        Self {
            vector_index,
            _generation_client: generation_client,
            _generation_model: generation_model.to_string(),
        }
    }

    /// Embed and store a knowledge chunk via direct Gemini embedding call
    pub async fn embed_and_store_chunk(
        &self,
        entry_id: Uuid,
        chunk_text: &str,
        chunk_index: i32,
        pool: &PgPool,
    ) -> Result<(), AppError> {
        let embedding_floats = self.vector_index.embed_text(chunk_text).await?;
        let vec = Vector::from(embedding_floats);

        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO knowledge_embeddings (id, entry_id, chunk_text, chunk_index, embedding)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(entry_id)
        .bind(chunk_text)
        .bind(chunk_index)
        .bind(&vec)
        .execute(pool)
        .await
        .map_err(|e| {
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("DB error storing chunk: {}", e),
            )
        })?;

        Ok(())
    }

    /// Chunk a document using the text-splitter framework (semantic-aware splitting)
    pub fn chunk_document(text: &str) -> Vec<String> {
        let splitter = TextSplitter::new(500);
        splitter.chunks(text).map(|c| c.to_string()).collect()
    }

    /// Get an embedding Vector for storing in the email_embeddings table
    pub async fn embed_for_storage(&self, text: &str) -> Result<Vector, AppError> {
        let floats = self.vector_index.embed_text(text).await?;
        Ok(Vector::from(floats))
    }
}
