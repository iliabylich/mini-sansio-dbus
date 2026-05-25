use crate::{DBusError, EncodeError};

/// A message that can encode itself into a caller-provided byte slice.
pub trait EncodeMessage {
    /// Returns the buffer size this message needs for encoding.
    fn encoded_capacity(&self) -> usize;

    /// Encodes this message without assigning a serial.
    ///
    /// # Errors
    ///
    /// Returns an error if the provided buffer cannot fit the encoded message.
    fn encode_message(&self, buf: &mut [u8]) -> Result<usize, EncodeError>;
}

/// Allocates outgoing D-Bus message serials.
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct DBusSerial {
    next: u32,
}

impl DBusSerial {
    /// Constructs a serial allocator starting at serial 1.
    pub const fn new() -> Self {
        Self { next: 1 }
    }

    /// Returns the serial that will be assigned to the next outgoing message.
    #[must_use]
    pub const fn current(&self) -> u32 {
        self.next
    }

    /// Marks the current serial as used.
    pub fn advance(&mut self) {
        self.next = self.next.checked_add(1).unwrap_or(1);
    }

    /// Writes `serial` into an encoded D-Bus message header.
    ///
    /// # Errors
    ///
    /// Returns an error if the message is too short to contain a D-Bus header.
    pub fn write_to_message(message: &mut [u8], serial: u32) -> Result<(), EncodeError> {
        let Some(serial_slot) = message.get_mut(8..12) else {
            return Err(EncodeError::BufferTooSmall);
        };
        serial_slot.copy_from_slice(&serial.to_le_bytes());
        Ok(())
    }
}

impl Default for DBusSerial {
    fn default() -> Self {
        Self::new()
    }
}

/// A caller-owned queue of encoded outgoing messages.
pub trait OutgoingQueue {
    /// Allocates and returns the next outgoing message serial.
    fn next_serial(&mut self) -> u32;

    /// Pushes encoded message bytes after writing the next D-Bus serial into the header.
    ///
    /// # Errors
    ///
    /// Returns an error if the message is too short to contain a D-Bus header, or if the queue
    /// cannot accept another message.
    fn push(&mut self, message: &mut [u8]) -> Result<u32, DBusError>;

    /// Returns the first queued message.
    fn front(&self) -> Option<&[u8]>;

    /// Removes the first queued message.
    fn pop_front(&mut self);
}
