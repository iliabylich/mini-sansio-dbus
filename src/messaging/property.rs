use crate::{
    Conf, DBusError, EncodeError, IncomingBody, IncomingMessage, IncomingValue, MessageType,
    messages::org_freedesktop_dbus::{GetProperty, Subscribe, Unsubscribe},
    messaging::reply_handler::HandleReply,
    value_is,
};

/// A helper trait to:
/// 1. get property value
/// 2. subscribe and unsubscribe from its changes
pub trait Property: Clone {
    /// Desired output
    type Output<'a>;

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
    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError>;

    /// Encodes "Subscribe" message
    ///
    /// # Errors
    ///
    /// Returns an error if given buffer is too short
    fn encode_subscribe<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        Subscribe::encode(
            buf,
            Some(Self::DESTINATION.resolve(self)),
            Some(Self::PATH.resolve(self)),
            Some("org.freedesktop.DBus.Properties"),
            Some("PropertiesChanged"),
        )
    }

    /// Encodes "Unsubscribe" message
    ///
    /// # Errors
    ///
    /// Returns an error if given buffer is too short
    fn encode_unsubscribe<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        Unsubscribe::encode(
            buf,
            Some(Self::DESTINATION.resolve(self)),
            Some(Self::PATH.resolve(self)),
            Some("org.freedesktop.DBus.Properties"),
            Some("PropertiesChanged"),
        )
    }

    /// Encodes "Get" request
    ///
    /// # Errors
    ///
    /// Returns an error if given buffer is too short.
    fn encode_get<'a>(&self, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        GetProperty::encode(
            buf,
            Self::DESTINATION.resolve(self),
            Self::PATH.resolve(self),
            Self::INTERFACE.resolve(self),
            Self::PROPERTY_NAME.resolve(self),
        )
    }

    /// Parses incoming message and returns changed Property value if:
    /// 1. it's a signal
    /// 2. it belongs to configured `PATH` and `INTERFACE`
    /// 3. one of the properties is `PROPERTY_NAME`
    ///
    /// # Errors
    ///
    /// Returns an error if given message is malformed.
    fn handle_signal<'a>(
        &self,
        message: IncomingMessage<'a>,
    ) -> Result<Option<Self::Output<'a>>, DBusError> {
        if message.message_type != MessageType::Signal {
            return Ok(None);
        }
        if message.interface != Some("org.freedesktop.DBus.Properties") {
            return Ok(None);
        }
        if message.path != Some(Self::PATH.resolve(self)) {
            return Ok(None);
        }
        let Some(mut body) = message.body else {
            return Ok(None);
        };

        let interface = body
            .try_next()?
            .ok_or(DBusError::Other("no Interface in Body"))?;
        value_is!(interface, IncomingValue::String(interface));
        if interface != Self::INTERFACE.resolve(self) {
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

            if key == Self::PROPERTY_NAME.resolve(self) {
                value_is!(value, IncomingValue::Variant(value));
                let value = value.materialize()?;
                return Ok(Some(Self::map(value)?));
            }
        }

        Ok(None)
    }
}

impl<T> HandleReply for T
where
    T: Property,
{
    type Output<'a> = <Self as Property>::Output<'a>;

    fn handle_reply_body<'a>(
        &self,
        mut body: IncomingBody<'a>,
    ) -> Result<Self::Output<'a>, DBusError> {
        let item = body
            .try_next()?
            .ok_or(DBusError::Other("expected Body to have one value"))?;
        value_is!(item, IncomingValue::Variant(item));
        let item = item.materialize()?;
        Self::map(item)
    }
}
