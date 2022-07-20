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
                        match sqlx::query(
                            "
                    INSERT INTO cpu (user_id) VALUES ($1)",
                        )
                        .bind(1 as i32)
                        .execute(&self.pool)
                        .await
                        {
                            Ok(_) => println!("Inserted CPU event"),
                            Err(err) => {
                                eprintln!("Failed to insert CPU event: {}", err);
                                return;
                            }
                        }
                    }
                    Event::Memory(mem) => println!("Memory event"),
                    Event::Mount(mount) => println!("Mount event"),
                    Event::NetworkDevice(net_device) => {}
                    Event::SystemInfo(info) => println!("System info event"),
                    Event::Battery(battery) => println!("Battery event"),
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
