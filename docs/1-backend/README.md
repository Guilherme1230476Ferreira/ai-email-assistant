# Backend Technical Documentation

## 1. Overview

The MailMate backend is a **Rust** application built with the **Axum** web framework, running on the **Tokio** async runtime. It serves as the API layer for the backoffice dashboard and the Chrome extension, providing authentication, email generation with RAG, knowledge base management, and administrative features.

**Entry point**: `src/main.rs`  
**Library root**: `src/lib.rs`  
**Base URL**: `http://localhost:3000` (dev) / proxied via Nginx in production

---

## 2. Project Structure

```
backend/
├── migrations/              # SQLx database migrations (9 files)
├── src/
│   ├── main.rs              # Server bootstrap, tracing, graceful shutdown
│   ├── lib.rs               # Crate root — re-exports all modules
│   ├── router.rs            # Axum router definition + OpenAPI/Swagger setup
│   ├── state.rs             # AppState struct — shared across all handlers
│   ├── app_error.rs         # Unified error type implementing IntoResponse
│   ├── handlers/            # HTTP request handlers (one file per domain)
│   │   ├── auth_handler.rs      # Login, register, Google OAuth, /me
│   │   ├── admin_handler.rs     # User CRUD (admin only)
│   │   ├── role_handler.rs      # Role CRUD (admin only)
│   │   ├── email_handler.rs     # Email generation (streaming + non-streaming), delete, telemetry
│   │   ├── knowledge_handler.rs # Knowledge base: Q&A pairs, document upload, listing, deletion
│   │   └── settings_handler.rs  # LLM settings management
│   ├── middleware/           # Axum middleware layers
│   │   ├── auth.rs              # JWT validation, AuthUser extractor
│   │   ├── rbac.rs              # Role-based access control guard
│   │   └── rate_limiter.rs      # IP-based rate limiting
│   ├── models/               # Domain entities and DTOs
│   │   ├── domain.rs            # Database entities: User, Role, Email, AppSetting, KnowledgeEntry, etc.
│   │   ├── dto.rs               # Request/response DTOs with validation
│   │   └── settings_dto.rs      # Settings-specific DTOs
│   ├── repositories/         # Database access layer (one repo per entity)
│   │   ├── user_repo.rs         # User CRUD with bcrypt hashing
│   │   ├── role_repo.rs         # Role CRUD
│   │   ├── email_repo.rs        # Email CRUD, embedding storage, telemetry, pagination
│   │   ├── knowledge_repo.rs    # Knowledge entries + embeddings CRUD
│   │   ├── settings_repo.rs     # Singleton settings with encrypted API keys
│   │   └── audit_repo.rs        # Immutable audit log storage
│   ├── services/             # Business logic services
│   │   ├── llm_service.rs       # Unified LLM client (OpenAI, Gemini, Groq)
│   │   └── rig_service.rs       # Rig RAG framework integration
│   └── infrastructure/       # Cross-cutting concerns
│       ├── config.rs            # Environment variable loading (.env)
│       └── crypto.rs            # AES-256-GCM encryption/decryption
└── tests/                    # Integration tests
    ├── common/mod.rs            # Shared test utilities (AppState builder, HTTP helpers)
    ├── auth_tests.rs            # Authentication integration tests
    ├── admin_users_tests.rs     # User management tests
    ├── role_tests.rs            # Role management tests
    ├── email_tests.rs           # Email generation tests
    ├── settings_tests.rs        # Settings management tests
    ├── audit_tests.rs           # Audit log tests
    └── infrastructure_tests.rs  # Crypto service tests
```

---

## 3. Application State (`state.rs`)

All shared dependencies are encapsulated in `AppState`, which implements `Clone` and is passed to every handler via Axum's `State` extractor.

| Field | Type | Purpose |
|---|---|---|
| `user_repo` | `Arc<UserRepository>` | User CRUD operations |
| `role_repo` | `Arc<RoleRepository>` | Role CRUD operations |
| `email_repo` | `Arc<EmailRepository>` | Email storage and retrieval |
| `knowledge_repo` | `Arc<KnowledgeRepository>` | Knowledge base CRUD |
| `settings_repo` | `Arc<SettingsRepository>` | LLM configuration management |
| `audit_repo` | `Arc<AuditRepository>` | Audit log storage |
| `crypto_service` | `Arc<CryptoService>` | AES-256-GCM encryption |
| `llm_service` | `Arc<dyn LlmService>` | LLM provider abstraction |
| `rig_service` | `Arc<RigRagService>` | Rig-powered RAG pipeline |
| `config` | `Arc<Config>` | Environment configuration |
| `rate_limiters` | `Arc<RwLock<HashMap>>` | Per-IP rate limiting state |

