use crate::{
    EncodeError,
    messaging::{
        DBusEncode,
        reply_handler::{HasReplyHandler, ReplyErrorHandler, ReplyHandler},
    },
};

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
    /// Pushes encoded message bytes after writing the next D-Bus serial into the header.
    fn push_raw_buf(&mut self, message: &[u8]) -> u32;

    /// High-level wrapper to encode -> push -> prepare for reply
    ///
    /// # Errors
    ///
    /// Returns an error if the message is too short to contain encoded message
    fn push_and_prepare_for_reply<const N: usize, M, E>(
        &mut self,
        message: M,
        data: M::Data,
        errhandler: E,
    ) -> Result<ReplyHandler<M, E>, EncodeError>
    where
        M: DBusEncode + HasReplyHandler,
        E: ReplyErrorHandler,
    {
        let mut buf = [0; N];
        let buf = M::encode(data, &mut buf)?;
        let serial = Self::push_raw_buf(self, buf);
        Ok(ReplyHandler::new(serial, message, errhandler))
    }

    /// High-level wrapper to encode -> push -> discard reply
    ///
    /// # Errors
    ///
    /// Returns an error if the message is too short to contain encoded message
    fn push_and_discard_reply<const N: usize, M>(
        &mut self,
        data: M::Data,
    ) -> Result<(), EncodeError>
    where
        M: DBusEncode,
    {
        let mut buf = [0; N];
        let buf = M::encode(data, &mut buf)?;
        let _serial = Self::push_raw_buf(self, buf);
        Ok(())
    }

    /// Returns the first queued message.
    fn peek(&self) -> Option<&[u8]>;

    /// Removes the first queued message.
    fn pop(&mut self);
}
