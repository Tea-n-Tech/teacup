#[path = "./data_collection.rs"]
mod data_collection;
#[path = "./local_settings.rs"]
mod local_settings;

use std::fmt::format;

use data_collection::proto::event_service_client::EventServiceClient;
use data_collection::proto::InitialStateRequest;
use systemstat::{Platform, System};

use tokio::sync::mpsc;
use tonic::codegen::InterceptedService;
use tonic::metadata::{Ascii, MetadataValue};
use tonic::service::Interceptor;
use tonic::transport::Channel;
use tonic::{Request, Status};

use crate::event_submitter::data_collection::{get_initial_state, get_system_info};
use crate::ClientCli;

struct InsertAuthTokenInterceptor {
    token: MetadataValue<Ascii>,
}

impl Interceptor for InsertAuthTokenInterceptor {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        request
            .metadata_mut()
            .insert("authorization", self.token.clone());
        Ok(request)
    }
}

pub struct EventSubmitter {
    client: EventServiceClient<InterceptedService<Channel, InsertAuthTokenInterceptor>>,
    submission_handler: Option<tokio::task::JoinHandle<()>>,
    machine_id: i64,
}

impl Drop for EventSubmitter {
    fn drop(&mut self) {
        match self.submission_handler {
            Some(ref mut submission_handler) => {
                submission_handler.abort();
            }
            None => {}
        }
    }
}

impl EventSubmitter {
    pub async fn new(cli: ClientCli, machine_id: i64, token: String) -> Self {
        let channel = Channel::from_shared(format!("{}:{}", cli.address, cli.port).to_string())
            .expect("Invalid server address")
            .connect()
            .await
            .expect("Error connecting to the server");

        let token_value: MetadataValue<_> = format!("Bearer {}", token).parse().unwrap();

        let event_service = EventServiceClient::with_interceptor(
            channel,
            InsertAuthTokenInterceptor { token: token_value },
        );

        Self {
            client: event_service,
            submission_handler: None,
            machine_id,
        }
    }

    pub async fn start(&mut self) -> Result<(), ()> {
        // This loop retries to contact the server in case of any errors
        // during communication such as a disconnect
        loop {
            match self.submit_events().await {
                Ok(_) => {
                    // The process runs indefinitely but
                    // we assume getting here means
                    // graceful termination for whatever
                    // reason.
                }
                Err(_) => {
                    println!("Waiting 5 seconds before trying again.");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
            return Ok(());
        }
    }

    async fn submit_events(&mut self) -> Result<(), ()> {
        let (tx, mut rx) = mpsc::channel::<data_collection::proto::ChangeEventBatch>(32);

        println!("Fetching initial state");
        let initial_state_result = self
            .client
            .initial_state(tonic::Request::new(
                get_initial_state(self.machine_id.clone()).await,
            ))
            .await;
        if let Err(err) = &initial_state_result {
            eprintln!("Failed to get initial state: {}", err);
            return Err(());
        }

        let initial_state = initial_state_result.unwrap().into_inner();
        println!("Got initial state: {:?}", initial_state);

        // collect data indefinitely and send data to the channel
        let machine_id_clone = self.machine_id.clone();
        self.submission_handler = Some(tokio::task::spawn(async move {
            data_collection::collect_events(tx, initial_state, machine_id_clone).await;
        }));

        loop {
            match rx.recv().await {
                Some(event_batch) => {
                    println!("Sending events {:?}", event_batch);
                    let request = tonic::Request::new(event_batch);
                    match self.client.send_events(request).await {
                        Ok(response) => {
                            println!("RESPONSE={:?}", response);
                        }
                        Err(e) => {
                            eprintln!("Error sending events: {:?}", e);
                            return Err(());
                        }
                    }
                }
                None => {}
            }
        }
    }
}
