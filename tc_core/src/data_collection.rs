extern crate num_cpus;
extern crate protocol as proto;
extern crate systemstat;
extern crate tokio;

use std::collections::HashMap;

use prost_types::Timestamp;
use systemstat::Platform;
use systemstat::System;
use tokio::sync::mpsc;
use tokio::time;

pub async fn get_change_events<T: proto::ToEvent + std::cmp::PartialEq>(
    prev_devices: &HashMap<String, T>,
    new_devices: &HashMap<String, T>,
) -> Vec<proto::ChangeEvent> {
    let mut events: Vec<proto::ChangeEvent> = Vec::new();

    let mut all_device_names: Vec<&String> = vec![];
    all_device_names.extend(prev_devices.keys());
    all_device_names.extend(new_devices.keys());

    for device_name in all_device_names {
        let prev_device = prev_devices.get(device_name);
        let new_device = new_devices.get(device_name);

        // new device found -> ADD
        if prev_device.is_none() && new_device.is_some() {
            events.push(new_device.unwrap().to_change_event(proto::EventType::Add));
        }

        // existing device changed -> UPDATE
        if prev_device.is_some() && new_device.is_some() && prev_device != new_device {
            events.push(
                new_device
                    .unwrap()
                    .to_change_event(proto::EventType::Update),
            );
        }

        // old device gone -> DELETE
        if prev_device.is_some() && new_device.is_none() {
            events.push(
                prev_device
                    .unwrap()
                    .to_change_event(proto::EventType::Delete),
            );
        }
    }

    events
}

pub async fn collect_events(
    tx: mpsc::Sender<proto::ChangeEventBatch>,
    initial_state: proto::InitialStateResponse,
    machine_id: i64,
) {
    let forever = tokio::task::spawn(async move {
        let sys = &System::new();
        // TODO make configurable
        let mut interval = time::interval(time::Duration::from_secs(5));

        // Get previous data so that we only send changes which happened
        // in the meantime
        let mut previous_mounts: HashMap<String, _> = initial_state
            .mounts
            .iter()
            .map(|x| (x.device_name.clone(), x.clone()))
            .collect();
        let mut previous_network_devices: HashMap<String, _> = initial_state
            .network_devices
            .iter()
            .map(|x| (x.name.clone(), x.clone()))
            .collect();

        // Do a looping ... wheee
        // Don't do that at home
        loop {
            interval.tick().await;

            let mut events: Vec<proto::ChangeEvent> = vec![];

            // cpu
            let cpu_info = get_cpu_update_event(sys).await;
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
                    let mount_events = get_change_events(&previous_mounts, &mounts).await;
                    previous_mounts = mounts;
                    events.extend(mount_events);
                }
                Err(err) => {
                    eprintln!("Error getting disk info: {:?}", err);
                }
            }

            // network
            match get_network_stats(sys).await {
                Ok(network_devices) => {
                    let network_events =
                        get_change_events(&previous_network_devices, &network_devices).await;
                    events.extend(network_events);
                    previous_network_devices = network_devices;
                }
                Err(err) => {
                    eprintln!("Error getting network info: {:?}", err);
                }
            }

            // Send stuff to the server
            match tx
                .send(proto::ChangeEventBatch { machine_id, events })
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    eprintln!("Error sending batch of events: {:?}", e);
                }
            }
        }
    });

    match forever.await {
        Ok(_) => {}
        Err(e) => eprintln!("Error collecting events: {}", e),
    }
}

pub async fn get_initial_state(machine_id: i64) -> proto::InitialStateRequest {
    let sys = System::new();

    proto::InitialStateRequest {
        machine_id,
        system_info: Some(get_system_info(&sys).await),
        cpu_info: Some(get_cpu_info().await),
    }
}

async fn get_cpu_info() -> proto::CpuInfo {
    let n_logical_cpus = num_cpus::get();
    let n_logical_cpus_i64 = match i64::try_from(n_logical_cpus) {
        Ok(n) => n,
        Err(err) => {
            eprintln!(
                "Error converting cpu count '{}' from usize to i64: {:?}",
                n_logical_cpus, err
            );
            0
        }
    };

    proto::CpuInfo {
        n_cores: n_logical_cpus_i64,
    }
}

