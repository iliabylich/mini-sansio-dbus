use crate::{
    DBusError, IncomingValue,
    incoming::{Cursor, IncomingCompleteType},
};

#[derive(Debug)]
pub struct IncomingVariantValue<'a> {
    cur: Cursor<'a>,
}

impl<'a> IncomingVariantValue<'a> {
    pub(crate) fn new(cur: Cursor<'a>) -> Self {
        Self { cur }
    }

    pub fn materialize(&self) -> Result<IncomingValue<'a>, DBusError> {
        let mut cur = self.cur;
        let signature = cur.cut_signature()?;

        let (type_, leftover) = IncomingCompleteType::cut(signature)?;
        if !leftover.is_empty() {
            return Err(DBusError::MalformedVariant);
        }

        let value = IncomingValue::cut(&mut cur, type_)?;
        if !cur.buf().is_empty() {
            return Err(DBusError::MalformedVariant);
        }

        Ok(value)
    }
}
