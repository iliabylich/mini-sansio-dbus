use crate::{
    DBusError, EncodeError, IncomingBody, IncomingMessage, IncomingValue, MessageType,
    OutgoingQueue,
    const_helpers::get_range_mut,
    messages::org_freedesktop_dbus::{Subscribe, Unsubscribe},
    messaging::{
        DBusSend,
        reply_handler::{HasReplyHandler, ReplyErrorHandler, ReplyHandler},
    },
    value_is,
};

/// A helper trait to handle signals on changing a single Property.
pub trait PropertyChangedSignalHandler {
    /// Desired output
    type Output;

    /// Path to subscribe to.
    fn path(&self) -> &str;
    /// Interface to subscribe to.
    fn interface(&self) -> &str;
    /// Property to subscribe to.
    fn property_name(&self) -> &str;

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
            self.path(),
            self.interface(),
            self.property_name(),
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

/// A helper trait to handle signals on changing a single Property, assuming that configuration is static.
pub trait StaticPropertyChangedSignalHandler {
    /// Desired output
    type Output;

    /// Path to subscribe to.
    const PATH: &str;
    /// Interface to subscribe to.
    const INTERFACE: &str;
    /// Property to subscribe to.
    const PROPERTY_NAME: &str;

    /// Parses incoming message and returns changed Property value if:
    /// 1. it's a signal
    /// 2. it belongs to configured `PATH` and `INTERFACE`
    /// 3. one of the properties is `PROPERTY_NAME`
    ///
    /// # Errors
    ///
    /// Returns an error if given message is malformed.
    fn handle(message: IncomingMessage<'_>) -> Result<Option<Self::Output>, DBusError> {
        let Some(value) = find_property_in_properties_changes_reply(
            message,
            Self::PATH,
            Self::INTERFACE,
            Self::PROPERTY_NAME,
        )?
        else {
            return Ok(None);
        };

        Ok(Some(Self::map(value)?))
    }

    /// Maps parsed Property value to `Self::Output`
    ///
    /// # Errors
    ///
    /// Can return an error if the value doesn't match the format.
    fn map(value: IncomingValue<'_>) -> Result<Self::Output, DBusError>;
}

fn find_property_in_properties_changes_reply<'a>(
    message: IncomingMessage<'a>,
    path_to_match: &str,
    interface_to_match: &str,
    proeprty_name_to_match: &str,
) -> Result<Option<IncomingValue<'a>>, DBusError> {
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
    pub fn subscribe<'q, Q>(&self, buf: &'q mut [u8], q: &mut Q) -> Result<u32, EncodeError>
    where
        Q: OutgoingQueue<'q>,
    {
        let len = Subscribe::encode(
            buf,
            self.destination,
            Some(self.path),
            Some(self.interface),
            Some("PropertiesChanged"),
        )?;
        let buf = get_range_mut(buf, 0, len).ok_or(EncodeError::BufferTooSmall)?;
        Ok(q.push(buf))
    }

    /// Unsubscribes
    ///
    /// # Errors
    ///
    /// Returns an error if given buffer is too short
    pub fn unsubscribe<'q, Q>(&self, buf: &'q mut [u8], q: &mut Q) -> Result<u32, EncodeError>
    where
        Q: OutgoingQueue<'q>,
    {
        let len = Unsubscribe::encode(
            buf,
            self.destination,
            Some(self.path),
            Some(self.interface),
            Some("PropertiesChanged"),
        )?;
        let buf = get_range_mut(buf, 0, len).ok_or(EncodeError::BufferTooSmall)?;
        Ok(q.push(buf))
    }
}

/// A trait representing a `GetProperty` call with a reply handler
pub trait PropertyGet
where
    Self: Sized + DBusSend,
{
    /// Output of the call
    type Output;

    /// Pushes encoded message to a given queue and returns a ready-to-use `ReplyHandler`
    ///
    /// # Errors
    ///
    /// Returns an error if a queue returns an error
    fn send_and_prepare_for_reply<'q, Q, E>(
        self,
        q: &mut Q,
        e: E,
    ) -> Result<ReplyHandler<Self, E>, Self::Error>
    where
        Q: OutgoingQueue<'q>,
        E: ReplyErrorHandler,
    {
        let serial = Self::send(q)?;
        Ok(ReplyHandler::new(serial, self, e))
    }

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
