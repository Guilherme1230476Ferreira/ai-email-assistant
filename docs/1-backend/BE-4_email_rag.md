# BE-4: Email Generation with RAG

## Description
As a user, I want the AI to generate email replies that mimic my tone and utilize my past knowledge so that I can respond more efficiently.

## Acceptance Criteria
- [ ] Implement `POST /api/emails/generate` to initiate response generation.
- [ ] Use `pgvector` to find similar past emails based on cosine similarity.
- [ ] Construct a RAG (Retrieval-Augmented Generation) prompt including retrieved context.
- [ ] Support SSE (Server-Sent Events) for real-time streaming of LLM tokens.
- [ ] Generate and store embeddings for both the prompt and the final response.
- [ ] Automatically calculate "Tokens Saved" based on character count and context reuse.

## Dependencies
- **BE-3**: Requires LLM configuration (URL, Key) to communicate with providers.
- **SEC-1**: Requires authenticated session to associate emails with a user.
