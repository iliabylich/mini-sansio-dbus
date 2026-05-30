use crate::{EncodeError, MessageType, SliceMessageEncoder, const_helpers::t_err, dbus_body};

/// Represents a request to get a single property of `DBus` object
pub struct GetProperty;

impl GetProperty {
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
        property: &str,
    ) -> Result<usize, EncodeError> {
        let mut encoder = t_err!(SliceMessageEncoder::new(buf, MessageType::MethodCall));
        t_err!(encoder.set_path(path));
        t_err!(encoder.set_member("Get"));
        t_err!(encoder.set_interface("org.freedesktop.DBus.Properties"));
        t_err!(encoder.set_destination(destination));
        dbus_body!(encoder, {
            str(interface),
            str(property),
        });
        encoder.finish()
    }
}
