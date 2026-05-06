use crate::{
    DBusError,
    incoming::{Cursor, IncomingCompleteType, IncomingValue},
};

/// Represents body of the incoming message
#[derive(Debug, Clone, Copy)]
#[must_use]
pub struct IncomingBody<'a> {
    signature: &'a str,
    cur: Cursor<'a>,
}

impl<'a> IncomingBody<'a> {
    pub(crate) const fn new(signature: &'a str, cur: Cursor<'a>) -> Self {
        Self { signature, cur }
    }

    /// Returns an iterator over `IncomingBody` elements (conceptually request body is an array)
    ///
    /// # Errors
    ///
    /// Returns an error if any of the lazily parsed values is invalid.
    pub fn try_next(&mut self) -> Result<Option<IncomingValue<'a>>, DBusError> {
        if self.signature.is_empty() && self.cur.buf().is_empty() {
            return Ok(None);
        }

        let (type_, remainder) = IncomingCompleteType::cut(self.signature)?;
        self.signature = remainder;

        let value = IncomingValue::cut(&mut self.cur, type_)?;
        Ok(Some(value))
    }
}
