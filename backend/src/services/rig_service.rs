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
use whatlang::{detect_lang, Lang};

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
    reranker_api_key: String,
    reranker_model: String,
    http: reqwest::Client,
}

impl PgVectorIndex {
    pub fn new(
        pool: Arc<PgPool>,
        embedding_base_url: String,
        embedding_api_key: String,
        embedding_model_name: String,
        reranker_api_key: String,
        reranker_model: String,
    ) -> Self {
        Self {
            pool,
            embedding_base_url,
            embedding_api_key,
            embedding_model_name,
            reranker_api_key,
            reranker_model,
            http: reqwest::Client::new(),
        }
    }

    /// Re-rank a list of retrieved contexts using the Jina cross-encoder.
    ///
    /// The cross-encoder reads the query AND each document together (not
    /// independently like an embedding) — this produces far more accurate
    /// relevance scores than cosine similarity alone.
    ///
    /// Graceful degradation: if the API call fails, returns `candidates`
    /// unchanged so generation continues without interruption.
    pub async fn rerank_results(
        &self,
        query: &str,
        mut candidates: Vec<RetrievedContext>,
        top_n: usize,
    ) -> Vec<RetrievedContext> {
        if candidates.is_empty() || self.reranker_api_key.is_empty() {
            candidates.truncate(top_n);
            return candidates;
        }

        let documents: Vec<&str> = candidates.iter().map(|c| c.text.as_str()).collect();

        let body = serde_json::json!({
            "model": self.reranker_model,
            "query": query,
            "documents": documents,
            "top_n": top_n
        });

        let response = self
            .http
            .post("https://api.jina.ai/v1/rerank")
            .header("Authorization", format!("Bearer {}", self.reranker_api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        match response {
            Ok(res) if res.status().is_success() => {
                match res.json::<serde_json::Value>().await {
                    Ok(json) => {
                        if let Some(results) = json["results"].as_array() {
                            let mut reranked: Vec<RetrievedContext> = results
                                .iter()
                                .filter_map(|r| {
                                    let idx = r["index"].as_u64()? as usize;
                                    let score = r["relevance_score"].as_f64().unwrap_or(0.0);
                                    candidates.get(idx).map(|c| RetrievedContext {
                                        title:    c.title.clone(),
                                        text:     c.text.clone(),
                                        score,
                                        entry_id: c.entry_id,
                                    })
                                })
                                .collect();
                            reranked.truncate(top_n);
                            tracing::debug!("Reranker: {} → {} results", candidates.len(), reranked.len());
                            return reranked;
                        }
                    }
                    Err(e) => tracing::warn!("Reranker JSON parse failed: {}", e),
                }
            }
            Ok(res) => {
                tracing::warn!("Reranker API returned {}: falling back to hybrid scores", res.status());
            }
            Err(e) => tracing::warn!("Reranker request failed: {} — using hybrid scores", e),
        }

        // Fallback: return original list truncated
        candidates.truncate(top_n);
        candidates
    }

    /// Call the Gemini OpenAI-compatible embedding endpoint directly via reqwest.
    /// Automatically retries on HTTP 429 (rate limit) using the delay the API specifies.
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let url = format!(
            "{}/embeddings",
            self.embedding_base_url.trim_end_matches('/')
        );

        let body = serde_json::json!({
            "model": self.embedding_model_name,
            "input": text
        });

        const MAX_RETRIES: u32 = 6;

        for attempt in 1..=MAX_RETRIES {
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

            let status = response.status();

            // ── Rate limit: wait the delay the API tells us, then retry ──────
            if status == 429 {
                let body_text = response.text().await.unwrap_or_default();

                // Parse "retryDelay": "4.619s" from the JSON error body
                let wait_secs: u64 = serde_json::from_str::<serde_json::Value>(&body_text)
                    .ok()
                    .and_then(|v| {
                        v["error"]["details"]
                            .as_array()?
                            .iter()
                            .find(|d| d["@type"].as_str() == Some("type.googleapis.com/google.rpc.RetryInfo"))?
                            ["retryDelay"]
                            .as_str()
                            .map(|s| {
                                // e.g. "4.619256081s" → 5
                                s.trim_end_matches('s')
                                    .parse::<f64>()
                                    .map(|f| (f.ceil() as u64) + 1)
                                    .unwrap_or(10)
                            })
                    })
                    .unwrap_or_else(|| 2u64.pow(attempt)); // exponential fallback

                if attempt < MAX_RETRIES {
                    tracing::warn!(
                        "Gemini embedding 429 (attempt {}/{}): waiting {}s before retry",
                        attempt, MAX_RETRIES, wait_secs
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(wait_secs)).await;
                    continue;
                } else {
                    return Err(AppError::new(
                        StatusCode::BAD_GATEWAY,
                        format!(
                            "Gemini embedding rate-limited after {} retries. \
                             Try uploading a smaller document or wait a minute.",
                            MAX_RETRIES
                        ),
                    ));
                }
            }

            // ── Other non-2xx errors ─────────────────────────────────────────
            if !status.is_success() {
                let body_text = response.text().await.unwrap_or_default();
                return Err(AppError::new(
                    StatusCode::BAD_GATEWAY,
                    format!("Embedding API returned {}: {}", status, body_text),
                ));
            }

            // ── Parse successful response ────────────────────────────────────
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

            return Ok(floats);
        }

        // Unreachable, but satisfies the compiler
        Err(AppError::new(StatusCode::BAD_GATEWAY, "Embedding failed after all retries"))
    }

    /// Search both knowledge_embeddings and email_embeddings for RAG context.
    ///
    /// Knowledge base results use **hybrid scoring**:
    ///   `final = 0.65 × semantic_score + 0.35 × bm25_score`
    ///
    /// BM25 is approximated via PostgreSQL `ts_rank_cd` against the pre-built
    /// `chunk_tsv` GIN index.  For Portuguese queries an additional pass using
    /// the `portuguese` text-search configuration is run and its score merged,
    /// which improves morphological matching (e.g. "projetos" → "projeto").
    pub async fn search_all_context(
        &self,
        query_text: &str,
        query_embedding: &[f32],
        user_id: Uuid,
        limit: i64,
    ) -> Result<Vec<RetrievedContext>, AppError> {
        let vec = Vector::from(query_embedding.to_vec());
        let mut results = Vec::new();

        // ── Language detection (for BM25 config routing) ──────────────────────
        let is_portuguese = detect_lang(query_text) == Some(Lang::Por);
        let ts_config = if is_portuguese { "portuguese" } else { "simple" };

        tracing::debug!(
            "RAG search: lang={} ts_config={} query_len={}",
            if is_portuguese { "pt" } else { "en/other" },
            ts_config,
            query_text.len()
        );

        // ── 1. Knowledge base — hybrid BM25 + semantic ────────────────────────
        // Retrieve top-(limit*2) by semantic, then blend BM25 in Rust.
        // The BM25 column (chunk_tsv) uses 'simple' config (language-agnostic),
        // so the query term normalisation matches what was stored at index time.
        let kb_rows = sqlx::query(
            r#"
            SELECT
                ke.id            AS entry_id,
                ke.title,
                kc.chunk_text,
                (1.0 - (kc.embedding <=> $1))                          AS semantic_score,
                COALESCE(
                    ts_rank_cd(kc.chunk_tsv,
                               plainto_tsquery('simple', $2),
                               32),          -- 32 = normalise by document length
                    0.0
                )::float8                                               AS bm25_simple,
                COALESCE(
                    ts_rank_cd(
                        to_tsvector($3, kc.chunk_text),
                        plainto_tsquery($3, $2),
                        32),
                    0.0
                )::float8                                               AS bm25_lang
            FROM knowledge_embeddings kc
            JOIN knowledge_entries ke ON kc.entry_id = ke.id
            WHERE kc.embedding IS NOT NULL
            ORDER BY kc.embedding <=> $1
            LIMIT $4
            "#,
        )
        .bind(&vec)
        .bind(query_text)
        .bind(ts_config)
        .bind(limit * 2)   // over-fetch so BM25 re-ranking has room
        .fetch_all(&*self.pool)
        .await
        .unwrap_or_default();

        for row in &kb_rows {
            let semantic: f64 = row.get("semantic_score");
            let bm25_simple: f64 = row.get("bm25_simple");
            let bm25_lang: f64   = row.get("bm25_lang");
            // Merge: prefer language-specific BM25 when available
            let bm25 = if bm25_lang > bm25_simple { bm25_lang } else { bm25_simple };
            let hybrid = 0.65 * semantic + 0.35 * bm25;

            results.push(RetrievedContext {
                title:    row.get("title"),
                text:     row.get("chunk_text"),
                score:    hybrid,
                entry_id: row.try_get("entry_id").ok(),
            });
        }

        // ── 2. Past emails — pure semantic (no BM25 column there) ─────────────
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
                    text:  format!("Received: {}\nReplied: {}", content, resp),
                    score: row.get::<f64, _>("similarity"),
                    entry_id: None,
                });
            }
        }

        // ── 3. Merge, hybrid-sort, rerank ─────────────────────────────────────
        // First sort by hybrid score so top candidates go into reranker
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Rerank top candidates with Jina cross-encoder (top-10 → top-5)
        // Gracefully falls back to hybrid scores if the API is unavailable.
        let top_n = limit as usize;
        let candidates = results;
        let results = self.rerank_results(query_text, candidates, top_n).await;

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
        reranker_api_key: &str,
        reranker_model: &str,
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
            reranker_api_key.to_string(),
            reranker_model.to_string(),
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

    /// Chunk a document with a sliding-window overlap so boundary sentences are
    /// preserved in both the preceding and following chunk.
    ///
    /// Strategy:
    ///   1. Split into base chunks of ~500 chars (semantic-aware via text-splitter).
    ///   2. Re-window: prepend the last `OVERLAP` chars of chunk[i-1] to chunk[i].
    ///
    /// This increases total chunk count by ~20%, costing ~20% more embedding calls,
    /// but significantly improves recall for sentences that straddle a boundary.
    pub fn chunk_document(text: &str) -> Vec<String> {
        const CHUNK_SIZE: usize = 500;
        const OVERLAP: usize = 100; // ~20% of chunk size

        let splitter = TextSplitter::new(CHUNK_SIZE);
        let base: Vec<String> = splitter.chunks(text).map(|c| c.to_string()).collect();

        if base.len() <= 1 {
            return base;
        }

        let mut result = Vec::with_capacity(base.len());

        for (i, chunk) in base.iter().enumerate() {
            if i == 0 {
                result.push(chunk.clone());
            } else {
                // Prepend last OVERLAP chars of previous chunk (byte-safe boundary)
                let prev = &base[i - 1];
                let overlap_start = prev
                    .char_indices()
                    .rev()
                    .nth(OVERLAP.saturating_sub(1))
                    .map(|(idx, _)| idx)
                    .unwrap_or(0);
                let tail = &prev[overlap_start..];
                result.push(format!("{} {}", tail.trim(), chunk.trim()));
            }
        }

        result
    }

    /// Get an embedding Vector for storing in the email_embeddings table
    pub async fn embed_for_storage(&self, text: &str) -> Result<Vector, AppError> {
        let floats = self.vector_index.embed_text(text).await?;
        Ok(Vector::from(floats))
    }
}
