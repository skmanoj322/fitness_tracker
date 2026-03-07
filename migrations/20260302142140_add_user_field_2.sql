-- Add migration script here
ALTER TABLE users
    ADD COLUMN first_name TEXT,
    ADD COLUMN last_name TEXT,
    ADD COLUMN user_name TEXT;
