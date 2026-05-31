use crate::{DBusError, IncomingBody, IncomingMessage, MessageType};

/// A generic reply handler
#[must_use]
pub struct ReplyHandler<T>
where
    T: HandleReply,
{
    serial: u32,
    handler: T,
}

impl<T> ReplyHandler<T>
where
    T: HandleReply,
{
    /// Contstructor
    pub const fn new(serial: u32, handler: T) -> Self {
        Self { serial, handler }
    }

    /// Tries to handle a given message.
    ///
    /// # Errors
    ///
    /// Returns an error is message can't be parsed.
    pub fn handle(&self, message: IncomingMessage<'_>) -> Result<Option<T::Output>, DBusError> {
        if message.reply_serial != Some(self.serial) {
            return Ok(None);
        }
        if message.message_type != MessageType::MethodReturn {
            return Err(DBusError::ErrorReply);
        }
        let body = message.body.ok_or(DBusError::NoBody)?;
        let out = self.handler.handle_reply_body(body)?;
        Ok(Some(out))
    }
}

/// A reply handler trait, you should one your end
pub trait HandleReply {
    /// Output that it generates based on the given reply body.
    type Output;

    /// Called by `ReplyHandler` IF reply serial matches and IF reply is not an error.
    ///
    /// # Errors
    ///
    /// Any error that is returned is propagated by `ReplyHandler` that wraps `Self`
    fn handle_reply_body(&self, body: IncomingBody<'_>) -> Result<Self::Output, DBusError>;
}
