use crate::{EncodeError, MessageType, SliceMessageEncoder, dbus_body};

/// Represents a request to unsubscriibe from some `DBus` changes. The opposite of `AddMatch`.
pub struct RemoveMatch;

impl RemoveMatch {
    /// Writes a "remove match" message to a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given buffer.
    pub fn encode<'a>(buf: &'a mut [u8], rule: &str) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_path("/org/freedesktop/DBus")?;
        encoder.set_member("RemoveMatch")?;
        encoder.set_interface("org.freedesktop.DBus")?;
        encoder.set_destination("org.freedesktop.DBus")?;
        dbus_body!(encoder, {
            str(rule),
        });
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}
