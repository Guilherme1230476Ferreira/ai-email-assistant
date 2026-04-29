# SEC-3: System Hardening

## Description
As an enterprise application, the system must implement several layers of protection against common vulnerabilities and data exposure.

## Acceptance Criteria
- [ ] **Rate Limiting**: Implement a sliding-window rate limiter for the `/api/auth/login` endpoint (e.g., 5 attempts per minute per IP).
- [ ] **Data Sanitization**: Ensure that all API DTOs (Data Transfer Objects) use `#[serde(skip_serializing)]` for sensitive database fields like `password_hash`.
- [ ] **Encryption at Rest**: All LLM API keys must be encrypted using AES-256-GCM before being saved to the database.
- [ ] **CORS Policy**: Implement a strict Cross-Origin Resource Sharing policy to prevent unauthorized browser access (Tower-HTTP).

## Dependencies
- **BE-3**: LLM settings management logic.
