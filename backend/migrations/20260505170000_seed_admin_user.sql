-- Seed admin user: admin@gmail.com / admin (bcrypt cost 12)
-- This migration is IDEMPOTENT — safe to run multiple times.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

INSERT INTO users (email, password_hash, role_id)
SELECT
    'admin@gmail.com',
    crypt('admin', gen_salt('bf', 12)),
    r.id
FROM roles r
WHERE r.name = 'admin'
ON CONFLICT (email) DO UPDATE
    SET password_hash = crypt('admin', gen_salt('bf', 12)),
        role_id = (SELECT id FROM roles WHERE name = 'admin');
