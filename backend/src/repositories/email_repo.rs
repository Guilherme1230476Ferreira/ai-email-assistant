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
    ) -> Result<Email, sqlx::Error> {
        let email = sqlx::query_as!(
            Email,
            r#"
            INSERT INTO emails (user_id, original_content, generated_response)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
            user_id,
            original_content,
            generated_response
        )
        .fetch_one(&*self.pool)
        .await?;

        Ok(email)
    }
}

