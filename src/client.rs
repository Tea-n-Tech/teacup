// mod
mod event_submitter;

use event_submitter::EventSubmitter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // receive change events from a channel and send them to the
    // server.
    let send_handler = tokio::task::spawn(async move {
        // This loop retries to contact in case of any errors
        // during communication such as a disconnect
        'submit_loop: loop {
            let mut submitter = EventSubmitter::new().await.unwrap();
            match submitter.submit_events().await {
                Ok(_) => {
                    // all went well
                    break 'submit_loop;
                }
                Err(_) => {
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
