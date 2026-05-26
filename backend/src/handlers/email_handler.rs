use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::StreamExt;

use crate::{
    app_error::AppError,
    infrastructure::config::Config,
    middleware::auth::AuthUser,
    models::{
        domain::Email,
        dto::{GenerateEmailRequest, PaginatedResponse, PaginationParams},
    },
    repositories::{
        audit_repo::AuditRepository,
        email_repo::EmailRepository,
        knowledge_repo::KnowledgeRepository,
        settings_repo::SettingsRepository,
    },
    services::llm_service::LlmService,
    services::rig_service::RigRagService,
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/api/emails/generate",
    request_body = GenerateEmailRequest,
    responses(
        (status = 200, description = "Email generated successfully", body = Email),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Email"
)]
#[axum::debug_handler(state = AppState)]
pub async fn generate_email_handler(
    State(settings_repo): State<Arc<SettingsRepository>>,
    State(llm_service): State<Arc<dyn LlmService + Send + Sync>>,
    State(email_repo): State<Arc<EmailRepository>>,
    State(rig_service): State<Arc<RigRagService>>,
    State(_config): State<Arc<Config>>,
    auth_user: AuthUser,
    Json(request): Json<GenerateEmailRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Chat/Generation Configurations from DB
    let settings = settings_repo.get_settings().await?;
    let api_key = settings_repo
        .get_decrypted_api_key()
        .await?
        .ok_or_else(|| AppError::new(StatusCode::BAD_REQUEST, "LLM API Key is not configured."))?;

    // 2. Rig-powered RAG: embed query → retrieve from knowledge base + past emails
    let mut context_str = String::new();
    let mut prompt_embedding = None;

    match rig_service.vector_index.embed_text(&request.prompt).await {
        Ok(embedding_floats) => {
            let vec = pgvector::Vector::from(embedding_floats.clone());

            // Use Rig's unified context search (knowledge base + past emails)
            if let Ok(results) = rig_service.vector_index
                .search_all_context(&embedding_floats, auth_user.0.id, 5)
                .await
            {
                for (i, ctx) in results.iter().enumerate() {
                    context_str.push_str(&format!(
                        "--- Context #{} [{}] (relevance: {:.2}):\n{}\n\n",
                        i + 1, ctx.title, ctx.score, ctx.text
                    ));
                }
            }

            prompt_embedding = Some(vec);
        }
        Err(e) => {
            eprintln!(
                "Warning: Rig embedding failed, proceeding without RAG context: {:?}",
                e
            );
        }
    }

    // 3. Create the dummy Email entity
    let email_to_generate = Email {
        id: uuid::Uuid::new_v4(),
        user_id: auth_user.0.id,
        original_content: request.prompt.clone(),
        generated_response: Some("".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let generated_response = llm_service
        .generate_reply(
            &email_to_generate,
            &context_str,
            &settings.llm_base_url,
            &settings.llm_model,
            &api_key,
        )
        .await
        .map_err(|e| {
            eprintln!("LLM service error: {:?}", e);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate email response".to_string(),
            )
        })?;

    // 4. Fetch Embedding for the LLM's Generated Response using Rig
    let response_embedding = match rig_service.embed_for_storage(&generated_response).await {
        Ok(vec) => Some(vec),
        Err(e) => {
            eprintln!(
                "Warning: Rig failed to embed generated response: {:?}",
                e
            );
            None
        }
    };

    let email = email_repo
        .create_email(
            auth_user.0.id,
            &request.prompt,
            &generated_response,
            prompt_embedding,
            response_embedding,
        )
        .await
        .map_err(|e| {
            eprintln!("Email repo error: {:?}", e);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to save generated email".to_string(),
            )
        })?;

    Ok((StatusCode::OK, Json(email)))
}

// SSE Streaming Endpoint

#[utoipa::path(
    post,
    path = "/api/emails/generate/stream",
    request_body = GenerateEmailRequest,
    responses(
        (status = 200, description = "SSE stream of generated email tokens"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Email"
)]
#[axum::debug_handler(state = AppState)]
pub async fn generate_email_stream_handler(
    State(settings_repo): State<Arc<SettingsRepository>>,
    State(llm_service): State<Arc<dyn LlmService + Send + Sync>>,
    State(email_repo): State<Arc<EmailRepository>>,
    State(knowledge_repo): State<Arc<KnowledgeRepository>>,
    State(rig_service): State<Arc<RigRagService>>,
    State(_config): State<Arc<Config>>,
    auth_user: AuthUser,
    Json(request): Json<GenerateEmailRequest>,
) -> Result<
    axum::response::sse::Sse<
        impl futures_util::Stream<Item = Result<axum::response::sse::Event, std::convert::Infallible>>,
    >,
    AppError,
