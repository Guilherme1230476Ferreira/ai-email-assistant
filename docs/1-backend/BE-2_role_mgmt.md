# BE-2: Role Management

## Description
As a system administrator, I want to define and manage user roles so that I can implement granular access control across the application.

## Acceptance Criteria
- [ ] Implement `POST /api/admin/roles` to create a new role.
- [ ] Implement `GET /api/admin/roles` to retrieve a list of all available roles.
- [ ] Implement `DELETE /api/admin/roles/:id` to remove a role.
- [ ] System roles like `admin` and `user` should be protected or seeded by default.
- [ ] Prevent deletion of roles that are currently assigned to active users.

## Dependencies
- **SEC-2**: RBAC middleware depends on these roles to enforce permissions.