Each field implements `FromRef<AppState>` for granular state extraction in handlers.

---

## 4. API Endpoints

### Authentication
| Method | Path | Auth | Description |
|---|---|---|---|
| `POST` | `/api/auth/login` | ❌ | Login with email/password, returns JWT |
| `POST` | `/api/auth/register` | ❌ | Register a new user account |
| `GET` | `/api/auth/me` | ✅ | Get current authenticated user info |
| `GET` | `/api/auth/google` | ❌ | Initiate Google OAuth2 flow |
| `GET` | `/api/auth/google/callback` | ❌ | Google OAuth2 callback handler |

### Email Generation
| Method | Path | Auth | Description |
|---|---|---|---|
| `POST` | `/api/emails/generate` | ✅ | Generate email reply (JSON response) |
| `POST` | `/api/emails/generate/stream` | ✅ | Generate email reply (SSE streaming) |
| `GET` | `/api/emails` | ✅ | List user's emails (paginated) |
| `DELETE` | `/api/emails/:id` | ✅ | Delete a specific email |
| `GET` | `/api/telemetry` | ✅ | RAG execution telemetry metrics |

### Knowledge Base
| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/api/knowledge` | ✅ Admin | List all knowledge entries |
| `POST` | `/api/knowledge` | ✅ Admin | Create a Q&A knowledge pair |
| `POST` | `/api/knowledge/upload` | ✅ Admin | Upload document (PDF/TXT/MD) |
| `DELETE` | `/api/knowledge/:id` | ✅ Admin | Delete a knowledge entry |

### Administration (Admin Only)
| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/api/admin/users` | ✅ Admin | List all users (paginated) |
| `POST` | `/api/admin/users` | ✅ Admin | Create a new user |
| `DELETE` | `/api/admin/users/:user_id` | ✅ Admin | Delete a user |
| `PUT` | `/api/admin/users/:user_id/role` | ✅ Admin | Update a user's role |
| `GET` | `/api/admin/roles` | ✅ Admin | List all roles |
| `POST` | `/api/admin/roles` | ✅ Admin | Create a new role |
| `DELETE` | `/api/admin/roles/:role_id` | ✅ Admin | Delete a role |
| `GET` | `/api/admin/audit-logs` | ✅ Admin | List audit logs (paginated) |

