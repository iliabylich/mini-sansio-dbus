use crate::{EncodeError, MessageType, SliceMessageEncoder, dbus_body};

/// Represents a request to `DBus` to occupy some name
pub struct RequestName;

impl RequestName {
    /// Writes a "request name" message to a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given buffer.
    pub fn encode<'a>(buf: &'a mut [u8], name: &str) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_path("/org/freedesktop/DBus")?;
        encoder.set_member("RequestName")?;
        encoder.set_interface("org.freedesktop.DBus")?;
        encoder.set_destination("org.freedesktop.DBus")?;
        dbus_body!(encoder, {
            str(name),
            u32(7),
        });
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}
