use std::hash::Hash;

tonic::include_proto!("change_events");

pub trait ToEvent {
    fn to_change_event(&self, event_type: EventType) -> ChangeEvent;
}

macro_rules! impl_to_event {
    ($a: tt) => {
        impl ToEvent for $a {
            fn to_change_event(&self, event_type: EventType) -> ChangeEvent {
                ChangeEvent {
                    event_type: event_type.into(),
                    event: Some(change_event::Event::$a(self.clone())),
                }
            }
        }
    };
}

impl_to_event!(NetworkDevice);
impl_to_event!(Mount);

impl Eq for NetworkDevice {}

// As we cannot apply the deconstruction Macro on protobuf
// entities since they are automatically generated, we do
// it here manually.
impl<'a, R: ::sqlx::Row> ::sqlx::FromRow<'a, R> for NetworkDevice
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    i64: ::sqlx::decode::Decode<'a, R::Database>,
    i64: ::sqlx::types::Type<R::Database>,
    i64: ::sqlx::decode::Decode<'a, R::Database>,
    i64: ::sqlx::types::Type<R::Database>,
{
    fn from_row(row: &'a R) -> ::sqlx::Result<Self> {
        let name: String = row.try_get("device_name")?;
        let bytes_received: i64 = row.try_get("bytes_received")?;
        let bytes_sent: i64 = row.try_get("bytes_sent")?;
        ::std::result::Result::Ok(NetworkDevice {
            name,
            bytes_received,
            bytes_sent,
        })
    }
}

// As we cannot apply the deconstruction Macro on protobuf
// entities since they are automatically generated, we do
// it here manually.
impl<'a, R: ::sqlx::Row> ::sqlx::FromRow<'a, R> for Mount
where
    &'a ::std::primitive::str: ::sqlx::ColumnIndex<R>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    String: ::sqlx::decode::Decode<'a, R::Database>,
    String: ::sqlx::types::Type<R::Database>,
    i64: ::sqlx::decode::Decode<'a, R::Database>,
    i64: ::sqlx::types::Type<R::Database>,
    i64: ::sqlx::decode::Decode<'a, R::Database>,
    i64: ::sqlx::types::Type<R::Database>,
{
    fn from_row(row: &'a R) -> ::sqlx::Result<Self> {
        let device_name: String = row.try_get("device_name")?;
        let mount_location: String = row.try_get("mount_location")?;
        let fs_type: String = row.try_get("fs_type")?;
        let free: i64 = row.try_get("free")?;
        let total: i64 = row.try_get("total")?;
        ::std::result::Result::Ok(Mount {
            device_name,
            mount_location,
            fs_type,
            free,
            total,
        })
    }
}
