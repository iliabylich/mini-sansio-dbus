use crate::{
    EncodeError, MessageType, OutgoingQueue, SliceMessageEncoder, dbus_body, messaging::DBusEncode,
};

/// Helper type to send an `Event` method call
pub struct Event;

pub struct EventArgs<'a> {
    id: i32,
    timestamp: u32,
    destination: &'a str,
    path: &'a str,
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

impl Event {
    /// Sends an `Event` method call
    ///
    /// # Errors
    ///
    /// Returns an error if encoded message doesn't fit into a given `buf`
    pub fn send<Q>(
        buf: &mut [u8],
        q: &mut Q,
        id: i32,
        timestamp: u32,
        destination: &str,
        path: &str,
    ) -> Result<(), EncodeError>
    where
        Q: OutgoingQueue,
    {
        let buf = Self::encode(
            EventArgs {
                id,
                timestamp,
                destination,
                path,
            },
            buf,
        )?;
        q.push_raw_buf(buf);
        Ok(())
    }
}
