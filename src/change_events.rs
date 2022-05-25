#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CpuChangeEvent {
    #[prost(float, tag="1")]
    pub usage: f32,
    #[prost(float, tag="2")]
    pub temp: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MemoryChangeEvent {
    #[prost(uint64, tag="1")]
    pub total: u64,
    #[prost(uint64, tag="2")]
    pub free: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Mount {
    #[prost(string, tag="1")]
    pub device_name: ::prost::alloc::string::String,
    #[prost(string, tag="2")]
    pub mount_location: ::prost::alloc::string::String,
    #[prost(uint64, tag="3")]
    pub total: u64,
    #[prost(uint64, tag="4")]
    pub free: u64,
    #[prost(string, tag="5")]
    pub fs_type: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NetworkDevice {
    #[prost(string, tag="1")]
    pub name: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub bytes_received: u64,
    #[prost(uint64, tag="3")]
    pub bytes_sent: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BatteryChangeEvent {
    #[prost(uint64, tag="1")]
    pub remaining_seconds: u64,
    #[prost(float, tag="2")]
    pub remaining_capacity: f32,
    #[prost(bool, tag="3")]
    pub power_connected: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SystemInfo {
    #[prost(message, optional, tag="1")]
    pub boot_time: ::core::option::Option<::prost_types::Timestamp>,
}
