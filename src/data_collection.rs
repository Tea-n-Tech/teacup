extern crate systemstat;
extern crate tokio;

use prost_types::Timestamp;
use systemstat::Platform;
use systemstat::System;
use tokio::sync::mpsc;
use tokio::time;

pub mod proto {
    #![allow(unreachable_pub)]
    #![allow(missing_docs)]
    tonic::include_proto!("change_events");
}

pub async fn collect_events(tx: mpsc::Sender<proto::ChangeEventBatch>) {
    let forever = tokio::task::spawn(async move {
        let sys = &System::new();
        // TODO make configurable
        let mut interval = time::interval(time::Duration::from_secs(5));
        loop {
            interval.tick().await;

            let mut events: Vec<proto::ChangeEvent> = vec![];

            // cpu
            let cpu_info = get_cpu_update_event(sys).await.unwrap();
            let cpu_change_event = proto::ChangeEvent {
                event: Some(proto::change_event::Event::Cpu(cpu_info)),
                event_type: proto::EventType::Update.into(),
            };
            events.push(cpu_change_event);

            // ram
            match get_ram_info(sys).await {
                Ok(ram_info) => {
                    let mem_change_event = proto::ChangeEvent {
                        event: Some(proto::change_event::Event::Memory(ram_info)),
                        event_type: proto::EventType::Update.into(),
                    };
                    events.push(mem_change_event)
                }
                Err(e) => {
                    eprintln!("Error getting RAM info: {:?}", e);
                }
            }

            // disk
            match get_disk_info(sys).await {
                Ok(mounts) => {
                    // TODO
                }
                Err(err) => {
                    eprintln!("Error getting disk info: {:?}", err);
                }
            }

            // network
            match get_network_stats(sys).await {
                Ok(network_devices) => {
                    // TODO
                }
                Err(err) => {
                    eprintln!("Error getting network info: {:?}", err);
                }
            }

            // System
            match get_system_info(sys).await {
                Ok(system_info) => {
                    let system_change_event = proto::ChangeEvent {
                        event: Some(proto::change_event::Event::SystemInfo(system_info)),
                        event_type: proto::EventType::Update.into(),
                    };
                    events.push(system_change_event);
                }
                Err(e) => {
                    eprintln!("Error getting system info: {:?}", e);
                }
            }

            // Send stuff to the server
            match tx.send(proto::ChangeEventBatch { events: events }).await {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error sending batch of events: {:?}", e);
                }
            }
        }
    });

    match forever.await {
        Ok(_) => {}
        Err(e) => println!("Error collecting events: {}", e),
    }
}

async fn get_cpu_update_event(
    sys: &impl Platform,
) -> Result<proto::CpuChangeEvent, std::io::Error> {
    let usage: f32;
    let temp: f32;

    match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            println!("Measuring CPU load...");

            tokio::time::sleep(time::Duration::from_secs(1)).await;

            match cpu.done() {
                Ok(cpu_load) => {
                    println!(
                        "CPU load: {}% user, {}% nice, {}% system, {}% intr, {}% idle ",
                        cpu_load.user * 100.0,
                        cpu_load.nice * 100.0,
                        cpu_load.system * 100.0,
                        cpu_load.interrupt * 100.0,
                        cpu_load.idle * 100.0
                    );
                    usage = cpu_load.user;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(e);
                }
            }
        }
        Err(x) => {
            eprintln!("CPU load: error: {}", x);
            return Err(x);
        }
    }

    match sys.cpu_temp() {
        Ok(cpu_temp) => {
            println!("CPU temp: {}", cpu_temp);
            temp = cpu_temp;
        }
        Err(x) => {
            eprintln!("CPU temp: error: {}", x);
            return Err(x);
        }
    }

    Ok(proto::CpuChangeEvent {
        temp: temp,
        usage: usage,
    })
}

async fn get_ram_info(sys: &impl Platform) -> Result<proto::MemoryChangeEvent, std::io::Error> {
    match sys.memory() {
        Ok(mem) => {
            println!("Memory Total: {}, Free: {}", mem.total, mem.free);
            // mem.platform_memory
            //     .meminfo
            //     .into_iter()
            //     .for_each(|x| println!("{}: {}", x.0, x.1));

            Ok(proto::MemoryChangeEvent {
                free: mem.free.as_u64(),
                total: mem.total.as_u64(),
            })
        }
        Err(x) => {
            eprintln!("Memory load: error: {}", x);
            Err(x)
        }
    }
}

async fn get_disk_info(sys: &impl Platform) -> Result<Vec<proto::Mount>, std::io::Error> {
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
                .map(|fs| proto::Mount {
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
            eprintln!("Disk load: error: {}", x);
            Err(x)
        }
    }
}

async fn get_battery_info(
    sys: &impl Platform,
) -> Result<proto::BatteryChangeEvent, std::io::Error> {
    let on_ac;
    match sys.on_ac_power() {
        Ok(on_ac_power) => on_ac = on_ac_power,
        Err(err) => {
            println!("On AC load: error: {}", err);
            return Err(err);
        }
    }

    match sys.battery_life() {
        Ok(battery) => {
            println!(
                "Battery Life: Remain Capacity: {} Remaining Time: {} mins",
                battery.remaining_capacity,
                battery.remaining_time.as_secs() / 60,
            );
            Ok(proto::BatteryChangeEvent {
                remaining_capacity: battery.remaining_capacity,
                remaining_seconds: battery.remaining_time.as_secs(),
                power_connected: on_ac,
            })
        }
        Err(x) => {
            eprintln!("Battery load: error: {}", x);
            Err(x)
        }
    }
}

async fn get_system_info(sys: &impl Platform) -> Result<proto::SystemInfo, std::io::Error> {
    match sys.boot_time() {
        Ok(boot_time) => {
            println!("Boot Time: {}", boot_time);
            Ok(proto::SystemInfo {
                boot_time: Some(Timestamp {
                    seconds: boot_time.timestamp(),
                    // nanos are a bit too much
                    nanos: 0,
                }),
            })
        }
        Err(x) => {
            eprintln!("Boot Time: error: {}", x);
            Err(x)
        }
    }
}

async fn get_network_stats(
    sys: &impl Platform,
) -> Result<Vec<proto::NetworkDevice>, std::io::Error> {
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
                    proto::NetworkDevice {
                        name: name.clone(),
                        bytes_received: network.rx_bytes.as_u64(),
                        bytes_sent: network.tx_bytes.as_u64(),
                    }
                })
                .collect();
            Ok(device_stats)
        }
        Err(err) => {
            eprintln!("Network load: error: {}", err);
            Err(err)
        }
    }
}
