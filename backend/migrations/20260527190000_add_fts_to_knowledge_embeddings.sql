-- Add GIN-indexed tsvector column for BM25-style full-text search.
-- Using 'simple' config: language-agnostic, works for both Portuguese and English.
-- GENERATED ALWAYS AS ... STORED keeps the column automatically in sync
-- with chunk_text without any application-level updates.

ALTER TABLE knowledge_embeddings
  ADD COLUMN IF NOT EXISTS chunk_tsv tsvector
    GENERATED ALWAYS AS (to_tsvector('simple', chunk_text)) STORED;

CREATE INDEX IF NOT EXISTS idx_knowledge_embeddings_tsv
  ON knowledge_embeddings USING GIN (chunk_tsv);
