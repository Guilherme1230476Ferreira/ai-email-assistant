use crate::models::domain::KnowledgeEntry;
use pgvector::Vector;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

/// A knowledge chunk returned from a similarity search, including its parent entry.
pub struct SimilarKnowledge {
    pub title: String,
    pub chunk_text: String,
}

#[derive(Clone)]
pub struct KnowledgeRepository {
    pool: Arc<PgPool>,
}

impl KnowledgeRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Expose the pool reference for Rig service operations
    pub fn get_pool(&self) -> PgPool {
        (*self.pool).clone()
    }

    /// Create a knowledge entry (document or Q&A pair)
    pub async fn create_entry(
        &self,
        entry_type: &str,
        title: &str,
        content: &str,
    ) -> Result<KnowledgeEntry, sqlx::Error> {
        sqlx::query_as::<_, KnowledgeEntry>(
            r#"
            INSERT INTO knowledge_entries (entry_type, title, content)
            VALUES ($1, $2, $3)
            RETURNING id, entry_type, title, content, created_at, updated_at
            "#,
        )
        .bind(entry_type)
        .bind(title)
        .bind(content)
        .fetch_one(&*self.pool)
        .await
    }

    /// Store a single chunk embedding for a knowledge entry
    pub async fn create_chunk(
        &self,
        entry_id: Uuid,
        chunk_text: &str,
        chunk_index: i32,
        embedding: Option<Vector>,
    ) -> Result<(), sqlx::Error> {
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
        .bind(embedding)
        .execute(&*self.pool)
        .await?;
        Ok(())
    }

    /// Find the most similar knowledge chunks across ALL entries (global)
    pub async fn find_similar_knowledge(
        &self,
        embedding: &Vector,
        limit: i64,
    ) -> Result<Vec<SimilarKnowledge>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT ke.title, kc.chunk_text
            FROM knowledge_embeddings kc
            JOIN knowledge_entries ke ON kc.entry_id = ke.id
            WHERE kc.embedding IS NOT NULL
            ORDER BY kc.embedding <=> $1
            LIMIT $2
            "#,
        )
        .bind(embedding)
        .bind(limit)
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|row| SimilarKnowledge {
                title: row.get("title"),
                chunk_text: row.get("chunk_text"),
            })
            .collect())
    }

    /// List all knowledge entries with chunk counts (paginated)
    pub async fn get_entries_paginated(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<crate::models::dto::KnowledgeEntryResponse>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                ke.id,
                ke.entry_type,
                ke.title,
                ke.content,
                COUNT(kc.id) as chunk_count,
                ke.created_at
            FROM knowledge_entries ke
            LEFT JOIN knowledge_embeddings kc ON ke.id = kc.entry_id
            GROUP BY ke.id
            ORDER BY ke.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|row| crate::models::dto::KnowledgeEntryResponse {
                id: row.get("id"),
                entry_type: row.get("entry_type"),
                title: row.get("title"),
                content: row.get("content"),
                chunk_count: row.get::<i64, _>("chunk_count"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    /// Count all knowledge entries
    pub async fn get_entry_count(&self) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM knowledge_entries")
            .fetch_one(&*self.pool)
            .await?;
        Ok(row.get::<i64, _>("count"))
    }

    /// Delete a knowledge entry (cascades to chunks + embeddings)
    pub async fn delete_entry(&self, entry_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query("DELETE FROM knowledge_entries WHERE id = $1")
            .bind(entry_id)
            .execute(&*self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }
}
