use crate::{EncodeError, SliceMessageEncoder};

/// SNI icon pixmap in ARGB format
#[derive(Debug, Clone, Copy)]
pub struct IconPixmap<'a> {
    /// Width in pixels
    pub width: i32,
    /// Height in pixels
    pub height: i32,
    /// ARGB bytes, must have `width * height * 4` items
    pub argb: &'a [u8],
}

impl IconPixmap<'_> {
    pub(crate) fn encode(
        encoder: &mut SliceMessageEncoder<'_>,
        pixmap: Option<Self>,
    ) -> Result<(), EncodeError> {
        let array = encoder.__dbus_start_array(8)?;
        if let Some(Self {
            width,
            height,
            argb,
        }) = pixmap
        {
            encoder.__dbus_align(8)?;
            encoder.__dbus_write_i32(width)?;
            encoder.__dbus_write_i32(height)?;
            let bytes = encoder.__dbus_start_array(1)?;
            for byte in argb {
                encoder.__dbus_write_u8(*byte)?;
            }
            encoder.__dbus_finish_array(bytes)?;
        }
        encoder.__dbus_finish_array(array)
    }
}
