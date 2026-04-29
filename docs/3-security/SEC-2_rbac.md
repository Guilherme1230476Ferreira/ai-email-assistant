# SEC-2: Role-Based Access Control (RBAC)

## Description
As an administrator, I want to ensure that only users with the `admin` role can perform administrative actions like managing users or changing global settings.

## Acceptance Criteria
- [ ] Implement an `AdminUser` extractor/middleware in Axum.
- [ ] This middleware must verify the `role_id` or `role_name` associated with the JWT user.
- [ ] Routes under `/api/admin/*` must return `403 Forbidden` if accessed by a non-admin user.
- [ ] Ensure that even if a non-admin manually navigates to an admin URL on the frontend, the backend remains locked.

## Dependencies
- **SEC-1**: Authentication must be established first.
- **BE-2**: Role definitions must be present.
