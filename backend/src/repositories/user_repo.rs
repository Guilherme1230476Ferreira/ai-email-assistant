use std::sync::Arc;

use axum::http::StatusCode;
use bcrypt::hash;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AppError,
    models::{domain::User, dto::CreateUserRequest},
};

#[derive(Clone)]
pub struct UserRepository {
    pool: Arc<PgPool>,
}

impl UserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, payload: &CreateUserRequest) -> Result<User, AppError> {
        let hashed_password = hash(&payload.password, 12).map_err(|_| {
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to hash password")
        })?;

        let role = sqlx::query!("SELECT id FROM roles WHERE name = 'user'")
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| {
                AppError::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Default 'user' role not found",
                )
            })?;

        let user = sqlx::query_as!(
            User,
            "INSERT INTO users (email, password_hash, role_id) VALUES ($1, $2, $3) RETURNING *",
            payload.email,
            hashed_password,
            role.id
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.is_unique_violation() {
                    return AppError::new(
                        StatusCode::CONFLICT,
                        "User with this email already exists",
                    );
                }
            }
            e.into()
        })?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, AppError> {
        sqlx::query_as!(User, "SELECT * FROM users WHERE email = $1", email)
            .fetch_optional(&*self.pool)
            .await?
            .ok_or_else(|| AppError::new(StatusCode::UNAUTHORIZED, "Invalid credentials"))
    }

    pub async fn get_user_by_id(&self, id: Uuid) -> Result<Option<User>, AppError> {
        Ok(
            sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
                .fetch_optional(&*self.pool)
                .await?,
        )
    }

    pub async fn update_user_role(&self, user_id: Uuid, role_id: Uuid) -> Result<User, AppError> {
        sqlx::query_as!(
            User,
            "UPDATE users SET role_id = $1 WHERE id = $2 RETURNING *",
            role_id,
            user_id
        )
        .fetch_one(&*self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::RowNotFound = e {
                AppError::new(StatusCode::NOT_FOUND, "User not found")
            } else {
                e.into()
            }
        })
    }

    pub async fn get_all_users(&self) -> Result<Vec<User>, AppError> {
        Ok(
            sqlx::query_as!(User, "SELECT * FROM users ORDER BY created_at DESC")
                .fetch_all(&*self.pool)
                .await?,
        )
    }

    pub async fn get_all_users_paginated(
        &self,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<User>, AppError> {
        Ok(sqlx::query_as!(
            User,
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await?)
    }

    pub async fn get_user_count(&self) -> Result<i64, AppError> {
        let record = sqlx::query!("SELECT COUNT(*) as count FROM users")
            .fetch_one(&*self.pool)
            .await?;
        Ok(record.count.unwrap_or(0))
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&*self.pool)
            .await?;

        if result.rows_affected() == 0 {
            Err(AppError::new(StatusCode::NOT_FOUND, "User not found"))
        } else {
            Ok(())
        }
    }

    pub fn get_pool(&self) -> PgPool {
        (*self.pool).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn test_create_user_success(pool: PgPool) {
        let repo = UserRepository::new(Arc::new(pool));

        let request = CreateUserRequest {
            email: "test_create@example.com".to_string(),
            password: "Password123!".to_string(),
        };

        let user = repo
            .create_user(&request)
            .await
            .expect("Failed to create user");
        assert_eq!(user.email, "test_create@example.com");

        // Assert password was hashed, not stored in plaintext
        assert_ne!(user.password_hash, "Password123!");
    }

    #[sqlx::test]
    async fn test_create_user_duplicate_email(pool: PgPool) {
        let repo = UserRepository::new(Arc::new(pool));
        let request = CreateUserRequest {
            email: "duplicate@example.com".to_string(),
            password: "Password123!".to_string(),
        };

        // First creation should succeed
        repo.create_user(&request).await.unwrap();

        // Second creation should fail with CONFLICT
        let err = repo
            .create_user(&request)
            .await
            .expect_err("Expected error on duplicate email");
        assert_eq!(err.code(), StatusCode::CONFLICT);
        assert_eq!(err.message(), "User with this email already exists");
    }

    #[sqlx::test]
    async fn test_get_user_by_email(pool: PgPool) {
        let repo = UserRepository::new(Arc::new(pool));

        let request = CreateUserRequest {
            email: "get_by_email@example.com".to_string(),
            password: "Password123!".to_string(),
        };
        repo.create_user(&request).await.unwrap();

        let user = repo
            .get_user_by_email("get_by_email@example.com")
            .await
            .expect("User should exist");
        assert_eq!(user.email, "get_by_email@example.com");

        let no_user_err = repo
            .get_user_by_email("doesnotexist@example.com")
            .await
            .expect_err("User should not exist");
        assert_eq!(no_user_err.code(), StatusCode::UNAUTHORIZED);
    }

    #[sqlx::test]
    async fn test_update_user_role(pool: PgPool) {
        let repo = UserRepository::new(Arc::new(pool));
        let request = CreateUserRequest {
            email: "update_role@example.com".to_string(),
            password: "Password123!".to_string(),
        };
        let user = repo.create_user(&request).await.unwrap();

        // Fetch admin role id to update to
        let admin_role = sqlx::query!("SELECT id FROM roles WHERE name = 'admin'")
            .fetch_one(&*repo.pool)
            .await
            .unwrap();

        assert_ne!(user.role_id, admin_role.id);

        let updated = repo.update_user_role(user.id, admin_role.id).await.unwrap();
        assert_eq!(updated.role_id, admin_role.id);
    }

    #[sqlx::test]
    async fn test_delete_user(pool: PgPool) {
        let repo = UserRepository::new(Arc::new(pool));
        let request = CreateUserRequest {
            email: "delete_me@example.com".to_string(),
            password: "Password123!".to_string(),
        };
        let user = repo.create_user(&request).await.unwrap();

        repo.delete_user(user.id)
            .await
            .expect("Failed to delete user");

        let deleted_err = repo
            .get_user_by_email("delete_me@example.com")
            .await
            .expect_err("User should be deleted");
        assert_eq!(deleted_err.code(), StatusCode::UNAUTHORIZED);

        // Deleting non-existent should return NOT_FOUND
        let err = repo
            .delete_user(Uuid::new_v4())
            .await
            .expect_err("Expected NOT_FOUND");
        assert_eq!(err.code(), StatusCode::NOT_FOUND);
    }
}
