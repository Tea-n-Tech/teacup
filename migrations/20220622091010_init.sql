-- Add migration script here

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);


-- CPU Data
CREATE TABLE IF NOT EXISTS cpu (
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL,
    cores INTEGER,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);

CREATE TABLE IF NOT EXISTS cpu_statistics (
    id INTEGER PRIMARY KEY,
    cpu_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    usage FLOAT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP
);
