use crate::{EncodeError, MessageType, SliceMessageEncoder, dbus_body, messaging::DBusEncode};

/// Represents a "no such method" error reply
pub struct ErrorNoMethod;

impl DBusEncode for ErrorNoMethod {
    type Args<'a> = (&'a str, u32);

    fn encode<'a>(
        (destination, reply_serial): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::Error)?;
        encoder.set_error_name("org.freedesktop.DBus.Error.UnknownMethod")?;
        encoder.set_destination(destination)?;
        encoder.set_reply_serial(reply_serial)?;
        dbus_body!(encoder, { str("Unknown method") });
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}
