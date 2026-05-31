use crate::{EncodeError, MessageType, SliceMessageEncoder};

/// Represents an empty method return, with no body
pub struct EmptyMethodReturn;

impl EmptyMethodReturn {
    /// Encodes "empty method return" reply into given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if given `buf` is too short
    pub fn encode<'a>(
        buf: &'a mut [u8],
        destination: &str,
        reply_serial: u32,
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodReturn)?;
        encoder.set_destination(destination)?;
        encoder.set_reply_serial(reply_serial)?;
        encoder.set_body_signature("")?;
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}