async fn get_cpu_update_event(sys: &impl Platform) -> proto::CpuChangeEvent {
    // Note: we don't return result here since temp is expected to fail often
    // since measuring temp is badly supported. Subsequently, this routine
    // would fail quite a lot, thus we are a bit more forgiving here.

    let usage = match sys.cpu_load_aggregate() {
        Ok(cpu) => {
            // Measuring CPU load requires sleeping for a brief moment
            tokio::time::sleep(time::Duration::from_secs(1)).await;

            match cpu.done() {
                Ok(cpu_load) => {
                    eprintln!(
                        "CPU load: {}% user, {}% nice, {}% system, {}% intr, {}% idle ",
                        cpu_load.user * 100.0,
                        cpu_load.nice * 100.0,
                        cpu_load.system * 100.0,
                        cpu_load.interrupt * 100.0,
                        cpu_load.idle * 100.0
                    );
                    cpu_load.user
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    0.
                }
            }
        }
        Err(x) => {
            eprintln!("CPU load: error: {}", x);
            0.
        }
    };

    let temp = match sys.cpu_temp() {
        Ok(cpu_temp) => {
            eprintln!("CPU temp: {}", cpu_temp);
            cpu_temp
        }
        Err(err) => {
            eprintln!("CPU temp: error: {}", err);
            0.
        }
    };

    proto::CpuChangeEvent { temp, usage }
}

async fn get_ram_info(sys: &impl Platform) -> Result<proto::MemoryChangeEvent, std::io::Error> {
    match sys.memory() {
        Ok(mem) => {
            eprintln!("Memory Total: {}, Free: {}", mem.total, mem.free);

            Ok(proto::MemoryChangeEvent {
                free: u64_to_i64_or_default_and_log(mem.free.as_u64()),
                total: u64_to_i64_or_default_and_log(mem.total.as_u64()),
            })
        }
        Err(x) => {
            eprintln!("Memory load: error: {}", x);
            Err(x)
        }
    }
}

async fn get_disk_info(
    sys: &impl Platform,
) -> Result<HashMap<String, proto::Mount>, std::io::Error> {
    // We are only interested in the most common fs types.
    // I hope I don't exclude anything important 🫣
    const ALLOWED_MOUNT_TYPES: &[&str] = &["ext", "ntfs", "vfat", "btrfs", "xfs"];

    match sys.mounts() {
        Ok(mounts) => {
            let mount_vec = mounts
                .iter()
                .filter(|fs| {
                    ALLOWED_MOUNT_TYPES
                        .iter()
                        .any(|name| fs.fs_type.as_str().starts_with(name))
                })
                .map(|fs| {
                    eprintln!(
                        "{} -> {} ({}) {}/{} free",
                        fs.fs_mounted_from, fs.fs_mounted_on, fs.fs_type, fs.avail, fs.total
                    );
                    (
                        fs.fs_mounted_from.clone(),
                        proto::Mount {
                            device_name: fs.fs_mounted_from.clone(),
                            mount_location: fs.fs_mounted_on.clone(),
                            free: u64_to_i64_or_default_and_log(fs.avail.as_u64()),
                            total: u64_to_i64_or_default_and_log(fs.total.as_u64()),
                            fs_type: fs.fs_type.clone(),
                        },
                    )
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

pub async fn get_system_info(sys: &impl Platform) -> proto::SystemInfo {
    let mut boot_time: i64 = 0;
    match sys.boot_time() {
        Ok(new_boot_time) => {
            eprintln!("Boot Time: {}", new_boot_time);
            boot_time = new_boot_time.timestamp();
        }
        Err(x) => {
            eprintln!("Boot Time: error: {}", x);
        }
    }

    proto::SystemInfo {
        boot_time: Some(Timestamp {
            seconds: boot_time,
            // nanos are a bit too much
            nanos: 0,
        }),
    }
}

async fn get_network_stats(
    sys: &impl Platform,
) -> Result<HashMap<String, proto::NetworkDevice>, std::io::Error> {
    match sys.networks() {
        Ok(networks) => {
            let device_stats: HashMap<_, _> = networks
                .iter()
                .map(|network| network.0)
                .map(|name| {
                    let network = sys.network_stats(name).unwrap();
                    eprintln!(
                        "{}: sent: {}, recv: {}",
                        name, network.tx_bytes, network.rx_bytes
                    );
                    (
                        name.clone(),
                        proto::NetworkDevice {
                            name: name.clone(),
                            bytes_received: u64_to_i64_or_default_and_log(
                                network.rx_bytes.as_u64(),
                            ),
                            bytes_sent: u64_to_i64_or_default_and_log(network.tx_bytes.as_u64()),
                        },
                    )
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

fn u64_to_i64_or_default_and_log(number: u64) -> i64 {
    match i64::try_from(number) {
        Ok(number) => number,
        Err(err) => {
            // Unfortunately, sqlx does not support u64
            // so there is not much we can do here.
            // It is important to see in the logs how often this
            // occurs.
            eprintln!("Failed to convert u64 {} to i64: {}", number, err);
            0
        }
    }
}
