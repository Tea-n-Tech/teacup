-- Add migration script here

-- This handy little function updates a tiemstamp every time we
-- modify a row in table using this trigger.
CREATE OR REPLACE FUNCTION trigger_set_timestamp()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Users
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TRIGGER set_timestamp
BEFORE UPDATE ON users
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

-- Machines
CREATE TABLE IF NOT EXISTS machines (
    id BIGINT PRIMARY KEY,
    -- null means no user owns the machine
    user_id INTEGER,
    ip INET NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX machine_user_index on machines (user_id);

CREATE TRIGGER set_timestamp
BEFORE UPDATE ON machines
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

-- CPU Data
-- Covers all data which generally does not change during runtime
-- and requires to be set only once.
CREATE TABLE IF NOT EXISTS cpu (
    machine_id INTEGER PRIMARY KEY,
    n_cores INTEGER,
    model TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX cpu_machine_index on cpu (machine_id);

CREATE TRIGGER set_timestamp
BEFORE UPDATE ON cpu
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

-- CPU Statistics
-- Collects runtime metrics for CPU.
CREATE TABLE IF NOT EXISTS cpu_statistics (
    id BIGSERIAL PRIMARY KEY,
    machine_id BIGINT NOT NULL,
    usage FLOAT,
    temperature FLOAT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX cpu_statistics_machine_index on cpu_statistics (machine_id);
