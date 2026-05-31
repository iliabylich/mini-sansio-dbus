use crate::{EncodeError, MessageType, SliceMessageEncoder, dbus_body};

/// Represents a request to get all object properties
pub struct GetAllProperties;

impl GetAllProperties {
    /// Writes a "get" message to a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given buffer.
    pub fn encode<'a>(
        buf: &'a mut [u8],
        destination: &str,
        path: &str,
        interface: &str,
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_path(path)?;
        encoder.set_member("GetAll")?;
        encoder.set_interface("org.freedesktop.DBus.Properties")?;
        encoder.set_destination(destination)?;
        dbus_body!(encoder, {
            str(interface),
        });
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}
