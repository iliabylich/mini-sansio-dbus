use core::marker::PhantomData;

use crate::{
    DbusType, EncodeError, EncodeMessage, MessageType, SliceMessageEncoder, Str,
    encoder::{Variant, VariantSlot},
};

/// Represents a request to set a single property on a given `DBus` object
pub struct SetProperty<'a, T, F> {
    destination: &'a str,
    path: &'a str,
    interface: &'a str,
    property: &'a str,
    value_capacity: usize,
    write_value: F,
    _ty: PhantomData<T>,
}

impl<'a, T, F> SetProperty<'a, T, F> {
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
            _ty: PhantomData,
        }
    }
}

impl<T, F> EncodeMessage for SetProperty<'_, T, F>
where
    T: DbusType,
    F: for<'slot, 'buf> Fn(VariantSlot<'slot, 'buf, T>) -> Result<(), EncodeError>,
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
        encoder.next_body_slot::<Str>()?.write(self.interface)?;
        encoder.next_body_slot::<Str>()?.write(self.property)?;

        {
            let variant = encoder.next_body_slot::<Variant<T>>()?;
            (self.write_value)(variant)?;
        }

        encoder.finish()
    }
}
