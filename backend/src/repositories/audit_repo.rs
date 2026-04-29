use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{app_error::AppError, models::domain::AuditLog};

#[derive(Clone)]
pub struct AuditRepository {
    pool: PgPool,
}

impl AuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_log(
        &self,
        user_id: Option<Uuid>,
        action: &str,
        metadata: Option<Value>,
    ) -> Result<AuditLog, AppError> {
        let log = sqlx::query_as!(
            AuditLog,
            r#"
            INSERT INTO audit_logs (user_id, action, metadata)
            VALUES ($1, $2, $3)
            RETURNING id, user_id, action, metadata, created_at
            "#,
            user_id,
            action,
            metadata
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert audit log: {}", e);
            AppError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            )
        })?;

        Ok(log)
    }

    pub async fn get_logs_paginated(
        &self,
        page: i64,
        limit: i64,
    ) -> Result<Vec<AuditLog>, AppError> {
        let offset = (page.saturating_sub(1)) * limit;

        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, user_id, action, metadata, created_at
            FROM audit_logs
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch audit logs: {}", e);
            AppError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            )
        })?;

        Ok(logs)
    }

    pub async fn get_logs_count(&self) -> Result<i64, AppError> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) FROM audit_logs
            "#
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to count audit logs: {}", e);
            AppError::new(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Database error",
            )
        })?;

        Ok(count.unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::Arc;

    #[sqlx::test]
    async fn test_create_and_get_logs(pool: PgPool) {
        let repo = AuditRepository::new(pool.clone());

        // Let's test with None for user_id to avoid creating a user.

        let log = repo
            .create_log(None, "test_action", Some(json!({"key": "value"})))
            .await
            .unwrap();
        assert_eq!(log.action, "test_action");
        assert_eq!(
            log.metadata.unwrap().get("key").unwrap().as_str().unwrap(),
            "value"
        );

        let logs = repo.get_logs_paginated(1, 10).await.unwrap();
        assert!(logs.len() >= 1);

        let count = repo.get_logs_count().await.unwrap();
        assert!(count >= 1);
    }

    #[sqlx::test]
    async fn test_create_log_with_user(pool: PgPool) {
        let repo = AuditRepository::new(pool.clone());
        let user_repo = crate::repositories::user_repo::UserRepository::new(Arc::new(pool));

        let user = user_repo
            .create_user(&crate::models::dto::CreateUserRequest {
                email: "audit_user@example.com".to_string(),
                password: "Password123!".to_string(),
            })
            .await
            .unwrap();

        let log = repo
            .create_log(Some(user.id), "user_action", None)
            .await
            .unwrap();
        assert_eq!(log.user_id, Some(user.id));
        assert_eq!(log.action, "user_action");
    }
}
