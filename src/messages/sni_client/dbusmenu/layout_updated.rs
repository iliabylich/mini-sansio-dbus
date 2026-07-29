use crate::{EncodeError, MessageType, SliceMessageEncoder, messaging::DBusEncode};

/// Emits `LayoutUpdated` for a `com.canonical.dbusmenu` object.
pub struct LayoutUpdatedSignal;

impl DBusEncode for LayoutUpdatedSignal {
    type Args<'a> = (&'a str, u32, i32);

    fn encode<'a>(
        (path, revision, parent): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::Signal)?;
        encoder.set_path(path)?;
        encoder.set_interface("com.canonical.dbusmenu")?;
        encoder.set_member("LayoutUpdated")?;
        encoder.set_body_signature("ui")?;
        encoder.__dbus_begin_body()?;
        encoder.__dbus_write_u32(revision)?;
        encoder.__dbus_write_i32(parent)?;
        let len = encoder.finish()?;
        buf.get(..len).ok_or(EncodeError::BufferTooSmall)
    }
}
