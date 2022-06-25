use super::proto::change_event::Event;
use super::proto::{ChangeEvent, EventType};

pub fn process_event(event: &ChangeEvent) {
    match &event.event {
        Some(event) => match event {
            Event::Cpu(cpu) => println!("CPU event"),
            Event::Memory(mem) => println!("Memory event"),
            Event::Mount(mount) => println!("Mount event"),
            Event::NetworkDevice(net_device) => {}
            Event::SystemInfo(info) => println!("System info event"),
            Event::Battery(battery) => println!("Battery event"),
        },
        None => {}
    }

    match EventType::from_i32(event.event_type) {
        Some(EventType::Add) => {}
        Some(EventType::Update) => {}
        Some(EventType::Delete) => {}
        None => {}
    }

    println!("Got event: {:?}", event);
}
