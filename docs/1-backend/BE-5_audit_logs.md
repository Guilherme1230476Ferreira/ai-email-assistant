# BE-5: Audit Logging System

## Description
As a security auditor, I want to track all administrative actions (User creation/deletion, Role changes, Settings updates) so that I can maintain accountability and transparency.

## Acceptance Criteria
- [ ] Implement an `AuditRepository` to persist logs to the `audit_logs` table.
- [ ] Each log must include the acting User ID, the action name, and a JSON metadata payload.
- [ ] Implement `GET /api/admin/audit-logs` for administrators to view the history.
- [ ] Audit logs must be immutable (no update or delete endpoints).
- [ ] Integrate logging calls into all administrative handlers.

## Dependencies
- **SEC-2**: Access to audit logs must be restricted to administrators.
