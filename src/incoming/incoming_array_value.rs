use crate::{
    DBusError,
    incoming::{Cursor, IncomingCompleteType, IncomingValue},
};

#[derive(Debug)]
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

    pub fn iter(&self) -> IncomingArrayValueIter<'a> {
        IncomingArrayValueIter {
            item_type: self.item_type,
            cur: self.cur,
        }
    }
}

pub struct IncomingArrayValueIter<'a> {
    item_type: IncomingCompleteType<'a>,
    cur: Cursor<'a>,
}

impl<'a> IncomingArrayValueIter<'a> {
    pub fn try_next(&mut self) -> Result<Option<IncomingValue<'a>>, DBusError> {
        if self.cur.buf().is_empty() {
            return Ok(None);
        }

        let value = IncomingValue::cut(&mut self.cur, self.item_type)?;
        Ok(Some(value))
    }
}
