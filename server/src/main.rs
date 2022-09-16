mod env;
mod metric_service;

use clap::Parser;
use env::{get_db_password, get_db_username};
use metric_service::MetricService;
use protocol::event_service_server::EventServiceServer;
use std::net::SocketAddr;
use tonic;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
struct ServerCli {
    #[clap(short = 'p', long, value_parser, default_value_t = 50055)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli_config = ServerCli::parse();
    let db_pw = get_db_password();
    let db_user = get_db_username();

    let addr: SocketAddr = format!("0.0.0.0:{}", cli_config.port).parse().unwrap();
    let sv = MetricService::new(db_user, db_pw).await;

    eprintln!("Listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(EventServiceServer::new(sv))
        .serve(addr)
        .await?;

    Ok(())
}
