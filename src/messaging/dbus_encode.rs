use crate::EncodeError;

/// A common trait for all messages that can be encoded into a buffer
pub trait DBusEncode {
    /// Encoding arguments
    type Args;

    /// Encodes `Self` + args into a given `buf`
    ///
    /// # Errors
    ///
    /// May return an error if message doesn't fit into `buf`
    fn encode(args: Self::Args, buf: &mut [u8]) -> Result<&[u8], EncodeError>;
}
