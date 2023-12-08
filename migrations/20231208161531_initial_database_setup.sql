CREATE TABLE IF NOT EXISTS "user" (
    "id" UUID PRIMARY KEY,
    "name" TEXT NOT NULL,
    "email" TEXT NOT NULL UNIQUE,
    "inserted_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL
);