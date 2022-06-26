use crate::proto::ChangeEventBatch;

use super::proto::change_event::Event;
use super::proto::{ChangeEvent, EventType};
use sqlx::pool::Pool;
use sqlx::postgres::{PgPoolOptions, Postgres};

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    pub async fn new(db_uri: &str) -> Database {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_uri)
            .await
            .expect("Failed to connect to database");

        Database { pool }
    }

    pub async fn process_event(&self, event_batch: &ChangeEventBatch) {
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
