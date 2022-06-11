mod data_collection;

use data_collection::proto::event_service_client::EventServiceClient;
use futures_util::try_join;
use tokio::sync::mpsc;
use tonic::transport::Channel;

struct EventSubmitter {
    client: EventServiceClient<Channel>,
    submission_handler: Option<tokio::task::JoinHandle<()>>,
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
    pub fn new(client: EventServiceClient<Channel>) -> Self {
        Self {
            client: client,
            submission_handler: None,
        }
    }

    async fn submit_events(&mut self) -> Result<(), ()> {
        let (tx, mut rx) = mpsc::channel::<data_collection::proto::ChangeEventBatch>(32);

        println!("Fetching initial state");
        let initial_state_result = self.client.initial_state(tonic::Request::new(())).await;
        if let Err(err) = &initial_state_result {
            eprintln!("Failed to get initial state: {}", err);
            // return err;
        }

        let initial_state = initial_state_result.unwrap().into_inner();
        println!("Got initial state: {:?}", initial_state);

        // collect data indefinitely and send data to the channel
        self.submission_handler = Some(tokio::task::spawn(async {
            data_collection::collect_events(tx).await;
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
                        }
                    }
                }
                None => {}
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // receive change events from a channel and send them to the
    // server.
    let send_handler = tokio::task::spawn(async move {
        let client_result = EventServiceClient::connect("http://[::1]:50051").await;
        if let Err(e) = client_result {
            eprintln!("Failed to connect to the server: {}", e);
            return;
        }
        let client = client_result.unwrap();

        let mut submitter = EventSubmitter::new(client);
        'submit_loop: loop {
            match submitter.submit_events().await {
                Ok(_) => {
                    break 'submit_loop;
                }
                Err(e) => {
                    eprintln!("Error submitting events: {:?}", e);
                    println!("Waiting 5 seconds before trying again.");
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    });

    match send_handler.await {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error sending events: {:?}", e);
        }
    }

    Ok(())
}
