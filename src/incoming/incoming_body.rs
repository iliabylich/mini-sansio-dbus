use crate::{
    DBusError,
    incoming::{Cursor, IncomingCompleteType, IncomingValue},
};

#[derive(Debug, Clone, Copy)]
pub struct IncomingBody<'a> {
    signature: &'a str,
    cur: Cursor<'a>,
}

impl<'a> IncomingBody<'a> {
    pub(crate) fn new(signature: &'a str, cur: Cursor<'a>) -> Self {
        Self { signature, cur }
    }

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
