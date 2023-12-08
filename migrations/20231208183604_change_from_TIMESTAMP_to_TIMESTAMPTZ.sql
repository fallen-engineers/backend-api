-- Add migration script here
ALTER TABLE "user"
ALTER COLUMN "inserted_at" TYPE TIMESTAMPTZ,
ALTER COLUMN "updated_at" TYPE TIMESTAMPTZ;
