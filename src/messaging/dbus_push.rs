use crate::OutgoingQueue;

/// A common trait for all messages that can be pushed into a queue
pub trait DBusPush {
    /// Data to include
    type Data;
    /// Returned error
    type Error;

    /// Encodes and pushes `self` to a given queue, without processing a reply
    ///
    /// # Errors
    ///
    /// May return an implementation-specific error
    fn push<'q, Q>(data: Self::Data, buf: &'q mut [u8], q: &mut Q) -> Result<u32, Self::Error>
    where
        Q: OutgoingQueue<'q>;
}
