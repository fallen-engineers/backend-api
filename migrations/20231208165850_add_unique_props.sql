-- Add migration script here
ALTER TABLE "user"
ADD CONSTRAINT user_name_email_key UNIQUE ("name", "email");