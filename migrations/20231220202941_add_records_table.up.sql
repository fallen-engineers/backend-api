-- Add up migration script here
CREATE TABLE IF NOT EXISTS
    "records" (
        id SERIAL PRIMARY KEY NOT NULL,
        created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        last_updated_by VARCHAR(255) NOT NULL,
        first_name VARCHAR(255) NOT NULL,
        last_name VARCHAR(255) NOT NULL,
        mi VARCHAR(255) NOT NULL,
        course VARCHAR(255) NOT NULL,
        year_level VARCHAR(255) NOT NULL,
        payment_for VARCHAR(255) NOT NULL,
        amount VARCHAR(255) NOT NULL,
        received_by VARCHAR(255) NOT NULL
);