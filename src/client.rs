mod data_collection;

use proto::event_service_client::EventServiceClient;
use tokio::sync::mpsc;

pub mod proto {
    #![allow(unreachable_pub)]
    #![allow(missing_docs)]
    tonic::include_proto!("change_events");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EventServiceClient::connect("http://[::1]:50051").await?;

    let (tx, rx) = mpsc::channel::<data_collection::proto::ChangeEventBatch>(32);
    data_collection::collect_events(tx).await;

    let request = tonic::Request::new(proto::ChangeEventBatch { events: vec![] });
    let response = client.send_events(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
