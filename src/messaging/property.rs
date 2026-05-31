use crate::{
    DBusError, EncodeError, IncomingBody, IncomingMessage, IncomingValue, MessageType,
    OutgoingQueue,
    messages::org_freedesktop_dbus::{GetProperty, Subscribe, Unsubscribe},
    messaging::reply_handler::{HandleReply, ReplyErrorHandler, ReplyHandler},
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

/// A helper trait to:
/// 1. get property value
/// 2. subscribe and unsubscribe from its changes
pub trait Property {
    /// Desired output
    type Output;

    /// Destination
    const DESTINATION: Conf<str, Self>;
    /// Path
    const PATH: Conf<str, Self>;
    /// Interface
    const INTERFACE: Conf<str, Self>;
    /// Property name
    const PROPERTY_NAME: Conf<str, Self>;

    /// Maps returned `DBus` value to desired output
    ///
    /// # Errors
    ///
    /// May return an error that will be returned form a `handle` method
    fn map(value: IncomingValue<'_>) -> Result<Self::Output, DBusError>;

    /// Subscribes
    ///
    /// # Errors
    ///
    /// Returns an error if given buffer is too short
    fn subscribe<Q>(&self, buf: &mut [u8], q: &mut Q) -> Result<u32, EncodeError>
    where
        Q: OutgoingQueue,
    {
        let buf = Subscribe::encode(
            buf,
            Some(Self::DESTINATION.resolve(self)),
            Some(Self::PATH.resolve(self)),
            Some("org.freedesktop.DBus.Properties"),
            Some("PropertiesChanged"),
        )?;
        Ok(q.push_raw_buf(buf))
    }

    /// Unsubscribes
    ///
    /// # Errors
    ///
    /// Returns an error if given buffer is too short
    fn unsubscribe<Q>(&self, buf: &mut [u8], q: &mut Q) -> Result<u32, EncodeError>
    where
        Q: OutgoingQueue,
    {
        let buf = Unsubscribe::encode(
            buf,
            Some(Self::DESTINATION.resolve(self)),
            Some(Self::PATH.resolve(self)),
            Some("org.freedesktop.DBus.Properties"),
            Some("PropertiesChanged"),
        )?;
        Ok(q.push_raw_buf(buf))
    }

    /// Pushes a get request into a given queue
    ///
    /// # Errors
    ///
    /// Returns an error if given buffer is too short.
    fn get<Q>(&self, buf: &mut [u8], q: &mut Q) -> Result<ReplyHandler, EncodeError>
    where
        Self: HandleReply + Sized,
        Q: OutgoingQueue,
    {
        let buf = GetProperty::encode(
            buf,
            Self::DESTINATION.resolve(self),
            Self::PATH.resolve(self),
            Self::INTERFACE.resolve(self),
            Self::PROPERTY_NAME.resolve(self),
        )?;
        let serial = q.push_raw_buf(buf);
        Ok(ReplyHandler::new(serial))
    }

    /// Parses incoming message and returns changed Property value if:
    /// 1. it's a signal
    /// 2. it belongs to configured `PATH` and `INTERFACE`
    /// 3. one of the properties is `PROPERTY_NAME`
    ///
    /// # Errors
    ///
    /// Returns an error if given message is malformed.
    fn handle_signal(
        &self,
        message: IncomingMessage<'_>,
    ) -> Result<Option<Self::Output>, DBusError> {
        let Some(value) = find_property_in_properties_changes_reply(
            message,
            Self::PATH.resolve(self),
            Self::INTERFACE.resolve(self),
            Self::PROPERTY_NAME.resolve(self),
        )?
        else {
            return Ok(None);
        };

        Ok(Some(Self::map(value)?))
    }
}

impl<T> HandleReply for T
where
    T: Property,
{
    type Output = <Self as Property>::Output;

    fn handle_reply(mut body: IncomingBody<'_>) -> Result<Self::Output, DBusError> {
        let item = body
            .try_next()?
            .ok_or(DBusError::Other("expected Body to have one value"))?;
        value_is!(item, IncomingValue::Variant(item));
        let item = item.materialize()?;
        let x = Self::map(item)?;
        Ok(x)
    }
}

/// A helper struct to combine getting a property and subscribing to its changes at the same time
pub struct PropertyGetAndSubscribe<P>
where
    P: Property,
{
    property: P,
    reply_handler: ReplyHandler,
}

impl<P> PropertyGetAndSubscribe<P>
where
    P: Property,
{
    /// Fires get + subscribe
    ///
    /// # Errors
    ///
    /// Returns an error if either `Get` or `AddMatch` message doesn't fit into a buffer
    pub fn get_and_subscribe<Q>(property: P, buf: &mut [u8], q: &mut Q) -> Result<Self, EncodeError>
    where
        Q: OutgoingQueue,
    {
        let reply_handler = property.get(buf, q)?;
        let _ = property.subscribe(buf, q)?;
        Ok(Self {
            property,
            reply_handler,
        })
    }

    /// Fires get
    ///
    /// # Errors
    ///
    /// Returns an error if `Get` message doesn't fit into a buffer
    pub fn get<Q>(property: P, buf: &mut [u8], q: &mut Q) -> Result<Self, EncodeError>
    where
        Q: OutgoingQueue,
    {
        let reply_handler = property.get(buf, q)?;
        Ok(Self {
            property,
            reply_handler,
        })
    }

    /// Fires subscribe
    ///
    /// # Errors
    ///
    /// Returns an error if `AddMatch` message doesn't fit into a buffer
    pub fn subscribe<Q>(&self, buf: &mut [u8], q: &mut Q) -> Result<(), EncodeError>
    where
        Q: OutgoingQueue,
    {
        self.property.subscribe(buf, q)?;
        Ok(())
    }

    /// Fires unsubscribe
    ///
    /// # Errors
    ///
    /// Returns an error if `RemoveMatch` message doesn't fit into a buffer
    pub fn unsubscribe<Q>(self, buf: &mut [u8], q: &mut Q) -> Result<(), EncodeError>
    where
        Q: OutgoingQueue,
    {
        self.property.unsubscribe(buf, q)?;
        Ok(())
    }

    /// Handles both reply and signal
    ///
    /// # Errors
    ///
    /// Returns an error if the message is invalid either as a matching reply or as a matching signal.
    pub fn handle_reply_or_signal<E>(
        &self,
        message: IncomingMessage<'_>,
    ) -> Result<Option<P::Output>, DBusError>
    where
        E: ReplyErrorHandler,
    {
        if let Some(out) = self.reply_handler.handle::<P, E>(message)? {
            Ok(Some(out))
        } else if let Some(out) = self.property.handle_signal(message)? {
            Ok(Some(out))
        } else {
            Ok(None)
        }
    }
}
