-- Add retrieval tracking to knowledge entries.
-- Allows the analytics dashboard to show which KB entries are most frequently retrieved
-- during email generation, so admins can identify and prune unused entries.
ALTER TABLE knowledge_entries ADD COLUMN retrieval_count BIGINT NOT NULL DEFAULT 0;
