use std::sync::Arc;

use axum::http::StatusCode;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{AppError, models::domain::Role};

#[derive(Clone)]
pub struct RoleRepository {
    pool: Arc<PgPool>,
}

impl RoleRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_role(&self, name: &str) -> Result<Role, AppError> {
        sqlx::query_as!(
            Role,
            "INSERT INTO roles (name) VALUES ($1) RETURNING *",
            name
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return AppError::new(
                        StatusCode::CONFLICT,
                        "Role with this name already exists",
                    );
                }
            }
            e.into()
        })
    }

    pub async fn get_all_roles(&self) -> Result<Vec<Role>, AppError> {
        Ok(sqlx::query_as!(Role, "SELECT * FROM roles ORDER BY name")
            .fetch_all(&*self.pool)
            .await?)
    }

    pub async fn get_role_by_name(&self, name: &str) -> Result<Option<Role>, AppError> {
        Ok(
            sqlx::query_as!(Role, "SELECT * FROM roles WHERE name = $1", name)
                .fetch_optional(&*self.pool)
                .await?,
        )
    }

    pub async fn get_role_by_id(&self, role_id: Uuid) -> Result<Option<Role>, AppError> {
        let role = sqlx::query_as!(Role, "SELECT id, name FROM roles WHERE id = $1", role_id,)
            .fetch_optional(&*self.pool)
            .await?;
        Ok(role)
    }

    pub async fn delete_role(&self, role_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM roles WHERE id = $1", role_id)
            .execute(&*self.pool)
            .await?;

        if result.rows_affected() == 0 {
            Err(AppError::new(StatusCode::NOT_FOUND, "Role not found"))
        } else {
            Ok(())
        }
    }
}
