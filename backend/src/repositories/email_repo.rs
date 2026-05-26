use crate::models::domain::Email;
#[cfg(test)]
use crate::models::domain::User;
use pgvector::Vector;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct EmailRepository {
    pool: Arc<PgPool>,
}

impl EmailRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_email(
        &self,
        user_id: Uuid,
        original_content: &str,
        generated_response: &str,
        prompt_embedding: Option<Vector>,
        response_embedding: Option<Vector>,
    ) -> Result<Email, sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let email = sqlx::query_as!(
            Email,
            r#"
            INSERT INTO emails (user_id, original_content, generated_response)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, original_content, generated_response, created_at, updated_at
            "#,
            user_id,
            original_content,
            generated_response
        )
        .fetch_one(&mut *tx)
        .await?;

        if prompt_embedding.is_some() || response_embedding.is_some() {
            let emb_id = Uuid::new_v4();
            sqlx::query!(
                r#"
                INSERT INTO email_embeddings (id, email_id, content_embedding, response_embedding)
                VALUES ($1, $2, $3, $4)
                "#,
                emb_id,
                email.id,
                prompt_embedding as Option<Vector>,
                response_embedding as Option<Vector>
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(email)
    }

    pub async fn get_emails_by_user(&self, user_id: Uuid) -> Result<Vec<Email>, sqlx::Error> {
        let emails = sqlx::query_as!(
            Email,
            r#"
            SELECT id, user_id, original_content, generated_response, created_at, updated_at
            FROM emails
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(emails)
    }

    pub async fn get_emails_by_user_paginated(
        &self,
        user_id: Uuid,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Email>, sqlx::Error> {
        sqlx::query_as!(
            Email,
            r#"
            SELECT id, user_id, original_content, generated_response, created_at, updated_at
            FROM emails
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn get_email_count_by_user(&self, user_id: Uuid) -> Result<i64, sqlx::Error> {
        let record = sqlx::query!(
            "SELECT COUNT(*) as count FROM emails WHERE user_id = $1",
            user_id
        )
        .fetch_one(&*self.pool)
        .await?;
        Ok(record.count.unwrap_or(0))
    }

    /// Delete an email by ID, scoped to the owning user for security.
    /// Email embeddings are deleted via ON DELETE CASCADE.
    pub async fn delete_email(&self, email_id: Uuid, user_id: Uuid) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            "DELETE FROM emails WHERE id = $1 AND user_id = $2",
        )
        .bind(email_id)
        .bind(user_id)
        .execute(&*self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_emails_paginated(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<Email>, sqlx::Error> {
        sqlx::query_as!(
            Email,
            r#"
            SELECT id, user_id, original_content, generated_response, created_at, updated_at
            FROM emails
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn get_email_count(&self) -> Result<i64, sqlx::Error> {
        let record = sqlx::query!("SELECT COUNT(*) as count FROM emails")
            .fetch_one(&*self.pool)
            .await?;
        Ok(record.count.unwrap_or(0))
    }

    pub async fn get_telemetry_for_user(
        &self,
        user_id: Uuid,
    ) -> Result<crate::models::dto::TelemetryData, sqlx::Error> {
        // ── 1. Email-level stats ──────────────────────────────────────────────
        let email_record = sqlx::query!(
            r#"
            SELECT
                COUNT(e.id)                                                          AS total_emails,
                SUM(LENGTH(COALESCE(e.generated_response,'')) +
                    LENGTH(e.original_content))                                      AS total_chars,
                COUNT(ee.id)                                                         AS emails_with_embedding,
                -- cosine similarity between prompt and response embeddings:
                -- a good proxy when we don't store per-retrieval scores
                AVG(CASE
                    WHEN ee.content_embedding IS NOT NULL AND ee.response_embedding IS NOT NULL
                    THEN (1.0 - (ee.content_embedding <=> ee.response_embedding))
                    ELSE NULL
                END)                                                                 AS avg_self_similarity
            FROM emails e
            LEFT JOIN email_embeddings ee ON e.id = ee.email_id
            WHERE e.user_id = $1
            "#,
            user_id
        )
        .fetch_one(&*self.pool)
        .await?;

        // ── 2. Knowledge-base hit stats ───────────────────────────────────────
        // Count how many KB chunks exist globally (admin-owned, not per-user).
        // This is the real denominator for KB coverage.
        let kb_record = sqlx::query!(
            r#"
            SELECT COUNT(*) AS total_kb_chunks
            FROM knowledge_embeddings
            WHERE embedding IS NOT NULL
            "#
        )
        .fetch_one(&*self.pool)
        .await?;

        let total_emails    = email_record.total_emails.unwrap_or(0) as f64;
        let total_chars     = email_record.total_chars.unwrap_or(0) as i64;
        let emails_embedded = email_record.emails_with_embedding.unwrap_or(0) as f64;
        let avg_similarity  = email_record.avg_self_similarity.unwrap_or(0.0);
        let kb_chunks       = kb_record.total_kb_chunks.unwrap_or(0) as i64;

        // Embedding coverage: share of emails that have a vector stored
        let context_retrieval_rate = if total_emails > 0.0 {
            (emails_embedded / total_emails).min(1.0)
        } else {
            0.0
        };

        // KB hit rate: if KB has chunks and at least some emails were embedded, assume
        // every embedded email performed a KB retrieval (the RAG pipeline always does).
        let kb_hit_rate = if kb_chunks > 0 && emails_embedded > 0.0 {
            (emails_embedded / total_emails).min(1.0)
        } else {
            0.0
        };

        // knowledge_matches = total KB chunks that could have been retrieved *per* embedded email
        // (limit is 5 in search_all_context, capped by actual KB size)
        let retrievals_per_email = (kb_chunks).min(5);
        let knowledge_matches = emails_embedded as i64 * retrievals_per_email;

        Ok(crate::models::dto::TelemetryData {
            context_retrieval_rate,
            avg_similarity_score: avg_similarity,
            chars_processed: total_chars,
            knowledge_matches,
            kb_hit_rate,
        })
    }

    /// Bucket all self-similarity scores (prompt↔response) for a user's emails
    /// into 5 ranges, returning counts per bucket. Used by the histogram chart.
    pub async fn get_similarity_distribution(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<crate::models::dto::SimilarityBucket>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                width_bucket(
                    CAST(1.0 - (ee.content_embedding <=> ee.response_embedding) AS float8),
                    0.0::float8, 1.0001::float8, 5
                ) AS bucket,
                COUNT(*)::bigint AS cnt
            FROM email_embeddings ee
            JOIN emails e ON e.id = ee.email_id
            WHERE e.user_id = $1
              AND ee.content_embedding IS NOT NULL
              AND ee.response_embedding IS NOT NULL
            GROUP BY bucket
            ORDER BY bucket
            "#,
        )
        .bind(user_id)
        .fetch_all(&*self.pool)
        .await?;

        let labels = ["0.0–0.2", "0.2–0.4", "0.4–0.6", "0.6–0.8", "0.8–1.0"];
        // Pre-fill all 5 buckets with 0
        let mut buckets: Vec<crate::models::dto::SimilarityBucket> = labels
            .iter()
            .map(|l| crate::models::dto::SimilarityBucket {
                label: l.to_string(),
                count: 0,
            })
            .collect();

        for row in &rows {
            let bucket_num: i32 = row.try_get("bucket").unwrap_or(0);
            let count: i64 = row.try_get("cnt").unwrap_or(0);
            // width_bucket returns 1..=5; adjust to 0-indexed
            let idx = ((bucket_num - 1).clamp(0, 4)) as usize;
            buckets[idx].count = count;
        }

        Ok(buckets)
    }

    /// Per-day average similarity and KB-hit-rate for the last `days` days.
    /// Used by the line chart on the analytics page.
    pub async fn get_telemetry_history(
        &self,
        user_id: Uuid,
        days: i32,
    ) -> Result<Vec<crate::models::dto::TelemetryHistoryPoint>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT
                DATE(e.created_at)::text AS day,
                COALESCE(AVG(
                    CASE
                        WHEN ee.content_embedding IS NOT NULL AND ee.response_embedding IS NOT NULL
                        THEN CAST(1.0 - (ee.content_embedding <=> ee.response_embedding) AS float8)
                        ELSE NULL
                    END
                ), 0.0)::float8 AS avg_sim,
                COALESCE(
                    COUNT(ee.id)::float8 / NULLIF(COUNT(e.id)::float8, 0),
                    0.0
                )::float8 AS hit_rate
            FROM emails e
            LEFT JOIN email_embeddings ee ON e.id = ee.email_id
            WHERE e.user_id = $1
              AND e.created_at >= NOW() - ($2::int * INTERVAL '1 day')
            GROUP BY DATE(e.created_at)
            ORDER BY day ASC
            "#,
        )
        .bind(user_id)
        .bind(days)
        .fetch_all(&*self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| crate::models::dto::TelemetryHistoryPoint {
                day: r.try_get("day").unwrap_or_default(),
                avg_similarity: r.try_get("avg_sim").unwrap_or(0.0),
                kb_hit_rate: r.try_get("hit_rate").unwrap_or(0.0),
            })
            .collect())
    }

    pub async fn find_similar_emails(
        &self,
        user_id: Uuid,
        embedding: &Vector,
        limit: i64,
    ) -> Result<Vec<Email>, sqlx::Error> {
        // Find emails belonging to the user that are closest via cosine distance (`<=>` operator mapping to 1 - cosine_similarity).
        // Using the separate email_embeddings table to see vectors clearly.
        let emails = sqlx::query_as!(
            Email,
            r#"
            SELECT e.id, e.user_id, e.original_content, e.generated_response, e.created_at, e.updated_at
            FROM emails e
            JOIN email_embeddings ee ON e.id = ee.email_id
            WHERE e.user_id = $1 AND ee.content_embedding IS NOT NULL
            ORDER BY ee.content_embedding <=> $2
            LIMIT $3
            "#,
            user_id,
            embedding as &Vector,
            limit
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(emails)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dto::CreateUserRequest;
    use crate::repositories::user_repo::UserRepository;

    async fn setup_user(pool: &PgPool) -> User {
        let repo = UserRepository::new(Arc::new(pool.clone()));
        repo.create_user(&CreateUserRequest {
            email: format!("test_{}@example.com", Uuid::new_v4()),
            password: "Password123!".to_string(),
        })
        .await
        .unwrap()
    }

    #[sqlx::test]
    async fn test_create_email_and_get_paginated(pool: PgPool) {
        let user = setup_user(&pool).await;
        let repo = EmailRepository::new(Arc::new(pool));

        // No embeddings
        let email1 = repo
            .create_email(
                user.id,
                "Hello, please refund my ticket.",
                "Sure, refund processed.",
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(email1.original_content, "Hello, please refund my ticket.");
        assert_eq!(email1.user_id, user.id);

        // With embeddings
        let embedding = Vector::from(vec![0.1, 0.2, 0.3]);
        let email2 = repo
            .create_email(
                user.id,
                "Where is my order?",
                "It is shipped.",
                Some(embedding.clone()),
                Some(embedding.clone()),
            )
            .await
            .unwrap();

        assert_eq!(email2.original_content, "Where is my order?");

        // Test pagination
        let emails = repo.get_emails_paginated(0, 10).await.unwrap();
        assert!(emails.len() >= 2);

        let count = repo.get_email_count().await.unwrap();
        assert!(count >= 2);
    }

    #[sqlx::test]
    async fn test_get_user_history(pool: PgPool) {
        let user1 = setup_user(&pool).await;
        let user2 = setup_user(&pool).await;
        let repo = EmailRepository::new(Arc::new(pool));

        repo.create_email(user1.id, "User 1 mail", "Reply 1", None, None)
            .await
            .unwrap();
        repo.create_email(user2.id, "User 2 mail", "Reply 2", None, None)
            .await
            .unwrap();

        let history1 = repo
            .get_emails_by_user_paginated(user1.id, 0, 10)
            .await
            .unwrap();
        assert_eq!(history1.len(), 1);
        assert_eq!(history1[0].original_content, "User 1 mail");

        let count1 = repo.get_email_count_by_user(user1.id).await.unwrap();
        assert_eq!(count1, 1);
    }
}
