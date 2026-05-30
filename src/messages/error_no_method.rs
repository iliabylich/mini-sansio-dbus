use crate::{
    EncodeError, MessageType, SliceMessageEncoder,
    const_helpers::{get_range, t_err},
    dbus_body,
};

/// Represents a "no such method" error reply
pub struct ErrorNoMethod;

impl ErrorNoMethod {
    /// Encodes "no such method" error reply into given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if given `buf` is too short
    pub const fn encode<'a>(
        buf: &'a mut [u8],
        destination: &str,
        reply_serial: u32,
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = t_err!(SliceMessageEncoder::new(buf, MessageType::Error));
        t_err!(encoder.set_error_name("org.freedesktop.DBus.Error.UnknownMethod"));
        t_err!(encoder.set_destination(destination));
        t_err!(encoder.set_reply_serial(reply_serial));
        dbus_body!(encoder, { str("Unknown method") });
        let len = t_err!(encoder.finish());
        let Some(buf) = get_range(buf, 0, len) else {
            return Err(EncodeError::BufferTooSmall);
        };
        Ok(buf)
    }
}
