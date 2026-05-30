use crate::EncodeError;

/// A common trait for all messages that can be encoded into a buffer
pub trait DBusEncode {
    /// Data to include
    type Data;

    /// Encodes `Self` + data into a given `buf`
    ///
    /// # Errors
    ///
    /// May return an error if message doesn't fit into `buf`
    fn encode(data: Self::Data, buf: &mut [u8]) -> Result<usize, EncodeError>;
}
