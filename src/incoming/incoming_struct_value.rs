use crate::{
    DBusError, IncomingValue,
    incoming::{CompleteTypeStructFieldsIter, Cursor, IncomingCompleteType},
};

#[derive(Debug)]
pub struct IncomingStructValue<'a> {
    struct_type: IncomingCompleteType<'a>,
    cur: Cursor<'a>,
}

impl<'a> IncomingStructValue<'a> {
    pub(crate) fn new(struct_type: IncomingCompleteType<'a>, cur: Cursor<'a>) -> Self {
        Self { struct_type, cur }
    }

    pub fn iter(&self) -> Result<IncomingStructValueIter<'a>, DBusError> {
        Ok(IncomingStructValueIter {
            field_type_iter: CompleteTypeStructFieldsIter::new(self.struct_type.buf())?,
            cur: self.cur,
        })
    }
}

pub struct IncomingStructValueIter<'a> {
    field_type_iter: CompleteTypeStructFieldsIter<'a>,
    cur: Cursor<'a>,
}

impl<'a> IncomingStructValueIter<'a> {
    pub fn try_next(&mut self) -> Result<Option<IncomingValue<'a>>, DBusError> {
        let Some(field_type) = self.field_type_iter.try_next()? else {
            return Ok(None);
        };

        let value = IncomingValue::cut(&mut self.cur, field_type)?;

        Ok(Some(value))
    }
}
