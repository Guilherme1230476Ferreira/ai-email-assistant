//! # Knowledge Base Handler
//!
//! CRUD endpoints for managing the RAG knowledge base.
//! Uses the **Rig** framework (`RigRagService`) for embedding generation
//! and the **text-splitter** framework for semantic document chunking.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    app_error::AppError,
    middleware::auth::AuthUser,
    models::dto::{CreateQAPairRequest, KnowledgeEntryResponse, PaginatedResponse, PaginationParams},
    repositories::knowledge_repo::KnowledgeRepository,
    services::rig_service::RigRagService,
};

/// Extract text from a PDF file's bytes using pdf-extract
fn extract_pdf_text(bytes: &[u8]) -> Result<String, AppError> {
    pdf_extract::extract_text_from_mem(bytes).map_err(|e| {
        AppError::new(
            StatusCode::BAD_REQUEST,
            format!("Failed to extract text from PDF: {}", e),
        )
    })
}

// ── Q&A Pair Endpoint ────────────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/knowledge",
    request_body = CreateQAPairRequest,
    responses(
        (status = 201, description = "Q&A pair created", body = KnowledgeEntryResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("bearer_auth" = [])),
    tag = "Knowledge Base"
)]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn create_qa_pair_handler(
    State(knowledge_repo): State<Arc<KnowledgeRepository>>,
    State(rig_service): State<Arc<RigRagService>>,
    _auth_user: AuthUser,
    Json(request): Json<CreateQAPairRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Create the entry
    let entry = knowledge_repo
        .create_entry("qa_pair", &request.question, &request.answer)
        .await
        .map_err(|e| {
            eprintln!("Knowledge repo error: {:?}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create Q&A pair")
        })?;

    // 2. Embed the question using Rig framework and store the chunk
    let pool = knowledge_repo.get_pool();
    match rig_service
        .embed_and_store_chunk(entry.id, &request.answer, 0, &pool)
        .await
    {
        Ok(()) => {
            tracing::info!("Q&A pair embedded via Rig framework: {}", entry.id);
        }
        Err(e) => {
            tracing::warn!("Rig embedding failed for Q&A, stored without vector: {:?}", e);
            // Store chunk without embedding as fallback
            knowledge_repo.create_chunk(entry.id, &request.answer, 0, None).await.ok();
        }
    }

    let response = KnowledgeEntryResponse {
        id: entry.id,
        entry_type: entry.entry_type,
        title: entry.title,
        content: entry.content,
        chunk_count: 1,
        created_at: entry.created_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// ── Document Upload Endpoint ─────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/api/knowledge/upload",
    responses(
        (status = 201, description = "Document uploaded and embedded", body = KnowledgeEntryResponse),
        (status = 400, description = "Bad request (unsupported file type)"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("bearer_auth" = [])),
    tag = "Knowledge Base"
)]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn upload_document_handler(
    State(knowledge_repo): State<Arc<KnowledgeRepository>>,
    State(rig_service): State<Arc<RigRagService>>,
    _auth_user: AuthUser,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let mut file_name = String::from("untitled");
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut content_type = String::new();

    // Parse multipart fields
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::new(StatusCode::BAD_REQUEST, format!("Multipart error: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            file_name = field
                .file_name()
                .unwrap_or("untitled")
                .to_string();
            content_type = field
                .content_type()
                .unwrap_or("text/plain")
                .to_string();
            file_bytes = Some(field.bytes().await.map_err(|e| {
                AppError::new(StatusCode::BAD_REQUEST, format!("Failed to read file: {}", e))
            })?.to_vec());
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        AppError::new(StatusCode::BAD_REQUEST, "No file provided")
    })?;

    // Extract text based on file type
    let text = if content_type == "application/pdf" || file_name.ends_with(".pdf") {
        extract_pdf_text(&bytes)?
    } else if file_name.ends_with(".txt")
        || file_name.ends_with(".md")
        || content_type.starts_with("text/")
    {
        String::from_utf8(bytes).map_err(|_| {
            AppError::new(StatusCode::BAD_REQUEST, "File is not valid UTF-8 text")
        })?
    } else {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            "Unsupported file type. Accepted: .txt, .md, .pdf",
        ));
    };

    if text.trim().is_empty() {
        return Err(AppError::new(
            StatusCode::BAD_REQUEST,
            "File contains no extractable text",
        ));
    }

    // Create the entry
    let entry = knowledge_repo
        .create_entry("document", &file_name, &text)
        .await
        .map_err(|e| {
            eprintln!("Knowledge repo error: {:?}", e);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create document entry")
        })?;

    // Chunk text using the text-splitter framework (semantic-aware splitting)
    let chunks = RigRagService::chunk_document(&text);
    let chunk_count = chunks.len() as i64;

    // Embed each chunk using Rig framework
    let pool = knowledge_repo.get_pool();
    for (i, chunk) in chunks.iter().enumerate() {
        match rig_service
            .embed_and_store_chunk(entry.id, chunk, i as i32, &pool)
            .await
        {
            Ok(()) => {
                tracing::debug!("Chunk {} embedded via Rig for entry {}", i, entry.id);
            }
            Err(e) => {
                tracing::warn!("Rig embedding failed for chunk {}: {:?}", i, e);
                // Store chunk without embedding as fallback
                knowledge_repo.create_chunk(entry.id, chunk, i as i32, None).await.ok();
            }
        }
    }

    let response = KnowledgeEntryResponse {
        id: entry.id,
        entry_type: entry.entry_type,
        title: entry.title,
        content: entry.content,
        chunk_count,
        created_at: entry.created_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// ── List Entries ─────────────────────────────────────────────────────────────

#[utoipa::path(
    get,
    path = "/api/knowledge",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of knowledge entries"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal Server Error")
    ),
    security(("bearer_auth" = [])),
    tag = "Knowledge Base"
)]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn get_knowledge_entries_handler(
    State(knowledge_repo): State<Arc<KnowledgeRepository>>,
    _auth_user: AuthUser,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (offset, limit) = pagination.offset_limit();

    let (entries, total) = tokio::try_join!(
        async {
            knowledge_repo
                .get_entries_paginated(offset, limit)
                .await
                .map_err(|e| {
                    eprintln!("Knowledge repo error: {:?}", e);
                    AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch entries")
                })
        },
        async {
            knowledge_repo.get_entry_count().await.map_err(|e| {
                eprintln!("Knowledge count error: {:?}", e);
                AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to count entries")
            })
        }
    )?;

    let response = PaginatedResponse {
        items: entries,
        total,
        page: pagination.page_num(),
        limit: pagination.per_page(),
    };

    Ok((StatusCode::OK, Json(response)))
}

// ── Delete Entry ─────────────────────────────────────────────────────────────

#[utoipa::path(
    delete,
    path = "/api/knowledge/{id}",
    responses(
        (status = 204, description = "Entry deleted"),
        (status = 404, description = "Entry not found"),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = [])),
    tag = "Knowledge Base"
)]
#[axum::debug_handler(state = crate::state::AppState)]
pub async fn delete_knowledge_entry_handler(
    State(knowledge_repo): State<Arc<KnowledgeRepository>>,
    _auth_user: AuthUser,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let deleted = knowledge_repo.delete_entry(id).await.map_err(|e| {
        eprintln!("Knowledge delete error: {:?}", e);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete entry")
    })?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::new(StatusCode::NOT_FOUND, "Entry not found"))
    }
}
