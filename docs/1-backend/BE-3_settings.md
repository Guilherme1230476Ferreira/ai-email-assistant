# BE-3: Global Application Settings

## Description
As an administrator, I want to configure global LLM parameters (Base URL, Model name, API Key) so that the application can integrate with different AI providers.

## Acceptance Criteria
- [ ] Implement `GET /api/admin/settings` to retrieve current configuration.
- [ ] Implement `PUT /api/admin/settings` to update LLM parameters.
- [ ] LLM API Keys must be encrypted at rest using AES-256-GCM.
- [ ] API responses must mask the API Key (e.g., `sk-...****`) to prevent exposure in the UI.
- [ ] Settings must be stored as a singleton record in the database.

## Dependencies
- **SEC-3**: Depends on `CryptoService` for encryption/decryption.
