use crate::{
    DBusError, EncodeError, IncomingBody, IncomingMessage, IncomingValue, MessageType,
    OutgoingQueue,
    const_helpers::get_range_mut,
    messages::org_freedesktop_dbus::{Subscribe, Unsubscribe},
    messaging::{DBusEncode, reply_handler::HasReplyHandler},
    value_is,
};
use core::marker::PhantomData;

/// Either constant or dynamic value for Path / Interface / Property name
pub enum Conf<V: ?Sized + 'static, This: ?Sized> {
    #[doc(hidden)]
    Constant(&'static V),
    #[doc(hidden)]
    Dynamic {
        f: for<'a> fn(&'a This) -> &'a V,
        _phantom: PhantomData<This>,
    },
}
impl<V: ?Sized + 'static, This: ?Sized> Conf<V, This> {
    /// Constructs constant variant
    pub const fn constant(value: &'static V) -> Self {
        Self::Constant(value)
    }

    /// Constructs dynamic variant
    pub const fn dynamic(f: for<'a> fn(&'a This) -> &'a V) -> Self {
        Self::Dynamic {
            f,
            _phantom: PhantomData,
        }
    }

    fn resolve<'a>(&self, this: &'a This) -> &'a V {
        match self {
            Self::Constant(v) => v,
            Self::Dynamic { f, _phantom } => (f)(this),
        }
    }
}

/// A helper trait to handle signals on changing a single Property.
pub trait PropertyChangedSignalHandler {
    /// Desired output
    type Output;

    /// Path to subscribe to.
    const PATH: Conf<str, Self>;

    /// Interface to subscribe to.
    const INTERFACE: Conf<str, Self>;

    /// Property to subscribe to.
    const PROPERTY_NAME: Conf<str, Self>;

    /// Parses incoming message and returns changed Property value if:
    /// 1. it's a signal
    /// 2. it belongs to configured `PATH` and `INTERFACE`
    /// 3. one of the properties is `PROPERTY_NAME`
    ///
    /// # Errors
    ///
    /// Returns an error if given message is malformed.
    fn handle(&self, message: IncomingMessage<'_>) -> Result<Option<Self::Output>, DBusError> {
        let Some(value) = find_property_in_properties_changes_reply(
            message,
            Self::PATH.resolve(self),
            Self::INTERFACE.resolve(self),
            Self::PROPERTY_NAME.resolve(self),
        )?
        else {
            return Ok(None);
        };

        Ok(Some(self.map(value)?))
    }

    /// Maps parsed Property value to `Self::Output`
    ///
    /// # Errors
    ///
    /// Can return an error if the value doesn't match the format.
    fn map(&self, value: IncomingValue<'_>) -> Result<Self::Output, DBusError>;
}

fn find_property_in_properties_changes_reply<'a>(
    message: IncomingMessage<'a>,
    path_to_match: &str,
    interface_to_match: &str,
    proeprty_name_to_match: &str,
) -> Result<Option<IncomingValue<'a>>, DBusError> {
    assert!(!path_to_match.is_empty());
    assert!(!interface_to_match.is_empty());
    assert!(!proeprty_name_to_match.is_empty());

    if message.message_type != MessageType::Signal {
        return Ok(None);
    }
    if message.interface != Some("org.freedesktop.DBus.Properties") {
        return Ok(None);
    }
    if message.path != Some(path_to_match) {
        return Ok(None);
    }
    let Some(mut body) = message.body else {
        return Ok(None);
    };

    let interface = body
        .try_next()?
        .ok_or(DBusError::Other("no Interface in Body"))?;
    value_is!(interface, IncomingValue::String(interface));
    if interface != interface_to_match {
        return Ok(None);
    }

    let attributes = body
        .try_next()?
        .ok_or(DBusError::Other("no Attributes in Body"))?;
    value_is!(attributes, IncomingValue::Array(attributes));
    let mut iter = attributes.items_iter();
    while let Some(attribute) = iter.try_next()? {
        value_is!(attribute, IncomingValue::DictEntry(attribute));
        let (key, value) = attribute.key_value()?;
        value_is!(key, IncomingValue::String(key));

        if key == proeprty_name_to_match {
            value_is!(value, IncomingValue::Variant(value));
            let value = value.materialize()?;
            return Ok(Some(value));
        }
    }

    Ok(None)
}

/// A helper struct to quickly subscribe-to and unsubscribe-from `PropertiesChanged` signal
#[must_use]
pub struct PropertySubscriber<'a> {
    destination: Option<&'a str>,
    path: &'a str,
    interface: &'a str,
}

impl<'a> PropertySubscriber<'a> {
    /// Constructor
    pub const fn new(destination: Option<&'a str>, path: &'a str, interface: &'a str) -> Self {
        Self {
            destination,
            path,
            interface,
        }
    }

    /// Subscribes
    ///
    /// # Errors
    ///
    /// Returns an error if given buffer is too short
    pub fn subscribe<Q>(&self, buf: &mut [u8], q: &mut Q) -> Result<u32, EncodeError>
    where
        Q: OutgoingQueue,
    {
        let len = Subscribe::encode(
            buf,
            self.destination,
            Some(self.path),
            Some(self.interface),
            Some("PropertiesChanged"),
        )?;
        let buf = get_range_mut(buf, 0, len).ok_or(EncodeError::BufferTooSmall)?;
        Ok(q.push_raw_buf(buf))
    }

    /// Unsubscribes
    ///
    /// # Errors
    ///
    /// Returns an error if given buffer is too short
    pub fn unsubscribe<Q>(&self, buf: &mut [u8], q: &mut Q) -> Result<u32, EncodeError>
    where
        Q: OutgoingQueue,
    {
        let len = Unsubscribe::encode(
            buf,
            self.destination,
            Some(self.path),
            Some(self.interface),
            Some("PropertiesChanged"),
        )?;
        let buf = get_range_mut(buf, 0, len).ok_or(EncodeError::BufferTooSmall)?;
        Ok(q.push_raw_buf(buf))
    }
}

/// A trait representing a `GetProperty` call with a reply handler
pub trait PropertyGet
where
    Self: Sized + DBusEncode,
{
    /// Output of the call
    type Output;

    /// Maps returned `DBus` value to desired output
    ///
    /// # Errors
    ///
    /// May return an error that will be returned form a `handle` method
    fn map(value: IncomingValue<'_>) -> Result<<Self as PropertyGet>::Output, DBusError>;
}
impl<T> HasReplyHandler for T
where
    T: PropertyGet,
{
    type Output = <T as PropertyGet>::Output;

    fn handle(&self, mut body: IncomingBody<'_>) -> Result<Self::Output, DBusError> {
        let item = body
            .try_next()?
            .ok_or(DBusError::Other("expected Body to have one value"))?;
        value_is!(item, IncomingValue::Variant(item));
        let item = item.materialize()?;
        let x = Self::map(item)?;
        Ok(x)
    }
}
