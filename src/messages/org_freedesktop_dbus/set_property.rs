use crate::{EncodeError, EncodeMessage, MessageType, SliceMessageEncoder, dbus_body_fragment};

/// Represents a request to set a single property on a given `DBus` object
pub struct SetProperty<'a, F> {
    destination: &'a str,
    path: &'a str,
    interface: &'a str,
    property: &'a str,
    value_capacity: usize,
    write_value: F,
}

impl<'a, F> SetProperty<'a, F> {
    /// Constructor for the slice-encoded message.
    #[must_use]
    pub const fn new(
        destination: &'a str,
        path: &'a str,
        interface: &'a str,
        property: &'a str,
        value_capacity: usize,
        write_value: F,
    ) -> Self {
        Self {
            destination,
            path,
            interface,
            property,
            value_capacity,
            write_value,
        }
    }
}

impl<F> EncodeMessage for SetProperty<'_, F>
where
    F: Fn(&mut SliceMessageEncoder<'_>) -> Result<(), EncodeError>,
{
    fn encoded_capacity(&self) -> usize {
        256usize
            .saturating_add(self.destination.len())
            .saturating_add(self.path.len())
            .saturating_add(self.interface.len())
            .saturating_add(self.property.len())
            .saturating_add(self.value_capacity)
    }

    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall, 0)?;
        encoder.set_path(self.path)?;
        encoder.set_member("Set")?;
        encoder.set_interface("org.freedesktop.DBus.Properties")?;
        encoder.set_destination(self.destination)?;
        encoder.set_body_signature("ssv")?;
        encoder.__dbus_begin_body()?;
        dbus_body_fragment!(encoder, {
            str(self.interface),
            str(self.property),
        });
        (self.write_value)(&mut encoder)?;

        encoder.finish()
    }
}
