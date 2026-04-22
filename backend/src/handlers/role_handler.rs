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
    models::{domain::Role, dto::CreateRoleRequest},
    repositories::role_repo::RoleRepository,
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/admin/roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created successfully", body = Role),
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
    State(role_repo): State<Arc<RoleRepository>>,
    Json(request): Json<CreateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    let new_role = role_repo.create_role(&request.name).await?;
    Ok((StatusCode::CREATED, Json(new_role)))
}

#[utoipa::path(
    get,
    path = "/api/admin/roles",
    responses(
        (status = 200, description = "List of roles", body = Vec<Role>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Role"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_roles_handler(
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
    State(role_repo): State<Arc<RoleRepository>>,
    Path(role_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    role_repo.delete_role(role_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
