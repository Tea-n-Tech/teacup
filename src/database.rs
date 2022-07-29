use std::fmt::Debug;

use crate::proto::ChangeEventBatch;

use super::proto::{change_event::Event, CpuInfo, EventType, Mount, NetworkDevice, SystemInfo};

use async_trait::async_trait;
use sqlx::error::Error;
use sqlx::pool::Pool;
use sqlx::postgres::{PgPoolOptions, Postgres};

// As we cannot apply the deconstruction Macro on protobuf
// entities since they are automatically generated, we do
// it here manually.
impl<'a, R: ::sqlx::Row> ::sqlx::FromRow<'a, R> for NetworkDevice
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    i64: ::sqlx::decode::Decode<'a, R::Database>,
    i64: ::sqlx::types::Type<R::Database>,
    i64: ::sqlx::decode::Decode<'a, R::Database>,
    i64: ::sqlx::types::Type<R::Database>,
{
    fn from_row(row: &'a R) -> ::sqlx::Result<Self> {
        let name: String = row.try_get("device_name")?;
        let bytes_received: i64 = row.try_get("bytes_received")?;
        let bytes_sent: i64 = row.try_get("bytes_sent")?;
        ::std::result::Result::Ok(NetworkDevice {
            name,
            bytes_received,
            bytes_sent,
        })
    }
}

// As we cannot apply the deconstruction Macro on protobuf
// entities since they are automatically generated, we do
// it here manually.
impl<'a, R: ::sqlx::Row> ::sqlx::FromRow<'a, R> for Mount
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    i64: ::sqlx::decode::Decode<'a, R::Database>,
    i64: ::sqlx::types::Type<R::Database>,
    i64: ::sqlx::decode::Decode<'a, R::Database>,
    i64: ::sqlx::types::Type<R::Database>,
{
    fn from_row(row: &'a R) -> ::sqlx::Result<Self> {
        let device_name: String = row.try_get("device_name")?;
        let mount_location: String = row.try_get("mount_location")?;
        let fs_type: String = row.try_get("fs_type")?;
        let free: i64 = row.try_get("free")?;
        let total: i64 = row.try_get("total")?;
        ::std::result::Result::Ok(Mount {
            device_name,
            mount_location,
            fs_type,
            free,
            total,
        })
    }
}

#[async_trait]
pub trait Database: Sync + Send + Debug {
    async fn process_event(&self, event_batch: &ChangeEventBatch);
    async fn save_system_info(&self, machine_id: i64, system_info: &SystemInfo);
    async fn save_cpu_info(&self, machine_id: i64, cpu_info: &CpuInfo);
    async fn fetch_mounts(&self, machine_id: i64) -> Result<Vec<Mount>, Error>;
    async fn fetch_network_devices(&self, machine_id: i64) -> Result<Vec<NetworkDevice>, Error>;
}

#[derive(Debug, Clone)]
pub struct PgDatabase {
    pool: Pool<Postgres>,
}

impl PgDatabase {
    pub async fn new(db_uri: &str) -> PgDatabase {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_uri)
            .await
            .expect("Failed to connect to database");

        PgDatabase { pool }
    }
}

