use core::marker::PhantomData;

use crate::encoder::{
    ArraySlot, DictEntrySlot, EncodeError, EncodeResult, Slot, Struct2Slot, VariantSlot,
    cursor::SliceCursor,
};

/// A D-Bus string value.
#[derive(Debug, Clone, Copy)]
pub struct Str;

/// A D-Bus object path value.
#[derive(Debug, Clone, Copy)]
pub struct ObjectPath;

/// A D-Bus signature value.
#[derive(Debug, Clone, Copy)]
pub struct Signature;

/// A D-Bus unix file descriptor index.
#[derive(Debug, Clone, Copy)]
pub struct UnixFd;

/// A D-Bus array value with items of type `T`.
#[derive(Debug, Clone, Copy)]
pub struct Array<T>(PhantomData<T>);

/// A D-Bus struct value with two fields.
#[derive(Debug, Clone, Copy)]
pub struct Struct2<A, B>(PhantomData<(A, B)>);

/// A D-Bus dict entry value.
#[derive(Debug, Clone, Copy)]
pub struct DictEntry<K, V>(PhantomData<(K, V)>);

/// A D-Bus variant value whose runtime payload has type `T`.
#[derive(Debug, Clone, Copy)]
pub struct Variant<T>(PhantomData<T>);

/// A type that can occupy a D-Bus value slot.
pub trait DbusType {
    /// The D-Bus alignment for this type.
    const ALIGNMENT: usize;

    /// The concrete slot type used to encode this value.
    type Slot<'slot, 'buf>
    where
        Self: 'slot,
        'buf: 'slot;

    /// Starts a slot of this type at the cursor's current position.
    fn start_slot<'slot, 'buf>(
        cur: &'slot mut SliceCursor<'buf>,
    ) -> EncodeResult<Self::Slot<'slot, 'buf>>
    where
        Self: 'slot,
        'buf: 'slot;

    /// Writes this type's D-Bus signature.
    fn write_signature(cur: &mut SliceCursor<'_>) -> EncodeResult<()>;
}

/// A D-Bus type that can be written directly as one scalar value.
pub trait WriteValue {
    /// The D-Bus alignment for this type.
    const ALIGNMENT: usize;

    /// The D-Bus signature for this type.
    const SIGNATURE: &'static str;

    /// The value accepted by [`Slot::write`].
    type Arg<'value>;

    /// Writes this value into the cursor.
    fn write_value(cur: &mut SliceCursor<'_>, value: Self::Arg<'_>) -> EncodeResult<()>;
}

impl<T: WriteValue> DbusType for T {
    const ALIGNMENT: usize = T::ALIGNMENT;

    type Slot<'slot, 'buf>
        = Slot<'slot, 'buf, T>
    where
        T: 'slot,
        'buf: 'slot;

    fn start_slot<'slot, 'buf>(
        cur: &'slot mut SliceCursor<'buf>,
    ) -> EncodeResult<Self::Slot<'slot, 'buf>>
    where
        T: 'slot,
        'buf: 'slot,
    {
        Ok(Slot::new(cur))
    }

    fn write_signature(cur: &mut SliceCursor<'_>) -> EncodeResult<()> {
        cur.write_bytes(T::SIGNATURE.as_bytes())
    }
}

impl WriteValue for u8 {
    const ALIGNMENT: usize = 1;
    const SIGNATURE: &'static str = "y";

    type Arg<'value> = u8;

    fn write_value(cur: &mut SliceCursor<'_>, value: u8) -> EncodeResult<()> {
        cur.write_u8(value)
    }
}

impl WriteValue for bool {
    const ALIGNMENT: usize = 4;
    const SIGNATURE: &'static str = "b";

    type Arg<'value> = bool;

    fn write_value(cur: &mut SliceCursor<'_>, value: bool) -> EncodeResult<()> {
        cur.align(4)?;
        cur.write_u32(u32::from(value))
    }
}

impl WriteValue for u16 {
    const ALIGNMENT: usize = 2;
    const SIGNATURE: &'static str = "q";

