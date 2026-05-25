use crate::{EncodeError, EncodeMessage, MessageType, SliceMessageEncoder, Str};

/// Represents a request to `DBus` to occupy some name
pub struct RequestName<'a> {
    name: &'a str,
}

impl<'a> RequestName<'a> {
    /// Constructor for the slice-encoded message.
    #[must_use]
    pub const fn new(name: &'a str) -> Self {
        Self { name }
    }
}

impl EncodeMessage for RequestName<'_> {
    fn encoded_capacity(&self) -> usize {
        256usize.saturating_add(self.name.len())
    }

    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall, 0)?;
        encoder.set_path("/org/freedesktop/DBus")?;
        encoder.set_member("RequestName")?;
        encoder.set_interface("org.freedesktop.DBus")?;
        encoder.set_destination("org.freedesktop.DBus")?;
        encoder.set_body_signature("su")?;
        encoder.next_body_slot::<Str>()?.write(self.name)?;
        encoder.next_body_slot::<u32>()?.write(7)?;
        encoder.finish()
    }
}
