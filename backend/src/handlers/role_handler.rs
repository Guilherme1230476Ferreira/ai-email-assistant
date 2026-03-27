use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};

use crate::{
    models::dto::{CreateRoleRequest, ErrorResponse, RoleResponse},
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/admin/roles",
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created successfully", body = RoleResponse),
        (status = 409, description = "Role with this name already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Roles"
)]
pub async fn create_role(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateRoleRequest>,
) -> Result<(StatusCode, Json<RoleResponse>), (StatusCode, Json<ErrorResponse>)> {
    // Check if a role with the same name already exists
    if let Ok(Some(_)) = sqlx::query("SELECT id FROM roles WHERE name = $1")
        .bind(&body.name)
        .fetch_optional(&state.db)
        .await
    {
        return Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "Role with this name already exists".to_string(),
            }),
        ));
    }

    // Insert the new role into the database
    let new_role = match sqlx::query_as!(
        crate::models::domain::Role,
        r#"
        INSERT INTO roles (name)
        VALUES ($1)
        RETURNING id, name
        "#,
        body.name
    )
    .fetch_one(&state.db)
    .await
    {
        Ok(role) => role,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to create role: {}", e),
                }),
            ))
        }
    };

    Ok((
        StatusCode::CREATED,
        Json(RoleResponse {
            id: new_role.id,
            name: new_role.name,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/admin/roles",
    responses(
        (status = 200, description = "List of all roles", body = Vec<RoleResponse>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "Roles"
)]
pub async fn get_roles(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<RoleResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let roles = match sqlx::query_as!(
        crate::models::domain::Role,
        r#"
        SELECT id, name
        FROM roles
        ORDER BY name
        "#,
    )
    .fetch_all(&state.db)
    .await
    {
        Ok(roles) => roles,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to retrieve roles.".to_string(),
                }),
            ))
        }
    };

    let role_responses = roles
        .into_iter()
        .map(|role| RoleResponse {
            id: role.id,
            name: role.name,
        })
        .collect();

    Ok(Json(role_responses))
}
