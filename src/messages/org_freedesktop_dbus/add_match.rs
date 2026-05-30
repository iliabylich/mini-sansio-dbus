use crate::{EncodeError, MessageType, SliceMessageEncoder, const_helpers::t_err, dbus_body};

/// Represents a request to subscribe to something in `DBus`
pub struct AddMatch;

impl AddMatch {
    /// Writes an "add match" message to a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given buffer.
    pub const fn encode(buf: &mut [u8], rule: &str) -> Result<usize, EncodeError> {
        let mut encoder = t_err!(SliceMessageEncoder::new(buf, MessageType::MethodCall, 0));
        t_err!(encoder.set_path("/org/freedesktop/DBus"));
        t_err!(encoder.set_member("AddMatch"));
        t_err!(encoder.set_interface("org.freedesktop.DBus"));
        t_err!(encoder.set_destination("org.freedesktop.DBus"));
        dbus_body!(encoder, {
            str(rule),
        });
        encoder.finish()
    }
}
