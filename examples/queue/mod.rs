use mini_sansio_dbus::{
    DBusSerial, EncodeError, MessageType, OutgoingQueue,
    messaging::{
        DBusEncode,
        reply_handler::{HandleReply, ReplyErrorHandler, ReplyHandler},
    },
};
use std::collections::VecDeque;

#[derive(Debug)]
pub(crate) struct ExampleQueue {
    serial: DBusSerial,
    messages: VecDeque<Vec<u8>>,
}

impl ExampleQueue {
    pub(crate) fn new() -> Self {
        Self {
            serial: DBusSerial::new(),
            messages: VecDeque::new(),
        }
    }
}

impl ExampleQueue {
    fn next_serial(&mut self) -> u32 {
        let serial = self.serial.current();
        self.serial.advance();
        serial
    }

    #[allow(dead_code)]
    pub(crate) fn push_and_prepare_for_reply<M>(
        &mut self,
        data: M::Data,
    ) -> Result<ReplyHandler, EncodeError>
    where
        M: DBusEncode + HandleReply,
    {
        OutgoingQueue::push_and_prepare_for_reply::<1_024, M>(self, data)
    }

    #[allow(dead_code)]
    pub(crate) fn push_and_discard_reply<M>(&mut self, data: M::Data) -> Result<(), EncodeError>
    where
        M: DBusEncode,
    {
        OutgoingQueue::push_and_discard_reply::<1_024, M>(self, data)
    }
}

impl OutgoingQueue for ExampleQueue {
    fn push_raw_buf(&mut self, message: &[u8]) -> u32 {
        let serial = self.next_serial();
        let mut message = message.to_vec();
        DBusSerial::write_to_message(&mut message, serial).unwrap();
        self.messages.push_back(message);
        serial
    }

    fn peek(&self) -> Option<&[u8]> {
        self.messages.front().map(|m| m.as_slice())
    }

    fn pop(&mut self) {
        self.messages.pop_front();
    }
}

#[allow(dead_code)]
pub(crate) struct DefaultErrorHandler;

impl ReplyErrorHandler for DefaultErrorHandler {
    fn on_error(message_type: MessageType, error_name: &str) {
        log::error!("call failed: {message_type:?} - {error_name:?}")
    }
}
