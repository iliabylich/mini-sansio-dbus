use crate::{
    DBusError, IncomingBody, IncomingMessage, OutgoingMessage, sansio::DBusQueue,
    types::MessageType,
};
use core::marker::PhantomData;

#[derive(Debug, Clone, Copy)]
enum OneshotState {
    None,
    WaitingForReply(u32),
    ReplyReceived,
}

pub struct MethodCall<In, Out, Data>
where
    In: 'static,
    Out: 'static,
    Data: Clone + Default + 'static,
{
    send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    try_process:
        &'static dyn Fn(IncomingBody<'_>, Data) -> Result<Out, Box<dyn core::error::Error>>,
    state: OneshotState,
    data: Option<Data>,
}

impl<In, Out, Data> MethodCall<In, Out, Data>
where
    Data: Clone + Default,
{
    pub fn with_data(self, data: Data) -> Self {
        Self {
            send: self.send,
            try_process: self.try_process,
            state: self.state,
            data: Some(data),
        }
    }

    pub fn send(&mut self, input: In, queue: &mut DBusQueue) {
        if !matches!(self.state, OneshotState::None) {
            return;
        };

        let message: OutgoingMessage = (self.send)(input, self.data.clone().unwrap_or_default());
        let reply_serial = queue.push_back(message);
        self.state = OneshotState::WaitingForReply(reply_serial);
    }

    pub fn try_recv(&mut self, message: IncomingMessage<'_>) -> Result<Option<Out>, DBusError> {
        let OneshotState::WaitingForReply(reply_serial) = self.state else {
            return Ok(None);
        };
        if message.reply_serial != Some(reply_serial) {
            return Ok(None);
        }
        self.state = OneshotState::ReplyReceived;

        match message.message_type {
            MessageType::Error => Err(DBusError::DBusError(format!("{:?}", message.error_name))),
            MessageType::MethodReturn => {
                if let Some(body) = message.body {
                    Ok((self.try_process)(body, self.data.clone().unwrap_or_default()).ok())
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    pub fn reset(&mut self) {
        self.state = OneshotState::None
    }

    pub const fn builder() -> OneshotMethodCallBuilder<In, Out, Data, NeedsSend> {
        OneshotMethodCallBuilder {
            send: &|_: In, _: Data| todo!(),
            _state: PhantomData,
            _out: PhantomData,
        }
    }
}

impl<In, Out, Data> core::fmt::Debug for MethodCall<In, Out, Data>
where
    Data: Clone + Default,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OneshotMethodCall")
            .field("state", &self.state)
            .finish()
    }
}

pub struct NeedsSend;
pub struct NeedsTryProcess;

pub struct OneshotMethodCallBuilder<In, Out, Data, S>
where
    In: 'static,
    Out: 'static,
    Data: Default + Clone + 'static,
{
    send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    _state: PhantomData<S>,
    _out: PhantomData<Out>,
}

impl<In, Out, Data> OneshotMethodCallBuilder<In, Out, Data, NeedsSend>
where
    Data: Default + Clone,
{
    pub const fn send(
        self,
        send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    ) -> OneshotMethodCallBuilder<In, Out, Data, NeedsTryProcess> {
        OneshotMethodCallBuilder {
            send,
            _state: PhantomData,
            _out: PhantomData,
        }
    }
}
impl<In, Out, Data> OneshotMethodCallBuilder<In, Out, Data, NeedsTryProcess>
where
    Data: Default + Clone,
{
    pub const fn try_process(
        self,
        try_process: &'static dyn Fn(
            IncomingBody<'_>,
            Data,
        ) -> Result<Out, Box<dyn core::error::Error>>,
    ) -> MethodCall<In, Out, Data> {
        MethodCall {
            send: self.send,
            try_process,
            state: OneshotState::None,
            data: None,
        }
    }
}