    type Arg<'value> = u16;

    fn write_value(cur: &mut SliceCursor<'_>, value: u16) -> EncodeResult<()> {
        cur.align(2)?;
        cur.write_u16(value)
    }
}

impl WriteValue for i16 {
    const ALIGNMENT: usize = 2;
    const SIGNATURE: &'static str = "n";

    type Arg<'value> = i16;

    fn write_value(cur: &mut SliceCursor<'_>, value: i16) -> EncodeResult<()> {
        cur.align(2)?;
        cur.write_i16(value)
    }
}

impl WriteValue for u32 {
    const ALIGNMENT: usize = 4;
    const SIGNATURE: &'static str = "u";

    type Arg<'value> = u32;

    fn write_value(cur: &mut SliceCursor<'_>, value: u32) -> EncodeResult<()> {
        cur.align(4)?;
        cur.write_u32(value)
    }
}

impl WriteValue for i32 {
    const ALIGNMENT: usize = 4;
    const SIGNATURE: &'static str = "i";

    type Arg<'value> = i32;

    fn write_value(cur: &mut SliceCursor<'_>, value: i32) -> EncodeResult<()> {
        cur.align(4)?;
        cur.write_i32(value)
    }
}

impl WriteValue for u64 {
    const ALIGNMENT: usize = 8;
    const SIGNATURE: &'static str = "t";

    type Arg<'value> = u64;

    fn write_value(cur: &mut SliceCursor<'_>, value: u64) -> EncodeResult<()> {
        cur.align(8)?;
        cur.write_u64(value)
    }
}

impl WriteValue for i64 {
    const ALIGNMENT: usize = 8;
    const SIGNATURE: &'static str = "x";

    type Arg<'value> = i64;

    fn write_value(cur: &mut SliceCursor<'_>, value: i64) -> EncodeResult<()> {
        cur.align(8)?;
        cur.write_i64(value)
    }
}

impl WriteValue for f64 {
    const ALIGNMENT: usize = 8;
    const SIGNATURE: &'static str = "d";

    type Arg<'value> = f64;

    fn write_value(cur: &mut SliceCursor<'_>, value: f64) -> EncodeResult<()> {
        cur.align(8)?;
        cur.write_f64(value)
    }
}

impl WriteValue for UnixFd {
    const ALIGNMENT: usize = 4;
    const SIGNATURE: &'static str = "h";

    type Arg<'value> = u32;

    fn write_value(cur: &mut SliceCursor<'_>, value: u32) -> EncodeResult<()> {
        cur.align(4)?;
        cur.write_u32(value)
    }
}

impl WriteValue for Str {
    const ALIGNMENT: usize = 4;
    const SIGNATURE: &'static str = "s";

    type Arg<'value> = &'value str;

    fn write_value(cur: &mut SliceCursor<'_>, value: &str) -> EncodeResult<()> {
        write_string_like(cur, value)
    }
}

impl WriteValue for ObjectPath {
    const ALIGNMENT: usize = 4;
    const SIGNATURE: &'static str = "o";

    type Arg<'value> = &'value str;

    fn write_value(cur: &mut SliceCursor<'_>, value: &str) -> EncodeResult<()> {
        write_string_like(cur, value)
    }
}

impl WriteValue for Signature {
    const ALIGNMENT: usize = 1;
    const SIGNATURE: &'static str = "g";

    type Arg<'value> = &'value str;

    fn write_value(cur: &mut SliceCursor<'_>, value: &str) -> EncodeResult<()> {
        write_signature(cur, value)
    }
}

impl<T: DbusType> DbusType for Array<T> {
    const ALIGNMENT: usize = 4;

    type Slot<'slot, 'buf>
        = ArraySlot<'slot, 'buf, T>
    where
        T: 'slot,
        'buf: 'slot;

    fn start_slot<'slot, 'buf>(
        cur: &'slot mut SliceCursor<'buf>,
    ) -> EncodeResult<Self::Slot<'slot, 'buf>>
    where
        T: 'slot,
        'buf: 'slot,
    {
        cur.align(4)?;
        let len_pos = cur.pos();
        cur.write_u32(0)?;
        cur.align(T::ALIGNMENT)?;
        let data_start = cur.pos();
        Ok(ArraySlot::new(cur, len_pos, data_start))
    }

