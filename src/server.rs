mod store_events;
use std::net::SocketAddr;

use clap::Parser;
use store_events::Database;
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
    db: Database,
}

impl MetricService {
    pub async fn new() -> Self {
        let db = Database::new("postgres://teacup:teacup@localhost:5432/teacup").await;
        MetricService { db }
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
        _request: tonic::Request<InitialStateRequest>,
    ) -> Result<tonic::Response<InitialStateResponse>, tonic::Status> {
        Ok(tonic::Response::new(InitialStateResponse::default()))
    }

    async fn machine_handshake(
        &self,
        request: tonic::Request<proto::MachineHandshakeRequest>,
    ) -> Result<tonic::Response<proto::MachineHandshakeResponse>, tonic::Status> {
        Ok(tonic::Response::new(
            proto::MachineHandshakeResponse::default(),
        ))
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