> {
    // Pre-fetch settings + API key before the stream
    let settings = settings_repo.get_settings().await?;
    let api_key = settings_repo
        .get_decrypted_api_key()
        .await?
        .ok_or_else(|| AppError::new(StatusCode::BAD_REQUEST, "LLM API Key is not configured."))?;

    let user_id = auth_user.0.id;
    let prompt = request.prompt.clone();
    let model_name = settings.llm_model.clone();

    let sse_stream = async_stream::stream! {
        // Stage 1: Embedding
        yield Ok::<_, std::convert::Infallible>(
            axum::response::sse::Event::default().event("log").data("> [1/4] Embedding prompt with vector model...")
        );

        let mut context_str = String::new();
        let mut prompt_embedding = None;
        let mut kb_count: usize = 0;
        let mut email_count: usize = 0;

        match rig_service.vector_index.embed_text(&prompt).await {
            Ok(embedding_floats) => {
                let embed_dim = embedding_floats.len();
                yield Ok(axum::response::sse::Event::default()
                    .event("log")
                    .data(format!("> [1/4] Embedding ready ({} dims)", embed_dim)));

                let vec = pgvector::Vector::from(embedding_floats.clone());

                // Stage 2: RAG search
                yield Ok(axum::response::sse::Event::default()
                    .event("log")
                    .data("> [2/4] Searching knowledge base + past emails..."));

                match rig_service.vector_index
                    .search_all_context(&embedding_floats, user_id, 5)
                    .await
                {
                    Ok(results) => {
                        let mut total_score = 0.0f64;
                        // Collect distinct entry_ids for retrieval count tracking
                        let mut seen_entry_ids: std::collections::HashSet<uuid::Uuid> =
                            std::collections::HashSet::new();
                        for ctx in &results {
                            if ctx.title == "Past Email" {
                                email_count += 1;
                            } else {
                                kb_count += 1;
                                if let Some(eid) = ctx.entry_id {
                                    seen_entry_ids.insert(eid);
                                }
                            }
                            total_score += ctx.score;
                            context_str.push_str(&format!(
                                "--- Context [{}] (relevance: {:.2}):\n{}\n\n",
                                ctx.title, ctx.score, ctx.text
                            ));
                        }
                        let avg_score = if !results.is_empty() {
                            total_score / results.len() as f64
                        } else {
                            0.0
                        };
                        yield Ok(axum::response::sse::Event::default()
                            .event("log")
                            .data(format!(
                                "> [2/4] Retrieved {} KB chunk{}, {} past email{} (avg similarity: {:.2})",
                                kb_count,
                                if kb_count != 1 { "s" } else { "" },
                                email_count,
                                if email_count != 1 { "s" } else { "" },
                                avg_score
                            )));
                        // Fire-and-forget: increment retrieval counts for hit KB entries
                        if !seen_entry_ids.is_empty() {
                            let ids: Vec<uuid::Uuid> = seen_entry_ids.into_iter().collect();
                            let _ = knowledge_repo.increment_retrieval_counts(&ids).await;
                        }
                    }
                    Err(e) => {
                        yield Ok(axum::response::sse::Event::default()
                            .event("log")
                            .data(format!("> [2/4] Warning: vector search failed ({})", e.message())));
                    }
                }
                prompt_embedding = Some(vec);
            }
            Err(e) => {
                yield Ok(axum::response::sse::Event::default()
                    .event("log")
                    .data(format!("> [1/4] Warning: embedding failed ({}) -- proceeding without context", e.message())));
            }
        }

        // Stage 3: LLM call
        yield Ok(axum::response::sse::Event::default()
            .event("log")
            .data(format!("> [3/4] Calling {} ({} context chars)...",
                model_name,
                context_str.len()
            )));

        let email_to_generate = crate::models::domain::Email {
            id: uuid::Uuid::new_v4(),
            user_id,
            original_content: prompt.clone(),
            generated_response: Some(String::new()),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let token_stream = match llm_service
            .generate_reply_stream(
                &email_to_generate,
                &context_str,
                &settings.llm_base_url,
                &settings.llm_model,
                &api_key,
            )
            .await
        {
            Ok(s) => s,
            Err(e) => {
                yield Ok(axum::response::sse::Event::default()
                    .event("error")
                    .data(format!("Failed to start LLM stream: {:?}", e)));
                return;
            }
        };

        // Stage 4: Stream tokens
        yield Ok(axum::response::sse::Event::default()
            .event("log")
            .data("> [4/4] Streaming reply:"));

        let mut full_response = String::new();
        futures_util::pin_mut!(token_stream);

        while let Some(chunk_result) = token_stream.next().await {
            match chunk_result {
                Ok(token) => {
                    full_response.push_str(&token);
                    yield Ok(axum::response::sse::Event::default()
                        .event("token")
                        .data(token));
                }
                Err(e) => {
                    yield Ok(axum::response::sse::Event::default()
                        .event("error")
                        .data(format!("{}", e.message())));
                    return;
                }
            }
        }

        // Embed response + save to DB
        let resp_embedding = match rig_service.embed_for_storage(&full_response).await {
            Ok(v) => Some(v),
            Err(_) => None,
        };

        match email_repo
            .create_email(user_id, &prompt, &full_response, prompt_embedding, resp_embedding)
            .await
        {
            Ok(email) => {
                yield Ok(axum::response::sse::Event::default()
                    .event("log")
                    .data(format!("> Generation complete. Saved as #{}.", &email.id.to_string()[..8])));
                let done_data = serde_json::json!({ "id": email.id.to_string() }).to_string();
                yield Ok(axum::response::sse::Event::default()
                    .event("done")
                    .data(done_data));
            }
            Err(e) => {
                yield Ok(axum::response::sse::Event::default()
                    .event("error")
                    .data(format!("Failed to save email: {:?}", e)));
            }
        }
    };

    Ok(axum::response::sse::Sse::new(sse_stream))
}

// ─── Delete Email ────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/emails/{id}",
    responses(
        (status = 204, description = "Email deleted"),
        (status = 404, description = "Email not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = [])),
    tag = "Email"
)]
#[axum::debug_handler(state = AppState)]
pub async fn delete_email_handler(
    State(email_repo): State<Arc<EmailRepository>>,
    State(audit_repo): State<Arc<AuditRepository>>,
    auth_user: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let deleted = email_repo
        .delete_email(id, auth_user.0.id)
        .await
        .map_err(|e| {
            tracing::error!("Email delete error: {:?}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete email")
        })?;

    if deleted {
        let _ = audit_repo.create_log(
            Some(auth_user.0.id),
            "email.delete",
            Some(serde_json::json!({ "email_id": id })),
        ).await;

        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::new(StatusCode::NOT_FOUND, "Email not found"))
    }
}