#[async_trait]
impl Database for PgDatabase {
    async fn process_event(&self, event_batch: &ChangeEventBatch) {
        for event in event_batch.events.iter() {
            println!("Got event: {:?}", event);

            let event_type = event.event_type();
            let query = match &event.event {
                Some(event) => match event {
                    // CPU Statistics
                    Event::Cpu(cpu) => sqlx::query(
                        "
                        INSERT INTO cpu_statistics (machine_id, usage, temperature)
                            VALUES ($1, $2, $3)
                        ",
                    )
                    .bind(event_batch.machine_id)
                    .bind(cpu.usage)
                    .bind(cpu.temp),
                    // RAM Statistics
                    Event::Memory(mem) => sqlx::query(
                        "
                        INSERT INTO memory_statistics (machine_id, total, free)
                            VALUES ($1, $2, $3)
                            ",
                    )
                    .bind(event_batch.machine_id)
                    .bind(mem.total)
                    .bind(mem.free),
                    // Mounts
                    Event::Mount(mount) => match event_type {
                        EventType::Add | EventType::Update => sqlx::query(
                            "
                                INSERT INTO mounts (
                                    machine_id, device_name, mount_location,
                                    total, free, fs_type    
                                ) 
                                VALUES ($1, $2, $3, $4, $5, $6)
                                ON CONFLICT (machine_id, device_name) DO UPDATE SET
                                    mount_location = $3,
                                    total = $4,
                                    free = $5,
                                    fs_type = $6
                                    ",
                        )
                        .bind(event_batch.machine_id)
                        .bind(mount.device_name.to_string())
                        .bind(mount.mount_location.to_string())
                        .bind(mount.total)
                        .bind(mount.free)
                        .bind(mount.fs_type.to_string()),

                        EventType::Delete => sqlx::query(
                            "
                                DELETE FROM mounts WHERE
                                    machine_id = $1 AND
                                    device_name = $2
                                ",
                        )
                        .bind(event_batch.machine_id)
                        .bind(mount.device_name.to_string()),
                    },
                    // Network Devices
                    Event::NetworkDevice(net_device) => match event_type {
                        EventType::Add | EventType::Update => sqlx::query(
                            "
                                INSERT INTO network_device_statistics (
                                    machine_id, device_name, 
                                    bytes_received, bytes_sent
                                ) 
                                VALUES ($1, $2, $3, $4)
                                ON CONFLICT (machine_id, device_name) DO UPDATE SET
                                    device_name = $2,
                                    bytes_received = $3,
                                    bytes_sent = $4
                                    ",
                        )
                        .bind(event_batch.machine_id)
                        .bind(&net_device.name)
                        .bind(net_device.bytes_received)
                        .bind(net_device.bytes_sent),

                        EventType::Delete => sqlx::query(
                            "
                                DELETE FROM network_device_statistics WHERE
                                    machine_id = $1 AND
                                    device_name = $2
                                ",
                        )
                        .bind(event_batch.machine_id)
                        .bind(&net_device.name),
                    },
                },
                None => {
                    // null query
                    sqlx::query("")
                }
            };

            match query.execute(&self.pool).await {
                Ok(_) => println!("Updated database"),
                Err(err) => {
                    eprintln!("Failed to update database: {}", err);
                }
            };
        }
    }

    async fn save_system_info(&self, machine_id: i64, system_info: &SystemInfo) {
        let mut boot_time: i64 = 0;
        match &system_info.boot_time {
            Some(boot_time_) => boot_time = boot_time_.seconds,
            None => {}
        };

        match sqlx::query(
            "
        INSERT INTO system_info (machine_id, boot_time)
            VALUES ($1, to_timestamp($2))
            ON CONFLICT (machine_id) DO UPDATE SET
                boot_time = to_timestamp($2)
            ",
        )
        .bind(machine_id)
        .bind(boot_time)
        .execute(&self.pool)
        .await
        {
            Ok(_) => println!("Inserted system info"),
            Err(err) => {
                eprintln!("Failed to insert system info event: {}", err);
            }
        }
    }

    async fn save_cpu_info(&self, machine_id: i64, cpu_info: &CpuInfo) {
        match sqlx::query(
            "
        INSERT INTO cpu (machine_id, n_cores)
            VALUES ($1, $2)
            ON CONFLICT (machine_id) DO UPDATE SET
                n_cores = $2
                ",
        )
        .bind(machine_id)
        .bind(cpu_info.n_cores)
        .execute(&self.pool)
        .await
        {
            Ok(_) => println!("Updated database"),
            Err(err) => {
                eprintln!("Failed to update database: {}", err);
            }
        }
    }

    async fn fetch_mounts(&self, machine_id: i64) -> Result<Vec<Mount>, Error> {
        sqlx::query_as::<_, Mount>(
            "
        SELECT device_name, mount_location, total, free, fs_type
            FROM mounts
            WHERE machine_id = $1
            ",
        )
        .bind(machine_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn fetch_network_devices(&self, machine_id: i64) -> Result<Vec<NetworkDevice>, Error> {
        sqlx::query_as::<_, NetworkDevice>(
            "
        SELECT device_name, bytes_received, bytes_sent
            FROM network_device_statistics
            WHERE machine_id = $1
            ",
        )
        .bind(machine_id)
        .fetch_all(&self.pool)
        .await
    }
}
