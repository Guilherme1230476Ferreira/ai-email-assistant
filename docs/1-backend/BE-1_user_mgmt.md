# BE-1: User Management (CRUD)

## Description
As an administrator, I want to manage system users (Create, Read, Update, Delete) so that I can control access to the platform and assign appropriate roles.

## Acceptance Criteria
- [ ] Implement `POST /api/admin/users` to create a new user with email and password.
- [ ] Password must be hashed using Bcrypt before storage.
- [ ] Implement `GET /api/admin/users` with support for pagination (limit/offset).
- [ ] Implement `PUT /api/admin/users/:id/role` to change a user's assigned role.
- [ ] Implement `DELETE /api/admin/users/:id` to remove a user from the system.
- [ ] API responses must exclude sensitive fields like `password_hash`.
- [ ] Duplicate email addresses must be rejected with a `409 Conflict` status.

## Dependencies
- **SEC-2**: Requires RBAC middleware to restrict access to Admins only.
- **BE-2**: Requires roles to exist for assignment.
