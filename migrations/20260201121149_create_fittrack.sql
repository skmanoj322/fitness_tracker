-- Add migration script here
CREATE TABLE users(
    id SERIAL PRIMARY KEY,
    telegram_id BIGINT UNIQUE NOT NULL,
    phone_number BIGINT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE exercise_tracker(
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name TEXT,
    weight_kg DECIMAL(5, 2),
    "set" INTEGER,
    rep INTEGER,
    completed_at TIMESTAMPTZ DEFAULT NOW()
);
