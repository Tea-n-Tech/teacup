// mod
mod event_submitter;

use clap::Parser;
use event_submitter::EventSubmitter;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct ClientCli {
    #[clap(short, long, value_parser, default_value_t = 5)]
    send_every: u32,
    #[clap(long, value_parser, default_value = "localhost")]
    server_address: String,
    #[clap(short = 'p', long, value_parser, default_value_t = 40040)]
    server_port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = ClientCli::parse();
    println!("Cli config: {:?}", &cli);

    // receive change events from a channel and send them to the
    // server.
    let send_handler = tokio::task::spawn(async move {
        // This loop retries to contact in case of any errors
        // during communication such as a disconnect
        'submit_loop: loop {
            let mut submitter = EventSubmitter::new(cli.clone()).await.unwrap();
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
