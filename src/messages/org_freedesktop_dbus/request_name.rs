use crate::{EncodeError, MessageType, SliceMessageEncoder, const_helpers::try_, dbus_body};

/// Represents a request to `DBus` to occupy some name
pub struct RequestName;

impl RequestName {
    /// Writes a "request name" message to a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given buffer.
    pub const fn encode(buf: &mut [u8], name: &str) -> Result<usize, EncodeError> {
        let mut encoder = try_!(SliceMessageEncoder::new(buf, MessageType::MethodCall, 0));
        try_!(encoder.set_path("/org/freedesktop/DBus"));
        try_!(encoder.set_member("RequestName"));
        try_!(encoder.set_interface("org.freedesktop.DBus"));
        try_!(encoder.set_destination("org.freedesktop.DBus"));
        dbus_body!(encoder, {
            str(name),
            u32(7),
        });
        encoder.finish()
    }
}
