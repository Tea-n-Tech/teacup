extern crate tokio;

use prost_types::Timestamp;
use std::time;
use systemstat::Platform;

use crate::change_events::{
    BatteryChangeEvent, CpuChangeEvent, MemoryChangeEvent, Mount, NetworkDevice, SystemInfo,
};

pub async fn get_cpu_info(sys: &impl Platform) -> Result<CpuChangeEvent, std::io::Error> {
    let mut usage: f32 = 0.;
    let mut temp: f32 = 0.;

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
                    println!("Error: {}", e);
                    return Err(e);
                }
            }
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

pub async fn get_ram_info(sys: &impl Platform) -> Result<MemoryChangeEvent, std::io::Error> {
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

pub async fn get_disk_info(sys: &impl Platform) -> Result<Vec<Mount>, std::io::Error> {
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

pub async fn get_battery_info(sys: &impl Platform) -> Result<BatteryChangeEvent, std::io::Error> {
    let mut on_ac = true;
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
            Ok(BatteryChangeEvent {
                remaining_capacity: battery.remaining_capacity,
                remaining_seconds: battery.remaining_time.as_secs(),
                power_connected: on_ac,
            })
        }
        Err(x) => {
            println!("Battery load: error: {}", x);
            Err(x)
        }
    }
}

pub async fn get_system_info(sys: &impl Platform) -> Result<SystemInfo, std::io::Error> {
    match sys.boot_time() {
        Ok(boot_time) => {
            println!("Boot Time: {}", boot_time);
            Ok(SystemInfo {
                boot_time: Some(Timestamp {
                    seconds: boot_time.timestamp(),
                    // nanos are a bit too much
                    nanos: 0,
                }),
            })
        }
        Err(x) => {
            println!("Boot Time: error: {}", x);
            Err(x)
        }
    }
}

pub async fn get_network_stats(sys: &impl Platform) -> Result<Vec<NetworkDevice>, std::io::Error> {
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
