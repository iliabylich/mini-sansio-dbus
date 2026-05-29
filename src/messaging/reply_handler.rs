use crate::{DBusError, IncomingBody, IncomingMessage, MessageType};

/// A generic reply handler, parameterezed to a concrete `HasReplyHandler` and `ReplyErrorHandler`
pub struct ReplyHandler<T: HasReplyHandler, E: ReplyErrorHandler> {
    serial: u32,
    handler: T,
    errhandler: E,
}

impl<T: HasReplyHandler, E: ReplyErrorHandler> ReplyHandler<T, E> {
    /// Contstructor
    pub const fn new(serial: u32, handler: T, errhandler: E) -> Self {
        Self {
            serial,
            handler,
            errhandler,
        }
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
            self.errhandler.on_error(
                message.message_type,
                message.error_name.unwrap_or("<unknown error>"),
            );
            return Err(DBusError::ErrorReply);
        }
        let body = message.body.ok_or(DBusError::NoBody)?;
        let out = self.handler.handle(body)?;
        Ok(Some(out))
    }
}

/// An error handler trait, you should implement one on your end.
pub trait ReplyErrorHandler {
    /// A method that is called when reply has an error.
    /// The error is returned anyway, but here you can do something app-specific.
    fn on_error(&self, message_type: MessageType, error_name: &str);
}

/// A reply handler trait, you should one your end
pub trait HasReplyHandler {
    /// Output that it generates based on the given reply body.
    type Output;

    /// Called by `ReplyHandler` IF reply serial matches and IF reply is not an error.
    ///
    /// # Errors
    ///
    /// Any error that is returned is propagated by `ReplyHandler` that wraps `Self`
    fn handle(&self, body: IncomingBody<'_>) -> Result<Self::Output, DBusError>;
}
