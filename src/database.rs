use std::fmt::Debug;

use crate::proto::ChangeEventBatch;

use super::proto::change_event::Event;
use super::proto::EventType;
use async_trait::async_trait;
use sqlx::pool::Pool;
use sqlx::postgres::{PgPoolOptions, Postgres};

#[async_trait]
pub trait Database: Sync + Send + Debug {
    async fn process_event(&self, event_batch: &ChangeEventBatch);
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
            match &event.event {
                Some(event) => match event {
                    Event::Cpu(cpu) => {
                        // general CPU data which is time independent
                        match sqlx::query(
                            "
                    INSERT INTO cpu (machine_id, n_cores, model)
                        VALUES ($1, $2, $3)
                        ON CONFLICT (machine_id) DO UPDATE SET
                            n_cores = $2,
                            model = $3",
                        )
                        .bind(event_batch.machine_id)
                        .bind(1 as i32)
                        .bind("AMD Ryzen 5 1600X")
                        .execute(&self.pool)
                        .await
                        {
                            Ok(_) => println!("Inserted CPU data"),
                            Err(err) => {
                                eprintln!("Failed to insert CPU event: {}", err);
                                return;
                            }
                        }

                        // CPU data which changes over time
                        match sqlx::query(
                            "
                    INSERT INTO cpu_statistics (machine_id, usage, temperature)
                        VALUES ($1, $2, $3)
                        ",
                        )
                        .bind(event_batch.machine_id)
                        .bind(cpu.usage)
                        .bind(cpu.temp)
                        .execute(&self.pool)
                        .await
                        {
                            Ok(_) => println!("Inserted CPU statistics"),
                            Err(err) => {
                                eprintln!("Failed to insert CPU event: {}", err);
                                return;
                            }
                        }
                    }
                    Event::Memory(mem) => {}
                    Event::Mount(mount) => {}
                    Event::NetworkDevice(net_device) => {}
                    Event::SystemInfo(info) => {}
                    Event::Battery(battery) => {}
                },
                None => {}
            }

            match EventType::from_i32(event.event_type) {
                Some(EventType::Add) => {}
                Some(EventType::Update) => {}
                Some(EventType::Delete) => {}
                None => {}
            }

            println!("Got event: {:?}", event);
        }
    }
}
