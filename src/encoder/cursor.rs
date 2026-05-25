use crate::encoder::{EncodeError, EncodeResult};

/// A cursor over the caller-provided output slice.
#[derive(Debug)]
pub struct SliceCursor<'buf> {
    buf: &'buf mut [u8],
    pos: usize,
}

impl<'buf> SliceCursor<'buf> {
    pub(crate) const fn new(buf: &'buf mut [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    pub(crate) const fn pos(&self) -> usize {
        self.pos
    }

    pub(crate) fn align(&mut self, align: usize) -> EncodeResult<()> {
        while !self.pos.is_multiple_of(align) {
            self.write_u8(0)?;
        }
        Ok(())
    }

    pub(crate) fn write_u8(&mut self, value: u8) -> EncodeResult<()> {
        self.write_bytes(&[value])
    }

    pub(crate) fn write_u16(&mut self, value: u16) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) fn write_i16(&mut self, value: i16) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) fn write_u32(&mut self, value: u32) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) fn write_i32(&mut self, value: i32) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) fn write_u64(&mut self, value: u64) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) fn write_i64(&mut self, value: i64) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) fn write_f64(&mut self, value: f64) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) fn write_bytes(&mut self, bytes: &[u8]) -> EncodeResult<()> {
        let end = self
            .pos
            .checked_add(bytes.len())
            .ok_or(EncodeError::ContainerTooLong)?;
        if end > self.buf.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        if end > u32::MAX as usize {
            return Err(EncodeError::ContainerTooLong);
        }
        let dst = self
            .buf
            .get_mut(self.pos..end)
            .ok_or(EncodeError::BufferTooSmall)?;
        dst.copy_from_slice(bytes);
        self.pos = end;
        Ok(())
    }

    pub(crate) fn set_u32(&mut self, at: usize, value: u32) -> EncodeResult<()> {
        let end = at.checked_add(4).ok_or(EncodeError::ContainerTooLong)?;
        let slot = self
            .buf
            .get_mut(at..end)
            .ok_or(EncodeError::BufferTooSmall)?;
        slot.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    pub(crate) fn set_u8(&mut self, at: usize, value: u8) -> EncodeResult<()> {
        let slot = self.buf.get_mut(at).ok_or(EncodeError::BufferTooSmall)?;
        *slot = value;
        Ok(())
    }
}
