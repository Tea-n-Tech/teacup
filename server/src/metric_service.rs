extern crate protocol as proto;

use self::proto::{
    event_service_server::EventService, ChangeEventBatch, InitialStateRequest, InitialStateResponse,
};

#[path = "database.rs"]
mod database;
use database::{Database, PgDatabase};

use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct MetricService {
    db: Arc<dyn Database>,
}

impl MetricService {
    pub async fn new(user: String, pw: String) -> Self {
        let address = format!("postgres://{}:{}@localhost:5432/teacup", user, pw);
        let db = PgDatabase::new(address.as_str()).await;
        MetricService { db: Arc::new(db) }
    }
}

impl tonic::transport::NamedService for MetricService {
    const NAME: &'static str = "EventService";
}

#[tonic::async_trait]
impl EventService for MetricService {
    async fn send_events(
        &self,
        request: tonic::Request<ChangeEventBatch>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let batch = request.into_inner();
        eprintln!("Got batch: {:?}", batch);

        self.db.process_event(&batch).await;

        Ok(tonic::Response::new(()))
    }

    async fn initial_state(
        &self,
        request: tonic::Request<InitialStateRequest>,
    ) -> Result<tonic::Response<InitialStateResponse>, tonic::Status> {
        let payload = request.into_inner();

        // Store system info which does not change over time
        match payload.system_info {
            Some(system_info) => {
                self.db
                    .save_system_info(payload.machine_id, &system_info)
                    .await;
            }
            None => {
                eprintln!("Initial request misses system info");
            }
        };

        // Store cpu info which does not change over time
        match payload.cpu_info {
            Some(cpu_info) => {
                self.db.save_cpu_info(payload.machine_id, &cpu_info).await;
            }
            None => {
                eprintln!("Initial request misses cpu info");
            }
        };

        // Fetch mounts so client sends us just updates
        let mounts = match self.db.fetch_mounts(payload.machine_id).await {
            Ok(mounts) => mounts,
            Err(e) => {
                eprintln!("Failed to fetch mounts from database: {}", e);
                return Err(tonic::Status::new(
                    tonic::Code::Internal,
                    "Failed to fetch mounts from database.",
                ));
            }
        };

        // Fetch mounts so client sends us just updates
        let network_devices = match self.db.fetch_network_devices(payload.machine_id).await {
            Ok(network_devices) => network_devices,
            Err(e) => {
                eprintln!("Failed to fetch network devices from database: {}", e);
                return Err(tonic::Status::new(
                    tonic::Code::Internal,
                    "Failed to fetch network devices from database.",
                ));
            }
        };

        Ok(tonic::Response::new(InitialStateResponse {
            mounts,
            network_devices,
        }))
    }
}
