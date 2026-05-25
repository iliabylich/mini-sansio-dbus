use crate::EncodeError;

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
}

impl Default for DBusSerial {
    fn default() -> Self {
        Self::new()
    }
}

/// Encoded message bytes backed by caller-provided storage.
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct EncodedMessage<B> {
    buf: B,
    len: usize,
}

impl<B: AsRef<[u8]>> EncodedMessage<B> {
    /// Constructs an encoded message from a buffer and encoded length.
    ///
    /// # Errors
    ///
    /// Returns an error if `len` is greater than the provided buffer length.
    pub fn new(buf: B, len: usize) -> Result<Self, EncodeError> {
        if len > buf.as_ref().len() {
            return Err(EncodeError::BufferTooSmall);
        }
        Ok(Self { buf, len })
    }

    /// Returns the encoded length.
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the encoded message is empty.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the wrapped buffer.
    pub fn into_inner(self) -> B {
        self.buf
    }
}

impl<B: AsMut<[u8]>> EncodedMessage<B> {
    /// Writes the message serial into the encoded header.
    ///
    /// # Errors
    ///
    /// Returns an error if the encoded message is too short to contain a D-Bus header.
    pub fn set_serial(&mut self, serial: u32) -> Result<(), EncodeError> {
        let Some(serial_slot) = self.buf.as_mut().get_mut(8..12) else {
            return Err(EncodeError::BufferTooSmall);
        };
        serial_slot.copy_from_slice(&serial.to_le_bytes());
        Ok(())
    }
}

impl<B: AsRef<[u8]>> AsRef<[u8]> for EncodedMessage<B> {
    fn as_ref(&self) -> &[u8] {
        self.buf.as_ref().get(..self.len).unwrap_or_default()
    }
}

/// A caller-owned queue of encoded outgoing messages.
pub trait OutgoingQueue {
    /// Message storage type accepted by this queue.
    type Message: AsRef<[u8]>;

    /// Error returned when a message cannot be pushed into the queue.
    type Error;

    /// Pushes already encoded message bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if the queue cannot accept another message.
    fn push(&mut self, message: Self::Message) -> Result<(), Self::Error>;

    /// Returns the first queued message.
    fn front(&self) -> Option<&[u8]>;

    /// Removes and returns the first queued message.
    fn pop_front(&mut self) -> Option<Self::Message>;
}
