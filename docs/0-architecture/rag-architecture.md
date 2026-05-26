# RAG Architecture — AI Email Assistant

## Overview

The AI Email Assistant implements a **Retrieval-Augmented Generation (RAG)** pipeline that grounds LLM responses in project-specific knowledge base documents. This prevents hallucinations and ensures replies are contextually accurate.

```
User Email (incoming)
        │
        ▼
┌───────────────────┐
│  Query Embedding  │  Jina AI jina-embeddings-v3 (1024-dim)
│  (embed_text)     │
└────────┬──────────┘
         │  query vector
         ▼
┌───────────────────────────────────────────────┐
│             pgvector Similarity Search         │
│                                               │
│  knowledge_embeddings  ←──── cosine distance  │
│  email_embeddings      ←──── cosine distance  │
└────────────────────┬──────────────────────────┘
                     │  top-K chunks (similarity ≥ threshold)
                     ▼
┌───────────────────────────────────────────────┐
│              Context Assembly                  │
│                                               │
│  [KB Chunk 1] [KB Chunk 2] [Email Thread 1]  │
│        max_chars truncation applied            │
└────────────────────┬──────────────────────────┘
                     │
                     ▼
┌───────────────────────────────────────────────┐
│          LLM Prompt Construction               │
│                                               │
│  System: You are a professional email         │
│          assistant. Use ONLY the context      │
│          below to answer.                     │
│  Context: {assembled chunks}                  │
│  User email: {incoming text}                  │
└────────────────────┬──────────────────────────┘
                     │
                     ▼
              LLM Response (streamed SSE)
```

---

## Components

### 1. Embedding Pipeline

**File:** `backend/src/services/rig_service.rs`

| Property | Value |
|---|---|
| Provider | Jina AI |
| Model | `jina-embeddings-v3` |
| Dimensions | 1024 |
| Endpoint | `https://api.jina.ai/v1/embeddings` |
| Rate limit | 500 RPM (free tier) |
| Inter-chunk delay | 150 ms (proactive throttle) |
| Retry strategy | Up to 6 retries; uses `retryDelay` from API 429 response |

**Upload flow (async):**
1. File received by `POST /api/knowledge/upload`
2. Text extracted (PDF via `pdf-extract`, plain text, Markdown)
3. Text chunked with `text-splitter` (semantic-aware, ~512 token chunks)
4. HTTP 202 returned immediately — user sees success
5. Background `tokio::spawn` embeds each chunk via Jina AI
6. Vectors stored in `knowledge_embeddings` table (pgvector)

### 2. Vector Storage

**Database:** PostgreSQL with `pgvector` extension

```sql
-- knowledge_embeddings
CREATE TABLE knowledge_embeddings (
    id          UUID PRIMARY KEY,
    entry_id    UUID REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    chunk_index INTEGER NOT NULL,
    chunk_text  TEXT NOT NULL,
    embedding   VECTOR   -- dimension-free; accepts any size
);

-- email_embeddings
CREATE TABLE email_embeddings (
    id                  UUID PRIMARY KEY,
    email_id            UUID REFERENCES emails(id) ON DELETE CASCADE,
    chunk_index         INTEGER NOT NULL,
    chunk_text          TEXT NOT NULL,
    content_embedding   VECTOR,
    response_embedding  VECTOR
);
```

Cosine similarity search via `<=>` operator:
```sql
SELECT chunk_text, (1.0 - (embedding <=> $1)) AS similarity
FROM knowledge_embeddings
WHERE embedding IS NOT NULL
ORDER BY embedding <=> $1
LIMIT 5;
```

### 3. Retrieval

**File:** `backend/src/services/rig_service.rs` — `search_similar_chunks()`

| Parameter | Value |
|---|---|
| Top-K knowledge chunks | 5 |
| Top-K email chunks | 3 |
| Similarity metric | Cosine (pgvector `<=>`) |
| Minimum similarity | No hard floor — ordered by distance, top-K selected |

Both knowledge base chunks and historical email embeddings are searched in parallel and merged by similarity score.

### 4. Context Assembly

Retrieved chunks are concatenated with section separators and truncated at `MAX_CONTEXT_CHARS` (currently ~8,000 chars) to avoid exceeding LLM context windows.

```
[Knowledge Base]
Chunk: <text>  (similarity: 0.87)
Chunk: <text>  (similarity: 0.82)

[Previous Emails]
Chunk: <text>  (similarity: 0.76)
```

### 5. LLM Generation

**File:** `backend/src/services/llm_service.rs`

- Provider: Configurable via `LLM_BASE_URL` + `LLM_MODEL` (stored in DB settings)
- Protocol: OpenAI-compatible chat completions (`/chat/completions`)
- Streaming: Server-Sent Events (SSE) streamed to the browser
- Prompt: System prompt enforces context grounding and instructs the model not to hallucinate beyond the provided context

---

## RAG Trace / Observability

Every email generation records:
- Whether context was retrieved (`has_context: bool`)
- Which knowledge base entries contributed (`kb_entry_ids: []`)
- Similarity scores for retrieved chunks
- Number of context characters injected

Accessible via the **RAG Trace** view in the email detail UI.

---

## Performance Characteristics

| Metric | Typical value |
|---|---|
| Embedding latency per chunk | ~100–300 ms (Jina API + network) |
| Upload time (100 chunks) | ~15 s background (non-blocking to user) |
| Retrieval latency | < 20 ms (pgvector index) |
| End-to-end generation (streaming first token) | 1–3 s |

---

## Configuration

All tunable via `/opt/ai-email-assistant/.env` on the server:

```env
EMBEDDING_API_URL=https://api.jina.ai/v1
EMBEDDING_API_KEY=jina_...
EMBEDDING_MODEL=jina-embeddings-v3
LLM_BASE_URL=https://generativelanguage.googleapis.com/v1beta/openai
LLM_MODEL=gemini-2.0-flash
```

LLM model and base URL can also be updated live via **Settings → LLM Configuration** in the admin dashboard (persisted in DB, no restart required).

---

## Known Limitations

| Limitation | Notes |
|---|---|
| Jina free tier: 1M tokens/month | ~500K chunks. Sufficient for academic deployment. |
| No BM25 hybrid search | Currently pure semantic (cosine) search. BM25 keyword search planned. |
| No re-ranking | Retrieved chunks are not re-ranked after retrieval. |
| PDF image content | Images/charts inside PDFs are not extracted or embedded. |
| Chunk overlap | No sliding-window overlap between chunks; boundary sentences may lose context. |
