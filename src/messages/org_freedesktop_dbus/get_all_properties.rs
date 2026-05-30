use crate::{
    EncodeError, MessageType, SliceMessageEncoder,
    const_helpers::{get_range, t_err},
    dbus_body,
};

/// Represents a request to get all object properties
pub struct GetAllProperties;

impl GetAllProperties {
    /// Writes a "get" message to a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given buffer.
    pub const fn encode<'a>(
        buf: &'a mut [u8],
        destination: &str,
        path: &str,
        interface: &str,
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = t_err!(SliceMessageEncoder::new(buf, MessageType::MethodCall));
        t_err!(encoder.set_path(path));
        t_err!(encoder.set_member("GetAll"));
        t_err!(encoder.set_interface("org.freedesktop.DBus.Properties"));
        t_err!(encoder.set_destination(destination));
        dbus_body!(encoder, {
            str(interface),
        });
        let len = t_err!(encoder.finish());
        let Some(buf) = get_range(buf, 0, len) else {
            return Err(EncodeError::BufferTooSmall);
        };
        Ok(buf)
    }
}
