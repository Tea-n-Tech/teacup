extern crate systemstat;
extern crate tokio;

mod data_collection;
mod server;
use tonic::transport::Server;

use data_collection::{
    get_battery_info, get_cpu_info, get_disk_info, get_network_stats, get_ram_info, get_system_info,
};
use server::MetricService;
use std::time;
use systemstat::{Platform, System};

use crate::server::proto::event_service_server::EventService;

// #[tokio::main]
// async fn main() {
//     // let emitter = CpuChangeEmitter {
//     //     last_update_index: 0,
//     //     update_index: 0,
//     //     cpu_usage: 0.0,
//     // };
//     // let mut stream = tokio_stream::iter(vec![3, 4]);

//     // let (tx, mut rx) = mpsc::channel::<CpuChangeEvent>(32);

//     // let sys = System::new();
//     // tokio::spawn(async move {
//     //     let change_event = get_cpu_info(&sys).await.unwrap();
//     //     let result = tx.send(change_event).await;
//     //     match result {
//     //         Ok(_) => println!("Sent"),
//     //         Err(e) => println!("Error: {}", e),
//     //     }
//     // });

//     // tokio::spawn(async move {
//     //     // tx2.send("sending from second handle").await;
//     // });

//     // while let Some(message) = rx.recv().await {
//     //     println!("GOT {:?}", message);
//     // }

//     let sys = &System::new();
//     loop {
//         let change_event = get_cpu_info(sys).await.unwrap();
//         println!("Change Event: {:?}", change_event);

//         let mem_change_event = get_ram_info(sys).await;
//         println!("Change Event: {:?}", mem_change_event);

//         let mounts = get_disk_info(sys).await;
//         println!("Change Event: {:?}", mounts);

//         let battery_change_events = get_battery_info(sys).await;
//         println!("Change Event: {:?}", battery_change_events);

//         let network_stats = get_network_stats(sys).await;
//         println!("Change Event: {:?}", network_stats);

//         let system_info = get_system_info(sys).await;
//         println!("Change Event: {:?}", system_info);

//         // sys.block_device_statistics()
//         //     .unwrap()
//         //     .iter()
//         //     .for_each(|d| println!("{}: {:?}", d.0, d.1));

//         let sleep_duration = time::Duration::from_secs(1);
//         tokio::time::sleep(sleep_duration).await;
//     }
// }
