#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CpuChangeEvent {
    #[prost(float, tag="1")]
    pub usage: f32,
}
