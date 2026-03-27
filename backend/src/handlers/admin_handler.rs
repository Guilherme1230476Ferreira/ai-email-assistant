/// Admin handlers — HTTP endpoints for backoffice operations.
/// Real implementations added in Sprint 3.
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    models::dto::{
        ApiKeyResponse, CreateUserRequest, ErrorResponse, UpdateApiKeyRequest, UpdateUserRoleRequest,
        UserResponse,
    },
    services::api_key_validator,
    state::AppState,
};

#[utoipa::path(
    get,
    path = "/api/admin/llm-provider-api-key",
    responses(
        (status = 200, description = "Check if an API key is configured", body = ApiKeyResponse)
    ),
    tag = "Admin"
)]
pub async fn get_api_key(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiKeyResponse>, (StatusCode, Json<ErrorResponse>)> {
    let settings = state.settings.read().await;

    Ok(Json(ApiKeyResponse {
        has_api_key: settings.api_key.is_some(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/admin/llm-provider-api-key",
    request_body = UpdateApiKeyRequest,
    responses(
        (status = 200, description = "API key updated successfully", body = ApiKeyResponse),
        (status = 401, description = "Invalid API key provided", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn update_api_key(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UpdateApiKeyRequest>,
) -> Result<Json<ApiKeyResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mut settings = state.settings.write().await;

    if let Some(api_key) = body.api_key.as_deref() {
        if api_key.trim().is_empty() {
            settings.api_key = None;
        } else if let Err(error) =
            api_key_validator::validate_api_key(api_key, &state.http_client).await
        {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: format!("api_key validation failed: {error}"),
                }),
            ));
        } else {
            settings.api_key = Some(api_key.to_string());
        }
    } else {
        // If api_key is None in the request, clear it in the settings
        settings.api_key = None;
    }

    Ok(Json(ApiKeyResponse {
        has_api_key: settings.api_key.is_some(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/admin/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 409, description = "User with this email already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Basic validation
    if body.email.is_empty() || body.password.len() < 8 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Email cannot be empty and password must be at least 8 characters long."
                    .to_string(),
            }),
        ));
    }

    // Hash the password
    let password_hash = match bcrypt::hash(&body.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to hash password.".to_string(),
                }),
            ))
        }
    };

    // Get the ID for the default 'user' role
    let user_role_id = match sqlx::query_scalar::<_, Uuid>("SELECT id FROM roles WHERE name = 'user'")
        .fetch_one(&state.db)
        .await
    {
        Ok(id) => id,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Default 'user' role not found in database.".to_string(),
                }),
            ))
        }
    };

    // Insert user into the database
    let new_user = match sqlx::query_as!(
        crate::models::domain::User,
        r#"
        INSERT INTO users (email, password_hash, role_id)
        VALUES ($1, $2, $3)
        RETURNING id, email, password_hash, role_id, created_at
        "#,
        body.email,
        password_hash,
        user_role_id
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(user) => user,
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => {
            return Err((
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "A user with this email already exists.".to_string(),
                }),
            ));
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to create user.".to_string(),
                }),
            ))
        }
    };

    Ok((
        StatusCode::CREATED,
        Json(UserResponse {
            id: new_user.id,
            email: new_user.email,
            role_id: new_user.role_id,
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/api/admin/users/{id}/role",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRoleRequest,
    responses(
        (status = 200, description = "User role updated successfully", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn update_user_role(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateUserRoleRequest>,
) -> Result<Json<UserResponse>, (StatusCode, Json<ErrorResponse>)> {
    let updated_user = match sqlx::query_as!(
        crate::models::domain::User,
        r#"
        UPDATE users
        SET role_id = $1
        WHERE id = $2
        RETURNING id, email, password_hash, role_id, created_at
        "#,
        body.role_id,
        id
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(user) => user,
        Err(sqlx::Error::RowNotFound) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found.".to_string(),
                }),
            ));
        }
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to update user role.".to_string(),
                }),
            ))
        }
    };

    Ok(Json(UserResponse {
        id: updated_user.id,
        email: updated_user.email,
        role_id: updated_user.role_id,
    }))
}

#[utoipa::path(
    get,
    path = "/api/admin/users",
    responses(
        (status = 200, description = "List of all users", body = Vec<UserResponse>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Admin"
)]
pub async fn get_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let users = match sqlx::query_as!(
        crate::models::domain::User,
        r#"
        SELECT id, email, password_hash, role_id, created_at
        FROM users
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(users) => users,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to retrieve users.".to_string(),
                }),
            ))
        }
    };

    let user_responses = users
        .into_iter()
        .map(|user| UserResponse {
            id: user.id,
            email: user.email,
            role_id: user.role_id,
        })
        .collect();

    Ok(Json(user_responses))
}

