use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    app_error::AppError,
    middleware::rbac::AdminUser,
    models::dto::{
        CreateUserRequest, PaginatedResponse, PaginationParams, UpdateUserRoleRequest, UserResponse,
    },
    repositories::{
        audit_repo::AuditRepository, role_repo::RoleRepository, user_repo::UserRepository,
    },
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/admin/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Admin privileges required"),
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
    admin: AdminUser,
    State(user_repo): State<Arc<UserRepository>>,
    State(_role_repo): State<Arc<RoleRepository>>,
    State(audit_repo): State<Arc<AuditRepository>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_user = user_repo
        .create_user(&CreateUserRequest {
            email: request.email.clone(),
            password: request.password.clone(),
        })
        .await?;

    let _ = audit_repo
        .create_log(
            Some(admin.0.id),
            "create_user",
            Some(serde_json::json!({ "created_user_id": new_user.id, "email": new_user.email })),
        )
        .await;

    Ok((StatusCode::CREATED, Json(UserResponse::from(new_user))))
}

#[utoipa::path(
    get,
    path = "/api/admin/users",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of users"),
        (status = 403, description = "Admin privileges required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_users_handler(
    _admin: AdminUser,
    State(user_repo): State<Arc<UserRepository>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (offset, limit) = pagination.offset_limit();
    let users = user_repo.get_all_users_paginated(offset, limit).await?;
    let total = user_repo.get_user_count().await?;
    let response = PaginatedResponse {
        items: users.into_iter().map(UserResponse::from).collect(),
        total,
        page: pagination.page_num(),
        limit: pagination.per_page(),
    };
    Ok((StatusCode::OK, Json(response)))
}

#[utoipa::path(
    put,
    path = "/api/admin/users/{user_id}/role",
    params(
        ("user_id" = Uuid, Path, description = "User id")
    ),
    request_body = UpdateUserRoleRequest,
    responses(
        (status = 200, description = "User role updated successfully", body = UserResponse),
        (status = 400, description = "Invalid input"),
        (status = 403, description = "Admin privileges required"),
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
    admin: AdminUser,
    State(user_repo): State<Arc<UserRepository>>,
    State(role_repo): State<Arc<RoleRepository>>,
    State(audit_repo): State<Arc<AuditRepository>>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let role = role_repo
        .get_role_by_id(request.role_id)
        .await?
        .ok_or_else(|| AppError::new(StatusCode::NOT_FOUND, "Role not found"))?;
    let updated_user = user_repo.update_user_role(user_id, role.id).await?;

    let _ = audit_repo.create_log(
        Some(admin.0.id),
        "update_user_role",
        Some(serde_json::json!({ "target_user_id": user_id, "new_role_id": role.id, "new_role_name": role.name })),
    ).await;

    Ok((StatusCode::OK, Json(UserResponse::from(updated_user))))
}

#[utoipa::path(
    delete,
    path = "/api/admin/users/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User id")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 403, description = "Admin privileges required"),
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
    admin: AdminUser,
    State(user_repo): State<Arc<UserRepository>>,
    State(audit_repo): State<Arc<AuditRepository>>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    user_repo.delete_user(user_id).await?;

    let _ = audit_repo
        .create_log(
            Some(admin.0.id),
            "delete_user",
            Some(serde_json::json!({ "deleted_user_id": user_id })),
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/admin/audit-logs",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of audit logs"),
        (status = 403, description = "Admin privileges required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Admin"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_audit_logs_handler(
    _admin: AdminUser,
    State(audit_repo): State<Arc<AuditRepository>>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let logs = audit_repo
        .get_logs_paginated(pagination.page_num(), pagination.per_page())
        .await?;
    let total = audit_repo.get_logs_count().await?;
    let response = PaginatedResponse {
        items: logs,
        total,
        page: pagination.page_num(),
        limit: pagination.per_page(),
    };
    Ok((StatusCode::OK, Json(response)))
}
