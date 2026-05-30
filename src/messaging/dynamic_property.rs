use crate::{DBusError, IncomingMessage, IncomingValue, MessageType, value_is};

/// A helper trait to handle signals on changing a single Property.
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
        if message.message_type != MessageType::Signal {
            return Ok(None);
        }
        if message.interface != Some("org.freedesktop.DBus.Properties") {
            return Ok(None);
        }
        if message.path != Some(Self::PATH) {
            return Ok(None);
        }
        let Some(mut body) = message.body else {
            return Ok(None);
        };

        let interface = body
            .try_next()?
            .ok_or(DBusError::Other("no Interface in Body"))?;
        value_is!(interface, IncomingValue::String(interface));
        if interface != Self::INTERFACE {
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

            if key == Self::PROPERTY_NAME {
                value_is!(value, IncomingValue::Variant(value));
                let value = value.materialize()?;
                let mapped = Self::map(value)?;
                return Ok(Some(mapped));
            }
        }

        Ok(None)
    }

    /// Maps parsed Property value to `Self::Output`
    ///
    /// # Errors
    ///
    /// Can return an error if the value doesn't match the format.
    fn map(value: IncomingValue<'_>) -> Result<Self::Output, DBusError>;
}
