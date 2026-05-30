use crate::OutgoingQueue;

/// A common trait for all sendable messages
pub trait DBusSend {
    /// Returned error
    type Error;

    /// Sends a static message to a given queue, without processing reply
    fn send<'q, Q>(q: &mut Q) -> Result<u32, Self::Error>
    where
        Q: OutgoingQueue<'q>;
}