### Settings
| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/api/admin/settings` | ✅ Admin | Get current LLM configuration |
| `PUT` | `/api/admin/settings` | ✅ Admin | Update LLM configuration |
| `POST` | `/api/admin/settings/verify` | ✅ Admin | Test LLM connection |

### System
| Method | Path | Auth | Description |
|---|---|---|---|
| `GET` | `/health` | ❌ | Health check endpoint |
| `GET` | `/swagger-ui` | ❌ | Interactive API documentation |

---

## 5. RAG Pipeline (Rig Framework)

### Architecture

The RAG pipeline is powered by **rig-core v0.36**, a Rust-native LLM/RAG framework. The integration uses a custom bridge pattern to connect Rig to the existing pgvector infrastructure.

#### Key Components (`services/rig_service.rs`)

| Component | Description |
|---|---|
| `KnowledgeDocument` | Implements Rig's `Embed` trait — tells Rig which text fields to vectorize |
| `PgVectorIndex` | Custom bridge between Rig and pgvector tables — dual search across knowledge + email embeddings |
| `RigRagService` | Top-level orchestration — embedding, retrieval, and storage in a single service |

#### RAG Flow
1. **Embedding**: Incoming text is vectorized using Rig's `EmbeddingsBuilder` with the Gemini embedding API
2. **Retrieval**: `PgVectorIndex.search_all_context()` performs cosine similarity search across:
   - `knowledge_embeddings` — global knowledge base (Q&A pairs, documents)
   - `email_embeddings` — user-specific past email history
3. **Ranking**: Results from both sources are merged and ranked by similarity score
4. **Generation**: Top-K context is injected into the system prompt alongside the original email
5. **Streaming**: LLM response is streamed back via Server-Sent Events (SSE)
6. **Indexing**: Both the prompt and response embeddings are stored for future retrieval

### Knowledge Ingestion

#### Q&A Pairs
- Admin submits a question + answer pair
- Combined text is embedded via Rig's `EmbeddingsBuilder`
- Stored in `knowledge_entries` (type: `qa_pair`) + `knowledge_embeddings`

#### Document Upload
- Supports PDF, TXT, and Markdown files (max 10MB)
- PDF text extraction via `pdf-extract` crate
- Documents are split into semantic chunks using `text-splitter` with a 500-character target size
- Each chunk is individually embedded and stored

---

## 6. Authentication & Security

### JWT Authentication (`middleware/auth.rs`)
- Tokens are created with `jsonwebtoken` using HS256 algorithm
- Payload contains `user_id` (UUID) and `exp` (expiration timestamp)
- `AuthUser` extractor validates the token on every protected request
- Token expiration is configurable via `JWT_EXPIRATION_HOURS` env var

### Role-Based Access Control (`middleware/rbac.rs`)
- Two default roles: `admin` and `user`
- Admin-only endpoints are protected at the handler level
- Role is resolved from `users.role_id → roles.name`

### Rate Limiting (`middleware/rate_limiter.rs`)
- IP-based sliding window rate limiter
- Applied to `/api/auth/login` to prevent brute-force attacks
- Configurable window size and max requests

### Encryption (`infrastructure/crypto.rs`)
- AES-256-GCM authenticated encryption
- Used to encrypt LLM API keys before database storage
- Unique 96-bit nonce generated for each encryption operation
- Encryption key loaded from `ENCRYPTION_KEY` environment variable

---

## 7. Database Schema

### Tables
| Table | Description |
|---|---|
| `roles` | Role definitions (admin, user) |
| `users` | User accounts with bcrypt password hashes |
| `emails` | Generated email records per user |
| `email_embeddings` | Vector embeddings for prompt/response similarity search |
| `settings` | Singleton LLM configuration (encrypted API key) |
| `audit_logs` | Immutable action log with JSONB metadata |
| `knowledge_entries` | Knowledge base entries (Q&A pairs, documents) |
| `knowledge_chunks` | Document chunks after text splitting |
| `knowledge_embeddings` | Vector embeddings for knowledge retrieval |

### Migrations
Migrations are managed by SQLx and stored in `backend/migrations/`. They are versioned with timestamps and applied automatically.

---

## 8. Environment Variables

| Variable | Required | Description |
|---|---|---|
| `DATABASE_URL` | ✅ | PostgreSQL connection string |
| `JWT_SECRET` | ✅ | Secret key for JWT signing |
| `JWT_EXPIRATION_HOURS` | ❌ | Token lifetime (default: 24) |
| `ENCRYPTION_KEY` | ✅ | AES-256-GCM encryption key |
| `EMBEDDING_API_URL` | ✅ | Embedding API base URL (Gemini) |
| `EMBEDDING_API_KEY` | ✅ | Embedding API key |
| `EMBEDDING_MODEL` | ✅ | Embedding model name |
| `PORT` | ❌ | Server port (default: 3000) |
| `GOOGLE_CLIENT_ID` | ❌ | Google OAuth2 client ID |
| `GOOGLE_CLIENT_SECRET` | ❌ | Google OAuth2 client secret |
| `PUBLIC_URL` | ❌ | Public-facing URL for OAuth redirects |

---

## 9. Testing

### Unit Tests
Located within source files using `#[cfg(test)]` modules. Cover:
- `app_error` — Error type construction
- `crypto` — Encryption/decryption roundtrip, error cases
- `dto` — Pagination parameter validation
- `email_repo` — Email CRUD operations (with `#[sqlx::test]`)
- `role_repo` — Role CRUD operations

### Integration Tests
Located in `tests/` directory. Each test file uses `#[sqlx::test]` which:
1. Creates an isolated temporary database
2. Runs all migrations
3. Executes the test
4. Drops the database

Test suites: `auth_tests`, `admin_users_tests`, `role_tests`, `email_tests`, `settings_tests`, `audit_tests`, `infrastructure_tests`

### Running Tests
```bash
# Requires a live PostgreSQL connection
cargo test

# Check compilation without running (uses .sqlx offline cache)
SQLX_OFFLINE=true cargo check
```
