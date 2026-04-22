use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{
    app_error::AppError,
    models::{
        domain::User,
        dto::{CreateUserRequest, UpdateUserRoleRequest},
    },
    repositories::{role_repo::RoleRepository, user_repo::UserRepository},
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/admin/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 400, description = "Invalid input"),
        (status = 409, description = "User already exists"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin"
)]
#[axum::debug_handler(state = AppState)]
pub async fn create_user_handler(
    State(user_repo): State<Arc<UserRepository>>,
    State(role_repo): State<Arc<RoleRepository>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let role_name = "user".to_string(); // TODO: Get role from request
    let role = role_repo.get_role_by_name(&role_name).await?;

    let new_user = user_repo
        .create_user(&CreateUserRequest {
            email: request.email.clone(),
            password: request.password.clone(),
        })
        .await?;

    Ok((StatusCode::CREATED, Json(new_user)))
}

#[utoipa::path(
    get,
    path = "/api/admin/users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_users_handler(
    State(user_repo): State<Arc<UserRepository>>,
) -> Result<impl IntoResponse, AppError> {
    let users = user_repo.get_all_users().await?;
    Ok((StatusCode::OK, Json(users)))
}

#[utoipa::path(
    put,
    path = "/api/admin/users/{user_id}/role",
    params(
        ("user_id" = Uuid, Path, description = "User id")
    ),
    request_body = UpdateUserRoleRequest,
    responses(
        (status = 200, description = "User role updated successfully", body = User),
        (status = 400, description = "Invalid input"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin"
)]
#[axum::debug_handler(state = AppState)]
pub async fn update_user_role_handler(
    State(user_repo): State<Arc<UserRepository>>,
    State(role_repo): State<Arc<RoleRepository>>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let role = role_repo.get_role_by_id(request.role_id).await?;
    let updated_user = user_repo.update_user_role(user_id, role.unwrap().id).await?;
    Ok((StatusCode::OK, Json(updated_user)))
}

#[utoipa::path(
    delete,
    path = "/api/admin/users/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User id")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin"
)]
#[axum::debug_handler(state = AppState)]
pub async fn delete_user_handler(
    State(user_repo): State<Arc<UserRepository>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    user_repo.delete_user(user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
