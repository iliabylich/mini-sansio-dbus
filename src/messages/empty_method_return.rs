use crate::{EncodeError, MessageType, SliceMessageEncoder, const_helpers::try_};

/// Represents an empty method return, with no body
pub struct EmptyMethodReturn;

impl EmptyMethodReturn {
    /// Encodes "empty method return" reply into given buffer.
    ///
    /// # Errors
    ///
    /// Returns an error if given `buf` is too short
    pub const fn encode(
        buf: &mut [u8],
        destination: &str,
        reply_serial: u32,
    ) -> Result<usize, EncodeError> {
        let mut encoder = try_!(SliceMessageEncoder::new(
            buf,
            MessageType::MethodReturn,
            reply_serial
        ));
        try_!(encoder.set_destination(destination));
        try_!(encoder.set_reply_serial(reply_serial));
        try_!(encoder.set_body_signature(""));
        encoder.finish()
    }
}
