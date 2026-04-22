CREATE TABLE settings (
    id VARCHAR(50) PRIMARY KEY,
    llm_base_url VARCHAR(255) NOT NULL DEFAULT 'https://api.openai.com/v1',
    llm_model VARCHAR(255) NOT NULL DEFAULT 'gpt-4o',
    llm_api_key_encrypted VARCHAR(512),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert the default singleton row immediately
INSERT INTO settings (id, llm_base_url, llm_model) 
VALUES ('singleton', 'https://api.openai.com/v1', 'gpt-4o');