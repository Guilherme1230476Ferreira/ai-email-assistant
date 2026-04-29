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

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_create_role(pool: PgPool) {
        let repo = RoleRepository::new(Arc::new(pool));

        let role = repo.create_role("moderator").await.unwrap();
        assert_eq!(role.name, "moderator");

        let duplicate = repo
            .create_role("moderator")
            .await
            .expect_err("Should fail");
        assert_eq!(duplicate.code(), StatusCode::CONFLICT);
    }

    #[sqlx::test]
    async fn test_get_roles(pool: PgPool) {
        let repo = RoleRepository::new(Arc::new(pool));

        // Seed migration already inserts "admin" and "user"
        let roles = repo.get_all_roles().await.unwrap();
        assert!(roles.len() >= 2);

        let admin_role = repo.get_role_by_name("admin").await.unwrap().unwrap();
        assert_eq!(admin_role.name, "admin");

        let fetched_by_id = repo.get_role_by_id(admin_role.id).await.unwrap().unwrap();
        assert_eq!(fetched_by_id.name, "admin");
    }

    #[sqlx::test]
    async fn test_delete_role(pool: PgPool) {
        let repo = RoleRepository::new(Arc::new(pool));

        let role = repo.create_role("temporary_role").await.unwrap();
        repo.delete_role(role.id).await.unwrap();

        let missing = repo.get_role_by_id(role.id).await.unwrap();
        assert!(missing.is_none());

        let err = repo
            .delete_role(Uuid::new_v4())
            .await
            .expect_err("Should return NOT_FOUND");
        assert_eq!(err.code(), StatusCode::NOT_FOUND);
    }
}
