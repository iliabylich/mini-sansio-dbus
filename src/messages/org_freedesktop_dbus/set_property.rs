use crate::{
    EncodeError, MessageType, SliceMessageEncoder, dbus_body_fragment, messaging::DBusEncode,
};

/// Represents a request to set a single property on a given `DBus` object
pub struct SetProperty;

impl DBusEncode for SetProperty {
    type Args<'a> = (
        &'a str,
        &'a str,
        &'a str,
        &'a str,
        &'a dyn Fn(&mut SliceMessageEncoder<'_>) -> Result<(), EncodeError>,
    );

    fn encode<'a>(
        (destination, path, interface, property, value): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodCall)?;
        encoder.set_path(path)?;
        encoder.set_member("Set")?;
        encoder.set_interface("org.freedesktop.DBus.Properties")?;
        encoder.set_destination(destination)?;
        encoder.set_body_signature("ssv")?;
        encoder.__dbus_begin_body()?;
        dbus_body_fragment!(encoder, {
            str(interface),
            str(property),
        });
        (value)(&mut encoder)?;

        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}
