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

CREATE INDEX machine_user_index
    ON machines (user_id);

CREATE TRIGGER set_timestamp
BEFORE UPDATE ON machines
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();

-- CPU Data
-- Covers all data which generally does not change during runtime
-- and requires to be set only once.
CREATE TABLE IF NOT EXISTS cpu (
    machine_id BIGINT PRIMARY KEY,
    n_cores INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX cpu_machine_index
    ON cpu (machine_id);

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

CREATE INDEX cpu_statistics_machine_index
    ON cpu_statistics (machine_id);

-- Mounts
CREATE TABLE IF NOT EXISTS mounts (
    id BIGSERIAL PRIMARY KEY,
    machine_id BIGINT NOT NULL,
    device_name TEXT NOT NULL,
    mount_location TEXT NOT NULL,
    total BIGINT NOT NULL,
    free BIGINT NOT NULL,
    fs_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX mounts_machine_index
    ON mounts (machine_id);
CREATE UNIQUE INDEX mounts_index
    ON mounts (machine_id, device_name);

CREATE TRIGGER set_timestamp
    BEFORE UPDATE ON mounts
    FOR EACH ROW
    EXECUTE PROCEDURE trigger_set_timestamp();

-- Memory
CREATE TABLE IF NOT EXISTS memory_statistics (
    id BIGSERIAL PRIMARY KEY,
    machine_id BIGINT,
    total BIGINT NOT NULL,
    free BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX memory_statistics_machine_index
    ON memory_statistics (machine_id);

-- System Info
CREATE TABLE IF NOT EXISTS system_info (
    machine_id BIGINT NOT NULL PRIMARY KEY,
    boot_time TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX system_info_machine_index
    ON system_info (machine_id);

CREATE TRIGGER set_timestamp
    BEFORE UPDATE ON system_info
    FOR EACH ROW
    EXECUTE PROCEDURE trigger_set_timestamp();

-- Network Devices
CREATE TABLE IF NOT EXISTS network_device_statistics (
    machine_id BIGINT NOT NULL,
    device_name TEXT NOT NULL,
    bytes_received BIGINT NOT NULL,
    bytes_sent BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX network_device_statistics_index 
    ON network_device_statistics (machine_id, device_name);

CREATE TRIGGER set_timestamp
    BEFORE UPDATE ON network_device_statistics
    FOR EACH ROW
    EXECUTE PROCEDURE trigger_set_timestamp();
