use crate::{EncodeError, EncodeMessage, MessageType, SliceMessageEncoder, Str};

/// Represents a request to unsubscriibe from some `DBus` changes. The opposite of `AddMatch`.
pub struct RemoveMatch<'a> {
    rule: &'a str,
}

impl<'a> RemoveMatch<'a> {
    /// Low-level constructor for the slice-encoded message.
    #[must_use]
    pub const fn new_from_rule(rule: &'a str) -> Self {
        Self { rule }
    }
}

impl EncodeMessage for RemoveMatch<'_> {
    fn encoded_capacity(&self) -> usize {
        256usize.saturating_add(self.rule.len())
    }

    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall, 0)?;
        encoder.set_path("/org/freedesktop/DBus")?;
        encoder.set_member("RemoveMatch")?;
        encoder.set_interface("org.freedesktop.DBus")?;
        encoder.set_destination("org.freedesktop.DBus")?;
        encoder.set_body_signature("s")?;
        encoder.next_body_slot::<Str>()?.write(self.rule)?;
        encoder.finish()
    }
}
