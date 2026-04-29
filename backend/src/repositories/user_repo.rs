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
}
