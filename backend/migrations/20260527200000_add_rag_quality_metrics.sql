-- Add per-email RAG quality metric columns.
-- faithfulness_score : fraction of reply sentences grounded in retrieved context (0.0-1.0)
-- context_precision  : average hybrid similarity score of retrieved chunks (0.0-1.0)
-- retrieval_ms       : wall-clock time for the entire embed+search+rerank stage

ALTER TABLE emails
  ADD COLUMN IF NOT EXISTS faithfulness_score FLOAT,
  ADD COLUMN IF NOT EXISTS context_precision  FLOAT,
  ADD COLUMN IF NOT EXISTS retrieval_ms       INTEGER;