// ─── Telemetry ───────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/telemetry",
    responses(
        (status = 200, description = "RAG execution telemetry", body = TelemetryData),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Email"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_telemetry_handler(
    State(email_repo): State<Arc<EmailRepository>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let telemetry = email_repo
        .get_telemetry_for_user(auth_user.0.id)
        .await
        .map_err(|e| {
            eprintln!("Telemetry error: {:?}", e);
            AppError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to compute telemetry".to_string(),
            )
        })?;

    Ok((StatusCode::OK, Json(telemetry)))
}

// ─── Paginated Email List ────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/emails",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of user emails"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "Email"
)]
#[axum::debug_handler(state = AppState)]
pub async fn get_emails_handler(
    State(email_repo): State<Arc<EmailRepository>>,
    auth_user: AuthUser,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (offset, limit) = pagination.offset_limit();

    let (emails, total) = tokio::try_join!(
        async {
            email_repo
                .get_emails_by_user_paginated(auth_user.0.id, offset, limit)
                .await
                .map_err(|e| {
                    eprintln!("Email repo error: {:?}", e);
                    AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch emails")
                })
        },
        async {
            email_repo
                .get_email_count_by_user(auth_user.0.id)
                .await
                .map_err(|e| {
                    eprintln!("Email count error: {:?}", e);
                    AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to count emails")
                })
        }
    )?;

    let response = PaginatedResponse {
        items: emails,
        total,
        page: pagination.page_num(),
        limit: pagination.per_page(),
    };

    Ok((StatusCode::OK, Json(response)))
}

/// GET /api/telemetry/distribution
/// Returns similarity score histogram (5 buckets, 0.0–1.0) for the authenticated user.
#[axum::debug_handler(state = AppState)]
pub async fn get_similarity_distribution_handler(
    State(email_repo): State<Arc<EmailRepository>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let buckets = email_repo
        .get_similarity_distribution(auth_user.0.id)
        .await
        .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?;
    Ok((StatusCode::OK, Json(buckets)))
}

/// GET /api/telemetry/history
/// Returns per-day avg_similarity + kb_hit_rate for the last 30 days.
#[axum::debug_handler(state = AppState)]
pub async fn get_telemetry_history_handler(
    State(email_repo): State<Arc<EmailRepository>>,
    auth_user: AuthUser,
) -> Result<impl IntoResponse, AppError> {
    let history = email_repo
        .get_telemetry_history(auth_user.0.id, 30)
        .await
        .map_err(|e| AppError::new(StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e)))?;
    Ok((StatusCode::OK, Json(history)))
}
