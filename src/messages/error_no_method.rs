use crate::{EncodeError, MessageType, SliceMessageEncoder, const_helpers::try_, dbus_body};

/// Represents a "no such method" error reply
pub struct ErrorNoMethod;

impl ErrorNoMethod {
    /// Encodes "no such method" error reply into given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if given `buf` is too short
    pub const fn encode(
        buf: &mut [u8],
        destination: &str,
        reply_serial: u32,
    ) -> Result<usize, EncodeError> {
        let mut encoder = try_!(SliceMessageEncoder::new(
            buf,
            MessageType::Error,
            reply_serial
        ));
        try_!(encoder.set_error_name("org.freedesktop.DBus.Error.UnknownMethod"));
        try_!(encoder.set_destination(destination));
        try_!(encoder.set_reply_serial(reply_serial));
        dbus_body!(encoder, { str("Unknown method") });
        encoder.finish()
    }
}
