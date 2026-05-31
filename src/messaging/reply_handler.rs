use crate::{DBusError, IncomingBody, IncomingMessage, MessageType};

/// A generic reply handler, parameterezed to a concrete `HasReplyHandler` and `ReplyErrorHandler`
#[must_use]
pub struct ReplyHandler {
    serial: u32,
}

impl ReplyHandler {
    /// Contstructor
    pub const fn new(serial: u32) -> Self {
        Self { serial }
    }

    /// Tries to handle a given message.
    ///
    /// # Errors
    ///
    /// Returns an error is message can't be parsed.
    pub fn handle<T, E>(&self, message: IncomingMessage<'_>) -> Result<Option<T::Output>, DBusError>
    where
        T: HandleReply,
        E: ReplyErrorHandler,
    {
        if message.reply_serial != Some(self.serial) {
            return Ok(None);
        }
        if message.message_type != MessageType::MethodReturn {
            E::on_error(
                message.message_type,
                message.error_name.unwrap_or("<unknown error>"),
            );
            return Err(DBusError::ErrorReply);
        }
        let body = message.body.ok_or(DBusError::NoBody)?;
        let out = T::handle_reply(body)?;
        Ok(Some(out))
    }
}

/// An error handler trait, you should implement one on your end.
pub trait ReplyErrorHandler {
    /// A method that is called when reply has an error.
    /// The error is returned anyway, but here you can do something app-specific.
    fn on_error(message_type: MessageType, error_name: &str);
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
    fn handle_reply(body: IncomingBody<'_>) -> Result<Self::Output, DBusError>;
}
