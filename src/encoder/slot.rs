use core::marker::PhantomData;

use crate::encoder::{
    EncodeError, EncodeResult,
    cursor::SliceCursor,
    types::{DbusType, WriteValue},
};

/// A slot for writing one D-Bus value of type `T`.
#[derive(Debug)]
pub struct Slot<'slot, 'buf, T> {
    cur: &'slot mut SliceCursor<'buf>,
    written: bool,
    _ty: PhantomData<T>,
}

impl<'slot, 'buf, T> Slot<'slot, 'buf, T> {
    pub(crate) const fn new(cur: &'slot mut SliceCursor<'buf>) -> Self {
        Self {
            cur,
            written: false,
            _ty: PhantomData,
        }
    }
}

impl<T: WriteValue> Slot<'_, '_, T> {
    /// Writes the value into this slot.
    pub fn write(&mut self, value: T::Arg<'_>) -> EncodeResult<()> {
        if self.written {
            return Err(EncodeError::ValueAlreadyWritten);
        }
        T::write_value(self.cur, value)?;
        self.written = true;
        Ok(())
    }
}

/// A slot for writing a D-Bus array with items of type `T`.
#[derive(Debug)]
pub struct ArraySlot<'slot, 'buf, T> {
    cur: &'slot mut SliceCursor<'buf>,
    len_pos: usize,
    data_start: usize,
    finalized: bool,
    _item: PhantomData<T>,
}

impl<'slot, 'buf, T> ArraySlot<'slot, 'buf, T> {
    pub(crate) const fn new(
        cur: &'slot mut SliceCursor<'buf>,
        len_pos: usize,
        data_start: usize,
    ) -> Self {
        Self {
            cur,
            len_pos,
            data_start,
            finalized: false,
            _item: PhantomData,
        }
    }

    /// Starts the next item slot in this array.
    pub fn next_slot(&mut self) -> EncodeResult<T::Slot<'_, 'buf>>
    where
        T: DbusType,
    {
        T::start_slot(self.cur)
    }

    /// Explicitly finalizes the array length field.
    pub fn finish(&mut self) -> EncodeResult<()> {
        self.finalize()
    }

    fn finalize(&mut self) -> EncodeResult<()> {
        if self.finalized {
            return Ok(());
        }
        let byte_len = self
            .cur
            .pos()
            .checked_sub(self.data_start)
            .ok_or(EncodeError::ContainerTooLong)?;
        let byte_len = u32::try_from(byte_len).map_err(|_| EncodeError::ContainerTooLong)?;
        self.cur.set_u32(self.len_pos, byte_len)?;
        self.finalized = true;
        Ok(())
    }
}

impl<T> Drop for ArraySlot<'_, '_, T> {
    fn drop(&mut self) {
        let _ = self.finalize();
    }
}

/// A slot for writing a two-field D-Bus struct.
#[derive(Debug)]
pub struct Struct2Slot<'slot, 'buf, A, B> {
    cur: &'slot mut SliceCursor<'buf>,
    next_field: u8,
    _fields: PhantomData<(A, B)>,
}

impl<'slot, 'buf, A, B> Struct2Slot<'slot, 'buf, A, B> {
    pub(crate) const fn new(cur: &'slot mut SliceCursor<'buf>) -> Self {
        Self {
            cur,
            next_field: 0,
            _fields: PhantomData,
        }
    }

    /// Starts the first field slot.
    pub fn first_slot(&mut self) -> EncodeResult<A::Slot<'_, 'buf>>
    where
        A: DbusType,
    {
        self.take_field(0)?;
        A::start_slot(self.cur)
    }

    /// Starts the second field slot.
    pub fn second_slot(&mut self) -> EncodeResult<B::Slot<'_, 'buf>>
    where
        B: DbusType,
    {
        self.take_field(1)?;
        B::start_slot(self.cur)
    }

    fn take_field(&mut self, expected: u8) -> EncodeResult<()> {
        if self.next_field != expected {
            return Err(EncodeError::TypeMismatch);
        }
        self.next_field = self
            .next_field
            .checked_add(1)
            .ok_or(EncodeError::TypeMismatch)?;
        Ok(())
    }
}

/// A slot for writing a D-Bus dict entry.
#[derive(Debug)]
pub struct DictEntrySlot<'slot, 'buf, K, V> {
    cur: &'slot mut SliceCursor<'buf>,
    next_field: u8,
    _fields: PhantomData<(K, V)>,
}

impl<'slot, 'buf, K, V> DictEntrySlot<'slot, 'buf, K, V> {
    pub(crate) const fn new(cur: &'slot mut SliceCursor<'buf>) -> Self {
        Self {
            cur,
            next_field: 0,
            _fields: PhantomData,
        }
    }

    /// Starts the key slot.
    pub fn key_slot(&mut self) -> EncodeResult<K::Slot<'_, 'buf>>
    where
        K: DbusType,
    {
        self.take_field(0)?;
        K::start_slot(self.cur)
    }

    /// Starts the value slot.
    pub fn value_slot(&mut self) -> EncodeResult<V::Slot<'_, 'buf>>
    where
        V: DbusType,
    {
        self.take_field(1)?;
        V::start_slot(self.cur)
    }

    fn take_field(&mut self, expected: u8) -> EncodeResult<()> {
        if self.next_field != expected {
            return Err(EncodeError::TypeMismatch);
        }
        self.next_field = self
            .next_field
            .checked_add(1)
            .ok_or(EncodeError::TypeMismatch)?;
        Ok(())
    }
}

/// A slot for writing a D-Bus variant payload of type `T`.
#[derive(Debug)]
pub struct VariantSlot<'slot, 'buf, T> {
    cur: &'slot mut SliceCursor<'buf>,
    written: bool,
    _ty: PhantomData<T>,
}

impl<'slot, 'buf, T> VariantSlot<'slot, 'buf, T> {
    pub(crate) const fn new(cur: &'slot mut SliceCursor<'buf>) -> Self {
        Self {
            cur,
            written: false,
            _ty: PhantomData,
        }
    }

    /// Starts the variant payload slot.
    pub fn payload_slot(&mut self) -> EncodeResult<T::Slot<'_, 'buf>>
    where
        T: DbusType,
    {
        if self.written {
            return Err(EncodeError::ValueAlreadyWritten);
        }
        self.written = true;
        T::start_slot(self.cur)
    }
}
