-- Add up migration script here
ALTER TABLE "users"
ADD COLUMN username VARCHAR(255) UNIQUE;