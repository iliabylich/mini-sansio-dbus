use crate::{
    const_helpers::{get_range_mut, t_err},
    encoder::{EncodeError, EncodeResult},
};

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

    pub(crate) const fn align(&mut self, align: usize) -> EncodeResult<()> {
        while !self.pos.is_multiple_of(align) {
            t_err!(self.write_u8(0));
        }
        Ok(())
    }

    pub(crate) const fn write_u8(&mut self, value: u8) -> EncodeResult<()> {
        self.write_bytes(&[value])
    }

    pub(crate) const fn write_u16(&mut self, value: u16) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) const fn write_i16(&mut self, value: i16) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) const fn write_u32(&mut self, value: u32) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) const fn write_i32(&mut self, value: i32) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) const fn write_u64(&mut self, value: u64) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) const fn write_i64(&mut self, value: i64) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) const fn write_f64(&mut self, value: f64) -> EncodeResult<()> {
        self.write_bytes(&value.to_le_bytes())
    }

    pub(crate) const fn write_bytes(&mut self, bytes: &[u8]) -> EncodeResult<()> {
        let Some(end) = self.pos.checked_add(bytes.len()) else {
            return Err(EncodeError::ContainerTooLong);
        };
        if end > self.buf.len() {
            return Err(EncodeError::BufferTooSmall);
        }
        if end > u32::MAX as usize {
            return Err(EncodeError::ContainerTooLong);
        }
        let Some(dst) = get_range_mut(self.buf, self.pos, end) else {
            return Err(EncodeError::BufferTooSmall);
        };
        dst.copy_from_slice(bytes);
        self.pos = end;
        Ok(())
    }

    pub(crate) const fn set_u32(&mut self, at: usize, value: u32) -> EncodeResult<()> {
        let Some(end) = at.checked_add(4) else {
            return Err(EncodeError::ContainerTooLong);
        };
        let Some(slot) = get_range_mut(self.buf, at, end) else {
            return Err(EncodeError::BufferTooSmall);
        };
        slot.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}
