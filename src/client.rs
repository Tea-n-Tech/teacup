mod data_collection;

use data_collection::proto::event_service_client::EventServiceClient;
use futures_util::try_join;
use tokio::sync::mpsc;
use tonic::transport::Channel;

struct EventSubmitter {
    client: EventServiceClient<Channel>,
    tx: mpsc::Sender<data_collection::proto::ChangeEventBatch>,
    rx: mpsc::Receiver<data_collection::proto::ChangeEventBatch>,
}

impl Drop for EventSubmitter {
    fn drop(&mut self) {
        self.rx.close();
    }
}

impl EventSubmitter {
    pub async fn new(channel: Channel) -> Self {
        let (tx, rx) = mpsc::channel::<data_collection::proto::ChangeEventBatch>(32);
        Self {
            client: EventServiceClient::new(channel),
            tx,
            rx,
        }
    }

    async fn start_submission(&mut self) {
        println!("Fetching initial state");
        let initial_state_result = self.client.initial_state(tonic::Request::new(())).await;
        if let Err(err) = &initial_state_result {
            eprintln!("Failed to get initial state: {}", err);
            // return err;
        }

        let initial_state = initial_state_result.unwrap().into_inner();
        println!("Got initial state: {:?}", initial_state);
        // TODO init data collection from initial state

        loop {
            match self.rx.recv().await {
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

async fn start_submission(mut client: EventServiceClient<Channel>) {
    // communication channels between data collection and sending
    let (tx, mut rx) = mpsc::channel::<data_collection::proto::ChangeEventBatch>(32);

    // collect data indefinitely and send data to the channel
    let collect_handler = tokio::task::spawn(async {
        data_collection::collect_events(tx).await;
    });

    println!("Fetching initial state");
    let initial_state_result = client.initial_state(tonic::Request::new(())).await;
    if let Err(err) = initial_state_result {
        eprintln!("Failed to get initial state: {}", err);
        // return err;
    }

    let initial_state = initial_state_result.unwrap().into_inner();
    println!("Got initial state: {:?}", initial_state);
    // TODO init data collection from initial state

    loop {
        match rx.recv().await {
            Some(event_batch) => {
                println!("Sending events {:?}", event_batch);
                let request = tonic::Request::new(event_batch);
                match client.send_events(request).await {
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
        let mut client = client_result.unwrap();

        'submit_loop: loop {}
    });

    match try_join!(collect_handler, send_handler) {
        Ok(_) => {}
        Err(e) => {
            println!("Error waiting for handlers: {:?}", e);
        }
    };

    Ok(())
}
