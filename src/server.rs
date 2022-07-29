mod database;
use std::{net::SocketAddr, sync::Arc};

use clap::Parser;
use database::{Database, PgDatabase};
use tonic;

use self::proto::{
    event_service_server::EventService, event_service_server::EventServiceServer, ChangeEventBatch,
    InitialStateRequest, InitialStateResponse,
};

pub mod proto {
    #![allow(unreachable_pub)]
    #![allow(missing_docs)]
    tonic::include_proto!("change_events");
}

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct ServerCli {
    #[clap(short = 'p', long, value_parser, default_value_t = 50055)]
    port: u16,
}

#[derive(Clone, Debug)]
pub struct MetricService {
    db: Arc<dyn Database>,
}

impl MetricService {
    pub async fn new() -> Self {
        let db = PgDatabase::new("postgres://teacup:teacup@localhost:5432/teacup").await;
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
        println!("Got batch: {:?}", batch);

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_config = ServerCli::parse();

    let addr: SocketAddr = format!("0.0.0.0:{}", cli_config.port).parse().unwrap();
    let sv = MetricService::new().await;

    println!("Listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(EventServiceServer::new(sv))
        .serve(addr)
        .await?;

    Ok(())
}