    fn write_signature(cur: &mut SliceCursor<'_>) -> EncodeResult<()> {
        cur.write_u8(b'a')?;
        T::write_signature(cur)
    }
}

impl<A: DbusType, B: DbusType> DbusType for Struct2<A, B> {
    const ALIGNMENT: usize = 8;

    type Slot<'slot, 'buf>
        = Struct2Slot<'slot, 'buf, A, B>
    where
        A: 'slot,
        B: 'slot,
        'buf: 'slot;

    fn start_slot<'slot, 'buf>(
        cur: &'slot mut SliceCursor<'buf>,
    ) -> EncodeResult<Self::Slot<'slot, 'buf>>
    where
        A: 'slot,
        B: 'slot,
        'buf: 'slot,
    {
        cur.align(8)?;
        Ok(Struct2Slot::new(cur))
    }

    fn write_signature(cur: &mut SliceCursor<'_>) -> EncodeResult<()> {
        cur.write_u8(b'(')?;
        A::write_signature(cur)?;
        B::write_signature(cur)?;
        cur.write_u8(b')')
    }
}

impl<K: DbusType, V: DbusType> DbusType for DictEntry<K, V> {
    const ALIGNMENT: usize = 8;

    type Slot<'slot, 'buf>
        = DictEntrySlot<'slot, 'buf, K, V>
    where
        K: 'slot,
        V: 'slot,
        'buf: 'slot;

    fn start_slot<'slot, 'buf>(
        cur: &'slot mut SliceCursor<'buf>,
    ) -> EncodeResult<Self::Slot<'slot, 'buf>>
    where
        K: 'slot,
        V: 'slot,
        'buf: 'slot,
    {
        cur.align(8)?;
        Ok(DictEntrySlot::new(cur))
    }

    fn write_signature(cur: &mut SliceCursor<'_>) -> EncodeResult<()> {
        cur.write_u8(b'{')?;
        K::write_signature(cur)?;
        V::write_signature(cur)?;
        cur.write_u8(b'}')
    }
}

impl<T: DbusType> DbusType for Variant<T> {
    const ALIGNMENT: usize = 1;

    type Slot<'slot, 'buf>
        = VariantSlot<'slot, 'buf, T>
    where
        T: 'slot,
        'buf: 'slot;

    fn start_slot<'slot, 'buf>(
        cur: &'slot mut SliceCursor<'buf>,
    ) -> EncodeResult<Self::Slot<'slot, 'buf>>
    where
        T: 'slot,
        'buf: 'slot,
    {
        let len_pos = cur.pos();
        cur.write_u8(0)?;
        let signature_start = cur.pos();
        T::write_signature(cur)?;
        let signature_len = cur.pos() - signature_start;
        let signature_len = u8::try_from(signature_len).map_err(|_| EncodeError::ValueTooLong)?;
        cur.set_u8(len_pos, signature_len);
        cur.write_u8(0)?;
        Ok(VariantSlot::new(cur))
    }

    fn write_signature(cur: &mut SliceCursor<'_>) -> EncodeResult<()> {
        cur.write_u8(b'v')
    }
}

pub(crate) fn write_string_like(cur: &mut SliceCursor<'_>, value: &str) -> EncodeResult<()> {
    let len = u32::try_from(value.len()).map_err(|_| EncodeError::ValueTooLong)?;
    cur.align(4)?;
    cur.write_u32(len)?;
    cur.write_bytes(value.as_bytes())?;
    cur.write_u8(0)
}

pub(crate) fn write_signature(cur: &mut SliceCursor<'_>, value: &str) -> EncodeResult<()> {
    let len = u8::try_from(value.len()).map_err(|_| EncodeError::ValueTooLong)?;
    cur.write_u8(len)?;
    cur.write_bytes(value.as_bytes())?;
    cur.write_u8(0)
}
