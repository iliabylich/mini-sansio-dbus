use crate::{EncodeError, EncodeMessage, MessageType, SliceMessageEncoder, Str};

/// Represents a request to get a single property of `DBus` object
pub struct GetProperty<'a> {
    destination: &'a str,
    path: &'a str,
    interface: &'a str,
    property: &'a str,
}

impl<'a> GetProperty<'a> {
    /// Constructor for the slice-encoded message.
    #[must_use]
    pub const fn new(
        destination: &'a str,
        path: &'a str,
        interface: &'a str,
        property: &'a str,
    ) -> Self {
        Self {
            destination,
            path,
            interface,
            property,
        }
    }
}

impl EncodeMessage for GetProperty<'_> {
    fn encoded_capacity(&self) -> usize {
        256usize
            .saturating_add(self.destination.len())
            .saturating_add(self.path.len())
            .saturating_add(self.interface.len())
            .saturating_add(self.property.len())
    }

    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall, 0)?;
        encoder.set_path(self.path)?;
        encoder.set_member("Get")?;
        encoder.set_interface("org.freedesktop.DBus.Properties")?;
        encoder.set_destination(self.destination)?;
        encoder.set_body_signature("ss")?;
        encoder.next_body_slot::<Str>()?.write(self.interface)?;
        encoder.next_body_slot::<Str>()?.write(self.property)?;
        encoder.finish()
    }
}
