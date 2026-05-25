use crate::{EncodeError, EncodeMessage, MessageType, SliceMessageEncoder, dbus_body};

/// Represents a request to subscribe to something in `DBus`
pub struct AddMatch<'a> {
    rule: &'a str,
}

impl<'a> AddMatch<'a> {
    /// Low-level constructor for the slice-encoded message.
    #[must_use]
    pub const fn new_from_rule(rule: &'a str) -> Self {
        Self { rule }
    }
}

impl EncodeMessage for AddMatch<'_> {
    fn encoded_capacity(&self) -> usize {
        256usize.saturating_add(self.rule.len())
    }

    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall, 0)?;
        encoder.set_path("/org/freedesktop/DBus")?;
        encoder.set_member("AddMatch")?;
        encoder.set_interface("org.freedesktop.DBus")?;
        encoder.set_destination("org.freedesktop.DBus")?;
        dbus_body!(encoder, {
            str(self.rule),
        });
        encoder.finish()
    }
}
