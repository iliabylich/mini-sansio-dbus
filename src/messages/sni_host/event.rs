use crate::{EncodeError, MessageType, SliceMessageEncoder, dbus_body, messaging::DBusEncode};

/// Helper type to send an `Event` method call
pub struct Event;

/// Arguments of the `Event` call
pub struct EventArgs<'a> {
    /// ID to trigger on
    pub id: i32,
    /// Timestamp
    pub timestamp: u32,
    /// Destination of the receiver
    pub destination: &'a str,
    /// Path of the receiver
    pub path: &'a str,
}

impl DBusEncode for Event {
    type Args<'a> = EventArgs<'a>;

    fn encode<'a>(
        EventArgs {
            id,
            timestamp,
            destination,
            path,
        }: Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_destination(destination.as_ref())?;
        encoder.set_path(path.as_ref())?;
        encoder.set_interface("com.canonical.dbusmenu")?;
        encoder.set_member("Event")?;
        dbus_body!(&mut encoder, {
            i32(id),
            str("clicked"),
            variant<i32>(0),
            u32(timestamp),
        });
        let len = encoder.finish()?;
        let buf = buf.get(..len).ok_or(EncodeError::BufferTooSmall)?;
        Ok(buf)
    }
}
