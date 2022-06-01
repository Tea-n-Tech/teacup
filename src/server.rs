use tonic::{
    transport::{NamedService, Server},
    Request, Response, Status,
};

use self::proto::{
    event_service_server::EventService, event_service_server::EventServiceServer, ChangeEventBatch,
};

pub mod proto {
    #![allow(unreachable_pub)]
    #![allow(missing_docs)]
    tonic::include_proto!("change_events");
}

#[derive(Clone, Debug, Default)]
pub struct MetricService {}

impl NamedService for MetricService {
    const NAME: &'static str = "EventService";
}

#[tonic::async_trait]
impl EventService for MetricService {
    async fn send_events(
        &self,
        request: Request<ChangeEventBatch>,
    ) -> Result<Response<()>, Status> {
        let batch = request.into_inner();
        println!("Got batch: {:?}", batch);
        Ok(Response::new(()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    let sv = MetricService::default();

    println!("Listening on {}", addr);

    Server::builder()
        .add_service(EventServiceServer::new(sv))
        .serve(addr)
        .await?;

    Ok(())
}
