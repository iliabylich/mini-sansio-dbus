use crate::EncodeError;

/// A common trait for all messages that can be encoded into a buffer
pub trait DBusEncode {
    /// Encoding arguments
    type Args<'a>;

    /// Encodes `Self` + args into a given `buf`
    ///
    /// # Errors
    ///
    /// May return an error if message doesn't fit into `buf`
    fn encode<'a>(args: Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError>;
}
