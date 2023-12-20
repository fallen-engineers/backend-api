-- Add down migration script here
ALTER TABLE "users"
ALTER COLUMN photo SET NOT NULL;