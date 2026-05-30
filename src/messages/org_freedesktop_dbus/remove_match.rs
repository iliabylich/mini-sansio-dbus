use crate::{
    EncodeError, MessageType, SliceMessageEncoder,
    const_helpers::{get_range, t_err},
    dbus_body,
};

/// Represents a request to unsubscriibe from some `DBus` changes. The opposite of `AddMatch`.
pub struct RemoveMatch;

impl RemoveMatch {
    /// Writes a "remove match" message to a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given buffer.
    pub const fn encode<'a>(buf: &'a mut [u8], rule: &str) -> Result<&'a [u8], EncodeError> {
        let mut encoder = t_err!(SliceMessageEncoder::new(buf, MessageType::MethodCall));
        t_err!(encoder.set_path("/org/freedesktop/DBus"));
        t_err!(encoder.set_member("RemoveMatch"));
        t_err!(encoder.set_interface("org.freedesktop.DBus"));
        t_err!(encoder.set_destination("org.freedesktop.DBus"));
        dbus_body!(encoder, {
            str(rule),
        });
        let len = t_err!(encoder.finish());
        let Some(buf) = get_range(buf, 0, len) else {
            return Err(EncodeError::BufferTooSmall);
        };
        Ok(buf)
    }
}
