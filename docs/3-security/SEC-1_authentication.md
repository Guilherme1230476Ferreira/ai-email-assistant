# SEC-1: JWT Authentication & Guarding

## Description
As a security-conscious application, we must ensure that only authenticated users can access the system and that their sessions are securely managed.

## Acceptance Criteria
- [ ] Implement `POST /api/auth/login` to issue JWT tokens.
- [ ] Use `HS256` signing with a strong environment-defined secret.
- [ ] Implement an `AuthMiddleware` in the backend to validate tokens for all protected routes.
- [ ] On the frontend, implement a "Layout Guard" that redirects unauthenticated users to the login page.
- [ ] Securely store the JWT in `localStorage` or `HttpOnly` cookies (current implementation uses `localStorage`).

## Dependencies
- **BE-1**: Backend user repository for credential validation.
