//! # Rig RAG Service
//!
//! Integrates the **Rig** framework (`rig-core`) as the RAG pipeline backbone.
//!
//! Architecture:
//! - **Rig OpenAI provider** (with custom base URL) → for Groq/OpenAI generation
//! - **Rig OpenAI provider** (with Gemini endpoint) → for embeddings
//! - **Custom `PgVectorIndex`** → implements Rig's vector retrieval,
//!   bridging Rig's RAG pipeline to our existing pgvector tables
//! - **`RigRagService`** → orchestrates the full RAG pipeline:
//!   embed query → retrieve from knowledge base → build context → generate reply

use std::sync::Arc;

use pgvector::Vector;
use rig::embeddings::{Embed, EmbeddingsBuilder, TextEmbedder};
use rig::prelude::EmbeddingsClient;
use rig::providers::openai;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use text_splitter::TextSplitter;
use uuid::Uuid;

use crate::app_error::AppError;
use axum::http::StatusCode;

// ─── Document types for Rig's Embed trait ───────────────────────────────────

/// A knowledge document that implements Rig's `Embed` trait manually.
/// The `text` field is embedded for vector generation.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct KnowledgeDocument {
    pub id: String,
    pub text: String,
    pub title: String,
    pub chunk_index: i32,
    pub entry_id: String,
}

/// Manual implementation of Rig's Embed trait
/// (avoids derive macro's rig_core crate resolution issue)
impl Embed for KnowledgeDocument {
    fn embed(&self, embedder: &mut TextEmbedder) -> Result<(), rig::embeddings::EmbedError> {
        embedder.embed(self.text.clone());
        Ok(())
    }
}

/// A retrieved knowledge result from the vector store
#[derive(Debug, Clone)]
pub struct RetrievedContext {
    pub title: String,
    pub text: String,
    pub score: f64,
}

// ─── Custom pgvector VectorStoreIndex for Rig ───────────────────────────────

/// Bridges Rig's RAG pipeline to our PostgreSQL pgvector tables.
/// Uses Rig's OpenAI-compatible embedding client for vector generation.
pub struct PgVectorIndex {
    pool: Arc<PgPool>,
    embedding_client: openai::CompletionsClient,
    embedding_model_name: String,
}

impl PgVectorIndex {
    pub fn new(pool: Arc<PgPool>, embedding_client: openai::CompletionsClient, embedding_model_name: String) -> Self {
        Self {
            pool,
            embedding_client,
            embedding_model_name,
        }
    }

    /// Generate an embedding vector using Rig's OpenAI-compatible embedding client
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let model = self.embedding_client.embedding_model(&self.embedding_model_name);

        let embeddings = EmbeddingsBuilder::new(model)
            .document(KnowledgeDocument {
                id: Uuid::new_v4().to_string(),
                text: text.to_string(),
                title: String::new(),
                chunk_index: 0,
                entry_id: String::new(),
            })
            .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("Rig embed error: {}", e)))?
            .build()
            .await
            .map_err(|e| AppError::new(StatusCode::BAD_GATEWAY, format!("Rig Embedding API error: {}", e)))?;

        // Extract the embedding vector from Rig's response
        if let Some((_doc, embeddings)) = embeddings.first() {
            let embedding = embeddings.first_ref();
            Ok(embedding.vec.iter().map(|&f| f as f32).collect())
        } else {
            Err(AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "No embedding returned by Rig"))
        }
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
            SELECT ke.title, kc.chunk_text, (1.0 - (kc.embedding <=> $1)) as similarity
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
                });
            }
        }

        // Sort by relevance score descending, take top results
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit as usize);

        Ok(results)
    }
}

// ─── Rig RAG Service ────────────────────────────────────────────────────────

/// The main RAG service powered by the **Rig** framework.
/// Orchestrates: embedding → retrieval → context building → LLM generation.
pub struct RigRagService {
    pub vector_index: Arc<PgVectorIndex>,
    _generation_client: openai::CompletionsClient,
    _generation_model: String,
}

impl RigRagService {
    /// Create a new Rig-powered RAG service.
    ///
    /// Uses the builder pattern from Rig 0.36 to create OpenAI-compatible clients:
    /// - One for generation (Groq/OpenAI)
    /// - One for embeddings (Gemini's OpenAI-compatible endpoint)
    pub fn new(
        generation_base_url: &str,
        generation_api_key: &str,
        generation_model: &str,
        embedding_base_url: &str,
        embedding_api_key: &str,
        embedding_model: &str,
        pool: Arc<PgPool>,
    ) -> Self {
        // Create Rig OpenAI-compatible client for generation (Groq)
        // Using completions API (compatible with Groq/OpenAI v1 endpoints)
        let generation_client = openai::Client::builder()
            .api_key(generation_api_key)
            .base_url(generation_base_url)
            .build()
            .expect("Failed to build Rig generation client")
            .completions_api();

        // Create Rig OpenAI-compatible client for embeddings (Gemini OpenAI-compat endpoint)
        let embedding_client = openai::Client::builder()
            .api_key(embedding_api_key)
            .base_url(embedding_base_url)
            .build()
            .expect("Failed to build Rig embedding client")
            .completions_api();

        let vector_index = Arc::new(PgVectorIndex::new(
            pool,
            embedding_client,
            embedding_model.to_string(),
        ));

        Self {
            vector_index,
            _generation_client: generation_client,
            _generation_model: generation_model.to_string(),
        }
    }

    /// Embed and store a knowledge chunk using Rig's embedding pipeline
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
        .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {}", e)))?;

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
