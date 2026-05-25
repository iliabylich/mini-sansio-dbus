use crate::{EncodeError, MessageType, SliceMessageEncoder, const_helpers::try_, dbus_body};

/// Represents a request to unsubscriibe from some `DBus` changes. The opposite of `AddMatch`.
pub struct RemoveMatch;

impl RemoveMatch {
    /// Writes a "remove match" message to a given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given buffer.
    pub const fn encode(buf: &mut [u8], rule: &str) -> Result<usize, EncodeError> {
        let mut encoder = try_!(SliceMessageEncoder::new(buf, MessageType::MethodCall, 0));
        try_!(encoder.set_path("/org/freedesktop/DBus"));
        try_!(encoder.set_member("RemoveMatch"));
        try_!(encoder.set_interface("org.freedesktop.DBus"));
        try_!(encoder.set_destination("org.freedesktop.DBus"));
        dbus_body!(encoder, {
            str(rule),
        });
        encoder.finish()
    }
}
