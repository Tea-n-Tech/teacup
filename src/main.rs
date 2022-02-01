extern crate systemstat;
extern crate tokio;

mod change_events;

use change_events::CpuChangeEvent;
use std::{
    pin::Pin,
    result,
    task::{Context, Poll},
    thread, time,
};
use systemstat::{saturating_sub_bytes, Platform, System};
use tokio::sync::mpsc;

async fn get_cpu_info(sys: &impl Platform) -> Result<CpuChangeEvent, ()> {
    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            println!("\nMeasuring CPU load...");

            tokio::time::sleep(time::Duration::from_secs(1)).await;

            let cpu = cpu.done().unwrap();
            println!(
                "CPU load: {}% user, {}% nice, {}% system, {}% intr, {}% idle ",
                cpu.user * 100.0,
                cpu.nice * 100.0,
                cpu.system * 100.0,
                cpu.interrupt * 100.0,
                cpu.idle * 100.0
            );
            Ok(CpuChangeEvent { usage: cpu.user })
        }
        Err(x) => {
            println!("\nCPU load: error: {}", x);
            Err(())
        }
    }
}

#[tokio::main]
async fn main() {
    // let emitter = CpuChangeEmitter {
    //     last_update_index: 0,
    //     update_index: 0,
    //     cpu_usage: 0.0,
    // };
    // let mut stream = tokio_stream::iter(vec![3, 4]);

    // let (tx, mut rx) = mpsc::channel::<CpuChangeEvent>(32);

    // let sys = System::new();
    // tokio::spawn(async move {
    //     let change_event = get_cpu_info(&sys).await.unwrap();
    //     let result = tx.send(change_event).await;
    //     match result {
    //         Ok(_) => println!("Sent"),
    //         Err(e) => println!("Error: {}", e),
    //     }
    // });

    // tokio::spawn(async move {
    //     // tx2.send("sending from second handle").await;
    // });

    // while let Some(message) = rx.recv().await {
    //     println!("GOT {:?}", message);
    // }

    loop {
        let change_event = get_cpu_info(&System::new()).await.unwrap();
        println!("Change Event: {:?}", change_event);

        let sleep_duration = time::Duration::from_secs(1);
        tokio::time::sleep(sleep_duration).await;
    }
}
