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

/// An "incomplete" method call that must be filled with data
#[must_use]
pub struct IncompleteMethodCall<In, Out, Data>
where
    In: 'static,
    Out: 'static,
    Data: Clone + 'static,
{
    send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    try_process:
        &'static dyn Fn(IncomingBody<'_>, Data) -> Result<Out, Box<dyn core::error::Error>>,
}

impl<In, Out, Data> IncompleteMethodCall<In, Out, Data>
where
    Data: Clone,
{
    /// Fills `self` with data, makes it send-able
    pub const fn with_data(self, data: Data) -> MethodCall<In, Out, Data> {
        MethodCall {
            send: self.send,
            try_process: self.try_process,
            state: OneshotState::None,
            data,
        }
    }
}

/// A "complete" method call, can be sent
#[must_use]
pub struct MethodCall<In, Out, Data>
where
    In: 'static,
    Out: 'static,
    Data: Clone + 'static,
{
    send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    try_process:
        &'static dyn Fn(IncomingBody<'_>, Data) -> Result<Out, Box<dyn core::error::Error>>,
    state: OneshotState,
    data: Data,
}

impl<In, Out, Data> MethodCall<In, Out, Data>
where
    Data: Clone,
{
    /// This is a builder pattern:
    ///
    /// ```ignore
    /// let reply = MethodCall::new(|input, data| { /* build request */ })
    ///     .try_process(|body, data| { /* parse and validate response */ 42 } )
    ///     .with_data("any object here");
    /// assert_eq!(reply, 42);
    /// ```
    #[expect(clippy::new_ret_no_self)]
    pub const fn new(
        send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    ) -> OneshotMethodCallBuilder<In, Out, Data, NeedsTryProcess> {
        OneshotMethodCallBuilder {
            send,
            _state: PhantomData,
            _out: PhantomData,
        }
    }

    /// Writes a message to a given `queue`
    pub fn send(&mut self, input: In, queue: &mut DBusQueue) {
        if !matches!(self.state, OneshotState::None) {
            return;
        }

        let message: OutgoingMessage = (self.send)(input, self.data.clone());
        let reply_serial = queue.push_back(message);
        self.state = OneshotState::WaitingForReply(reply_serial);
    }

    /// Tries to process incoming message
    ///
    /// # Errors
    ///
    /// Fails is `serial` of the message matches but it's not a `MethodReturn`
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
                    Ok((self.try_process)(body, self.data.clone()).ok())
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    /// Resets `self` which lets it to be sent again
    pub const fn reset(&mut self) {
        self.state = OneshotState::None;
    }
}

impl<In, Out, Data> core::fmt::Debug for MethodCall<In, Out, Data>
where
    Data: Clone,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OneshotMethodCall")
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

pub struct NeedsTryProcess;

pub struct OneshotMethodCallBuilder<In, Out, Data, S>
where
    In: 'static,
    Out: 'static,
    Data: Clone + 'static,
{
    send: &'static dyn Fn(In, Data) -> OutgoingMessage,
    _state: PhantomData<S>,
    _out: PhantomData<Out>,
}

impl<In, Out, Data> OneshotMethodCallBuilder<In, Out, Data, NeedsTryProcess>
where
    Data: Clone,
{
    pub const fn try_process(
        self,
        try_process: &'static dyn Fn(
            IncomingBody<'_>,
            Data,
        ) -> Result<Out, Box<dyn core::error::Error>>,
    ) -> IncompleteMethodCall<In, Out, Data> {
        IncompleteMethodCall {
            send: self.send,
            try_process,
        }
    }
}
