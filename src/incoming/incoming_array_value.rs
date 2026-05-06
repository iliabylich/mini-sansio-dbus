use crate::{
    DBusError,
    incoming::{Cursor, IncomingCompleteType, IncomingValue},
};

/// Array value that is a part of incoming message
#[derive(Debug)]
#[must_use]
pub struct IncomingArrayValue<'a> {
    item_type: IncomingCompleteType<'a>,
    cur: Cursor<'a>,
}

impl<'a> IncomingArrayValue<'a> {
    pub(crate) fn cut(cur: &mut Cursor<'a>, item_sig: &'a str) -> Result<Self, DBusError> {
        let (item_type, leftover) = IncomingCompleteType::cut(item_sig)?;
        if !leftover.is_empty() {
            return Err(DBusError::MalformedArray);
        }

        let array_bytesize = cur.cut_u32().map_err(|_| DBusError::MalformedArray)?;
        cur.align(item_type.alignment())
            .map_err(|_| DBusError::MalformedArray)?;
        let items_offset = cur.offset();
        let items_buf = cur
            .take(array_bytesize as usize)
            .map_err(|_| DBusError::MalformedArray)?;
        if !cur.buf().is_empty() {
            return Err(DBusError::MalformedArray);
        }

        Ok(Self {
            item_type,
            cur: Cursor::new(items_buf, items_offset),
        })
    }

    /// Returns an iterator over `self`
    pub const fn items_iter(&self) -> IncomingArrayValueIter<'a> {
        IncomingArrayValueIter {
            item_type: self.item_type,
            cur: self.cur,
        }
    }
}

/// An iterator over `IncomingArrayValue`
#[must_use]
pub struct IncomingArrayValueIter<'a> {
    item_type: IncomingCompleteType<'a>,
    cur: Cursor<'a>,
}

impl<'a> IncomingArrayValueIter<'a> {
    /// Returns the next item in `self`
    ///
    /// # Errors
    ///
    /// Returns an error if any of the lazily parsed values is invalid.
    pub fn try_next(&mut self) -> Result<Option<IncomingValue<'a>>, DBusError> {
        if self.cur.buf().is_empty() {
            return Ok(None);
        }

        let value = IncomingValue::cut(&mut self.cur, self.item_type)?;
        Ok(Some(value))
    }
}
