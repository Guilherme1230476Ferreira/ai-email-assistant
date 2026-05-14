-- Knowledge base: global entries (documents + Q&A pairs) for RAG context
-- Managed by admins, available to all users during email generation

CREATE TABLE IF NOT EXISTS knowledge_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- 'document' or 'qa_pair'
    entry_type TEXT NOT NULL CHECK (entry_type IN ('document', 'qa_pair')),
    -- For documents: filename. For Q&A: the question text
    title TEXT NOT NULL,
    -- Full content (extracted document text or Q&A answer)
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Chunked embeddings for knowledge entries
-- Documents are split into ~500-char chunks; Q&A pairs produce 1 chunk
CREATE TABLE IF NOT EXISTS knowledge_embeddings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entry_id UUID NOT NULL REFERENCES knowledge_entries(id) ON DELETE CASCADE,
    -- The text chunk that was embedded
    chunk_text TEXT NOT NULL,
    -- Ordering index for multi-chunk documents
    chunk_index INTEGER NOT NULL DEFAULT 0,
    -- The vector embedding of chunk_text
    embedding VECTOR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_knowledge_entries_type ON knowledge_entries(entry_type);
CREATE INDEX IF NOT EXISTS idx_knowledge_embeddings_entry ON knowledge_embeddings(entry_id);
