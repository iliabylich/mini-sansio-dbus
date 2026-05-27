use crate::{EncodeError, MessageType, SliceMessageEncoder, const_helpers::try_, dbus_body};

/// Represents a request to get all object properties
pub struct GetAllProperties;

impl GetAllProperties {
    /// Writes a "get" message to a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given buffer.
    pub const fn encode(
        buf: &mut [u8],
        destination: &str,
        path: &str,
        interface: &str,
    ) -> Result<usize, EncodeError> {
        let mut encoder = try_!(SliceMessageEncoder::new(buf, MessageType::MethodCall, 0));
        try_!(encoder.set_path(path));
        try_!(encoder.set_member("GetAll"));
        try_!(encoder.set_interface("org.freedesktop.DBus.Properties"));
        try_!(encoder.set_destination(destination));
        dbus_body!(encoder, {
            str(interface),
        });
        encoder.finish()
    }
}
