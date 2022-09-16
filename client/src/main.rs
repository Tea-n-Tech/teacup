extern crate core;

// mod
mod env;
mod event_submitter;

use clap::Parser;
use env::get_api_token;
use event_submitter::EventSubmitter;
use tc_core::{get_settings_filepath, load_settings};
// use tonic::metadata::MetadataValue;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct ClientCli {
    #[clap(short = 'e', long, value_parser, default_value_t = 5)]
    send_every: u32,
    #[clap(short = 'h', long, value_parser, default_value = "http://localhost")]
    address: String,
    #[clap(short = 'p', long, value_parser, default_value_t = 50055)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = ClientCli::parse();
    eprintln!("Cli config: {:?}", &cli);
    let api_token = get_api_token();

    let settings_filepath = get_settings_filepath().await;
    let settings = load_settings(&settings_filepath).await;

    // receive change events from a channel and send them to the
    // server.
    let send_handler = tokio::task::spawn(async move {
        let mut submitter = EventSubmitter::new(cli.clone(), settings.machine_id, api_token).await;
        match submitter.start().await {
            Ok(_) => {
                // graceful termination
            }
            Err(e) => {
                eprintln!("Error submitting events: {:?}", e);
            }
        }
    });

    match send_handler.await {
        Ok(_) => {
            // graceful termination
        }
        Err(e) => {
            eprintln!("Error sending events: {:?}", e);
        }
    }

    Ok(())
}
