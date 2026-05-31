use crate::{EncodeError, MessageType, SliceMessageEncoder, dbus_body};

/// Represents a "no such method" error reply
pub struct ErrorNoMethod;

impl ErrorNoMethod {
    /// Encodes "no such method" error reply into given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if given `buf` is too short
    pub fn encode<'a>(
        buf: &'a mut [u8],
        destination: &str,
        reply_serial: u32,
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
