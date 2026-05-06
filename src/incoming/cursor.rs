use crate::DBusError;

#[derive(Clone, Copy, Debug)]
#[must_use]
pub(crate) struct Cursor<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) const fn new(buf: &'a [u8], offset: usize) -> Self {
        Self { buf, offset }
    }

    pub(crate) const fn buf(&self) -> &'a [u8] {
        self.buf
    }

    pub(crate) const fn offset(&self) -> usize {
        self.offset
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub(crate) fn align(&mut self, alignment: usize) -> Result<(), DBusError> {
        let pad = (alignment - (self.offset % alignment)) % alignment;
        self.buf = self.buf.get(pad..).ok_or(DBusError::ParseError)?;
        self.offset += pad;
        Ok(())
    }

    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8], DBusError> {
        let (head, tail) = self.buf.split_at_checked(n).ok_or(DBusError::ParseError)?;
        self.buf = tail;
        self.offset += n;
        Ok(head)
    }

    pub(crate) fn cut_bytes<const N: usize>(
        &mut self,
        alignment: usize,
    ) -> Result<[u8; N], DBusError> {
        self.align(alignment)?;
        self.take(N)?.try_into().map_err(|_| DBusError::ParseError)
    }

    pub(crate) fn cut_u8(&mut self) -> Result<u8, DBusError> {
        Ok(self.cut_bytes::<1>(1)?[0])
    }

    pub(crate) fn cut_bool(&mut self) -> Result<bool, DBusError> {
        Ok(self.cut_u32()? != 0)
    }

    pub(crate) fn cut_i16(&mut self) -> Result<i16, DBusError> {
        Ok(i16::from_le_bytes(self.cut_bytes::<2>(2)?))
    }

    pub(crate) fn cut_u16(&mut self) -> Result<u16, DBusError> {
        Ok(u16::from_le_bytes(self.cut_bytes::<2>(2)?))
    }

    pub(crate) fn cut_i32(&mut self) -> Result<i32, DBusError> {
        Ok(i32::from_le_bytes(self.cut_bytes::<4>(4)?))
    }

    pub(crate) fn cut_u32(&mut self) -> Result<u32, DBusError> {
        Ok(u32::from_le_bytes(self.cut_bytes::<4>(4)?))
    }

    pub(crate) fn cut_i64(&mut self) -> Result<i64, DBusError> {
        Ok(i64::from_le_bytes(self.cut_bytes::<8>(8)?))
    }

    pub(crate) fn cut_u64(&mut self) -> Result<u64, DBusError> {
        Ok(u64::from_le_bytes(self.cut_bytes::<8>(8)?))
    }

    pub(crate) fn cut_f64(&mut self) -> Result<f64, DBusError> {
        Ok(f64::from_le_bytes(self.cut_bytes::<8>(8)?))
    }

    pub(crate) fn cut_signature(&mut self) -> Result<&'a str, DBusError> {
        let len = self.cut_u8()? as usize;
        let sig = self.take(len)?;
        let sig = core::str::from_utf8(sig).map_err(|_| DBusError::ParseError)?;
        self.take(1)?;
        Ok(sig)
    }

    pub(crate) fn cut_string(&mut self) -> Result<&'a str, DBusError> {
        let len = self.cut_u32()? as usize;
        let s = self.take(len)?;
        let s = core::str::from_utf8(s).map_err(|_| DBusError::ParseError)?;
        self.take(1)?;
        Ok(s)
    }
}
