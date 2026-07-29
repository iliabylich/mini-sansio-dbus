use crate::{EncodeError, MessageType, SliceMessageEncoder, messaging::DBusEncode};

/// Emits `NewIcon`, notifying hosts to re-read `IconName` and/or `IconPixmap`.
pub struct NewIconSignal;

impl DBusEncode for NewIconSignal {
    type Args<'a> = ();

    fn encode<'a>((): Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::Signal)?;
        encoder.set_path("/StatusNotifierItem")?;
        encoder.set_interface("org.kde.StatusNotifierItem")?;
        encoder.set_member("NewIcon")?;
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}
