-- Add down migration script here
ALTER TABLE "users"
ALTER COLUMN username DROP NOT NULL;