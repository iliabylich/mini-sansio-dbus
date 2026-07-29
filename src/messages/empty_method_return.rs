use crate::{EncodeError, MessageType, SliceMessageEncoder, messaging::DBusEncode};

/// Represents an empty method return, with no body
pub struct EmptyMethodReturn;

impl DBusEncode for EmptyMethodReturn {
    type Args<'a> = (&'a str, u32);

    fn encode<'a>(
        (destination, reply_serial): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodReturn)?;
        encoder.set_destination(destination)?;
        encoder.set_reply_serial(reply_serial)?;
        encoder.set_body_signature("")?;
        let len = encoder.finish()?;
        buf.get(0..len).ok_or(EncodeError::BufferTooSmall)
    }
}
