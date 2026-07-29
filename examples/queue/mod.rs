use mini_sansio_dbus::{
    DBusSerial, EncodeError, OutgoingQueue,
    messaging::{
        DBusEncode,
        reply_handler::{HandleReply, ReplyHandler},
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
    pub(crate) fn push_with_reply<M>(
        &mut self,
        message: M,
        args: M::Args<'_>,
    ) -> Result<ReplyHandler<M>, EncodeError>
    where
        M: DBusEncode + HandleReply,
    {
        let mut buf = [0; 8 * 1_024];
        let buf = M::encode(args, &mut buf)?;
        let handler = self.push_raw_and_prepare_for_reply(message, buf);
        Ok(handler)
    }

    #[allow(dead_code)]
    pub(crate) fn push_without_reply<M>(&mut self, args: M::Args<'_>) -> Result<(), EncodeError>
    where
        M: DBusEncode,
    {
        let mut buf = [0; 8 * 1_024];
        let buf = M::encode(args, &mut buf)?;
        let _ = self.push_raw(buf);
        Ok(())
    }
}

impl OutgoingQueue for ExampleQueue {
    fn push_raw(&mut self, message: &[u8]) -> u32 {
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
