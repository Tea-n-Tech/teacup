extern crate systemstat;
extern crate tokio;

mod change_events;

use change_events::BatteryChangeEvent;
use change_events::CpuChangeEvent;
use change_events::MemoryChangeEvent;
use change_events::Mount;
use change_events::NetworkDevice;
use std::net;
use std::vec;
use std::{
    pin::Pin,
    result,
    task::{Context, Poll},
    thread, time,
};
use systemstat::{saturating_sub_bytes, Platform, System};
use tokio::sync::mpsc;

async fn get_cpu_info(sys: &impl Platform) -> Result<CpuChangeEvent, std::io::Error> {
    let mut usage: f32 = 0.;
    let mut temp: f32 = 0.;

    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            println!("Measuring CPU load...");

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
            usage = cpu.user;
        }
        Err(x) => {
            println!("CPU load: error: {}", x);
            return Err(x);
        }
    }

    match sys.cpu_temp() {
        Ok(cpu_temp) => {
            println!("CPU temp: {}", cpu_temp);
            temp = cpu_temp;
        }
        Err(x) => {
            println!("CPU temp: error: {}", x);
            return Err(x);
        }
    }

    Ok(CpuChangeEvent {
        usage: usage,
        temp: temp,
    })
}

async fn get_ram_info(sys: &impl Platform) -> Result<MemoryChangeEvent, std::io::Error> {
    match sys.memory() {
        Ok(mem) => {
            println!("Memory Total: {}, Free: {}", mem.total, mem.free);
            // mem.platform_memory
            //     .meminfo
            //     .into_iter()
            //     .for_each(|x| println!("{}: {}", x.0, x.1));

            Ok(MemoryChangeEvent {
                free: mem.free.as_u64(),
                total: mem.total.as_u64(),
            })
        }
        Err(x) => {
            println!("Memory load: error: {}", x);
            Err(x)
        }
    }
}

async fn get_disk_info(sys: &impl Platform) -> Result<Vec<Mount>, std::io::Error> {
    match sys.mounts() {
        Ok(mounts) => {
            mounts.iter().for_each(|fs| {
                println!(
                    "{} -> {} ({}) {}/{} free",
                    fs.fs_mounted_from, fs.fs_mounted_on, fs.fs_type, fs.avail, fs.total
                )
            });
            let mount_vec = mounts
                .iter()
                .map(|fs| Mount {
                    device_name: fs.fs_mounted_from.clone(),
                    mount_location: fs.fs_mounted_on.clone(),
                    free: fs.avail.as_u64(),
                    total: fs.total.as_u64(),
                    fs_type: fs.fs_type.clone(),
                })
                .collect();
            Ok(mount_vec)
        }
        Err(x) => {
            println!("Disk load: error: {}", x);
            Err(x)
        }
    }
}

async fn get_battery_info(sys: &impl Platform) -> Result<BatteryChangeEvent, std::io::Error> {
    match sys.battery_life() {
        Ok(battery) => {
            println!(
                "Battery Life: Remain Capacity: {} Remaining Time: {} mins",
                battery.remaining_capacity,
                battery.remaining_time.as_secs() / 60,
            );
            Ok(BatteryChangeEvent {
                remaining_capacity: battery.remaining_capacity,
                remaining_seconds: battery.remaining_time.as_secs(),
            })
        }
        Err(x) => {
            println!("Battery load: error: {}", x);
            Err(x)
        }
    }
}

async fn get_network_stats(sys: &impl Platform) -> Result<Vec<NetworkDevice>, std::io::Error> {
    match sys.networks() {
        Ok(networks) => {
            let device_stats = networks
                .iter()
                .map(|network| network.0)
                .map(|name| {
                    let network = sys.network_stats(name).unwrap();
                    println!(
                        "{}: sent: {}, recv: {}",
                        name, network.tx_bytes, network.rx_bytes
                    );
                    NetworkDevice {
                        name: name.clone(),
                        bytes_received: network.rx_bytes.as_u64(),
                        bytes_sent: network.tx_bytes.as_u64(),
                    }
                })
                .collect();
            Ok(device_stats)
        }
        Err(err) => {
            println!("Network load: error: {}", err);
            Err(err)
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

    let sys = &System::new();
    loop {
        let change_event = get_cpu_info(sys).await.unwrap();
        println!("Change Event: {:?}", change_event);

        let mem_change_event = get_ram_info(sys).await;
        println!("Change Event: {:?}", mem_change_event);

        let mounts = get_disk_info(sys).await;
        println!("Change Event: {:?}", mounts);

        let battery_change_events = get_battery_info(sys).await;
        println!("Change Event: {:?}", battery_change_events);

        let network_stats = get_network_stats(sys).await;
        println!("Change Event: {:?}", network_stats);

        sys.block_device_statistics()
            .unwrap()
            .iter()
            .for_each(|d| println!("{}: {:?}", d.0, d.1));

        let sleep_duration = time::Duration::from_secs(1);
        tokio::time::sleep(sleep_duration).await;
    }
}
