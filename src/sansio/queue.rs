use crate::{OutgoingMessage, messages::org_freedesktop_dbus::Hello, outgoing::MessageEncoder};
use std::collections::VecDeque;

/// A queue of outgoing messages
#[must_use]
pub struct DBusQueue {
    serial: u32,
    q: VecDeque<Vec<u8>>,
}

impl DBusQueue {
    /// Constructs an empty queue
    pub const fn empty() -> Self {
        Self {
            serial: 1,
            q: VecDeque::new(),
        }
    }

    /// Pushes starting "hello" message to the queue
    pub fn push_hello(&mut self) {
        self.push_back(Hello::build());
    }

    /// Constructs a queue with a "hello" message inside
    pub fn new() -> Self {
        let mut this = Self {
            serial: 1,
            q: VecDeque::new(),
        };
        this.push_back(Hello::build());
        this
    }

    /// Pushes a new message
    pub fn push_back(&mut self, message: impl Into<OutgoingMessage>) -> u32 {
        let mut message: OutgoingMessage = message.into();
        *message.serial_mut() = self.serial;
        self.serial += 1;
        let buf = MessageEncoder::encode(&message);
        self.q.push_back(buf);
        message.serial()
    }

    pub(crate) fn pop_front(&mut self) -> Option<Vec<u8>> {
        self.q.pop_front()
    }
}

impl Default for DBusQueue {
    fn default() -> Self {
        Self::new()
    }
}
