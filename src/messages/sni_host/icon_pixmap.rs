use crate::{
    Conf, DBusError, IncomingArrayValueIter, IncomingValue, messaging::property::Property, value_is,
};
use core::marker::PhantomData;

/// A trait that a caller must implement to insantiate and accumulate `IconPixmap` bytes.
pub trait IconPixmapBytes {
    /// Constructor
    fn new() -> Self;
    /// Pushes a single byte
    fn push(&mut self, v: u8);
}

/// `IconName` property of the (K)SNI item
pub struct IconPixmap<D, B>
where
    D: AsRef<str> + Clone,
    B: IconPixmapBytes,
{
    destination: D,
    _marker: PhantomData<B>,
}

impl<D, B> Clone for IconPixmap<D, B>
where
    D: AsRef<str> + Clone,
    B: IconPixmapBytes,
{
    fn clone(&self) -> Self {
        Self {
            destination: self.destination.clone(),
            _marker: PhantomData,
        }
    }
}

impl<D, B> Property for IconPixmap<D, B>
where
    D: AsRef<str> + Clone,
    B: IconPixmapBytes,
{
    type Output<'a> = (i32, i32, B);

    const DESTINATION: Conf<str, Self> = Conf::dynamic(|this| this.destination.as_ref());
    const PATH: Conf<str, Self> = Conf::constant("/StatusNotifierItem");
    const INTERFACE: Conf<str, Self> = Conf::constant("org.kde.StatusNotifierItem");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("IconPixmap");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        const ERR: DBusError = DBusError::Other("failed to parse IconPixmap proeprty");

        value_is!(value, IncomingValue::Array(value));

        let mut iter = value.items_iter();
        let w_h_bytes = iter.try_next()?.ok_or(ERR)?;
        value_is!(w_h_bytes, IncomingValue::Struct(w_h_bytes));

        let mut iter = w_h_bytes.fields_iter()?;

        let width = iter.try_next()?.ok_or(ERR)?;
        value_is!(width, IncomingValue::Int32(width));

        let height = iter.try_next()?.ok_or(ERR)?;
        value_is!(height, IncomingValue::Int32(height));

        let bytes = iter.try_next()?.ok_or(ERR)?;
        value_is!(bytes, IncomingValue::Array(bytes));

        let bytes = {
            let mut out = B::new();
            let mut iter = bytes.items_iter();

            while let Some((a, r, g, b)) = read4(&mut iter)? {
                // argb -> rgba
                out.push(r);
                out.push(g);
                out.push(b);
                out.push(a);
            }

            out
        };

        Ok((width, height, bytes))
    }
}

impl<D, B> IconPixmap<D, B>
where
    D: AsRef<str> + Clone,
    B: IconPixmapBytes,
{
    /// Constructor
    pub const fn new(destination: D) -> Self {
        Self {
            destination,
            _marker: PhantomData,
        }
    }
}

fn read4(iter: &mut IncomingArrayValueIter<'_>) -> Result<Option<(u8, u8, u8, u8)>, DBusError> {
    let Some(b1) = iter.try_next()? else {
        return Ok(None);
    };
    value_is!(b1, IncomingValue::Byte(b1));

    let Some(b2) = iter.try_next()? else {
        return Ok(None);
    };
    value_is!(b2, IncomingValue::Byte(b2));

    let Some(b3) = iter.try_next()? else {
        return Ok(None);
    };
    value_is!(b3, IncomingValue::Byte(b3));

    let Some(b4) = iter.try_next()? else {
        return Ok(None);
    };
    value_is!(b4, IncomingValue::Byte(b4));

    Ok(Some((b1, b2, b3, b4)))
}
