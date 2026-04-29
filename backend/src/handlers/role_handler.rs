use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use uuid::Uuid;

use crate::{
    app_error::AppError,
    middleware::rbac::AdminUser,
    models::{domain::Role, dto::CreateRoleRequest},
    repositories::{audit_repo::AuditRepository, role_repo::RoleRepository},
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/admin/roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created successfully", body = Role),
        (status = 403, description = "Admin privileges required"),
        (status = 409, description = "Role already exists"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Role"
)]
#[axum::debug_handler(state = AppState)]
pub async fn create_role_handler(
    admin: AdminUser,
    State(role_repo): State<Arc<RoleRepository>>,
    State(audit_repo): State<Arc<AuditRepository>>,
    Json(request): Json<CreateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_role = role_repo.create_role(&request.name).await?;

    let _ = audit_repo
        .create_log(
            Some(admin.0.id),
            "create_role",
            Some(serde_json::json!({ "role_id": new_role.id, "name": new_role.name })),
        )
        .await;

    Ok((StatusCode::CREATED, Json(new_role)))
}

#[utoipa::path(
    get,
    path = "/api/admin/roles",
    responses(
        (status = 200, description = "List of roles", body = Vec<Role>),
        (status = 403, description = "Admin privileges required"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Role"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_roles_handler(
    _admin: AdminUser,
    State(role_repo): State<Arc<RoleRepository>>,
) -> Result<impl IntoResponse, AppError> {
    let roles = role_repo.get_all_roles().await?;
    Ok((StatusCode::OK, Json(roles)))
}

#[utoipa::path(
    delete,
    path = "/api/admin/roles/{role_id}",
    params(
        ("role_id" = Uuid, Path, description = "Role id")
    ),
    responses(
        (status = 204, description = "Role deleted successfully"),
        (status = 403, description = "Admin privileges required"),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Role"
)]
#[axum::debug_handler(state = AppState)]
pub async fn delete_role_handler(
    admin: AdminUser,
    State(role_repo): State<Arc<RoleRepository>>,
    State(audit_repo): State<Arc<AuditRepository>>,
    Path(role_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    role_repo.delete_role(role_id).await?;

    let _ = audit_repo
        .create_log(
            Some(admin.0.id),
            "delete_role",
            Some(serde_json::json!({ "role_id": role_id })),
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}
