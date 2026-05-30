use crate::{EncodeError, MessageType, SliceMessageEncoder, const_helpers::t_err};

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
        let mut encoder = t_err!(SliceMessageEncoder::new(
            buf,
            MessageType::MethodReturn,
            reply_serial
        ));
        t_err!(encoder.set_destination(destination));
        t_err!(encoder.set_reply_serial(reply_serial));
        t_err!(encoder.set_body_signature(""));
        encoder.finish()
    }
}
