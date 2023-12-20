-- Add up migration script here
ALTER TABLE "users"
    ALTER COLUMN photo DROP NOT NULL;
