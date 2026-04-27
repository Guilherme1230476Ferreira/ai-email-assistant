use crate::models::domain::{Email, User};
use pgvector::Vector;
use sqlx::PgPool;
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
