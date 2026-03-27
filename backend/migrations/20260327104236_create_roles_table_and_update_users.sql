-- Create the roles table
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE
);

-- Insert default roles
INSERT INTO roles (name) VALUES ('admin'), ('user');

-- Add a new role_id column to the users table, nullable for now
ALTER TABLE users ADD COLUMN role_id UUID;

-- Update the new role_id column based on the old enum role column
UPDATE users u SET role_id = (SELECT id FROM roles r WHERE r.name = u.role::text);

-- Now that the data is migrated, we can drop the old role column
ALTER TABLE users DROP COLUMN role;

-- And we can drop the now-unused enum type
DROP TYPE user_role;

-- Finally, make the role_id column not nullable and add the foreign key constraint
ALTER TABLE users ALTER COLUMN role_id SET NOT NULL;
ALTER TABLE users ADD CONSTRAINT fk_users_role_id FOREIGN KEY (role_id) REFERENCES roles(id);
