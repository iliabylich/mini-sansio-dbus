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

/// Defines known at compile-time encoded call to subscribe to a given destination + path + interface + member
#[macro_export]
macro_rules! def_static_subscribe_to_properties_changed {
    (
        name = $name:ident,
        size = $size:expr,
        destination = $destination:expr,
        path = $path:expr,
        interface = $interface:expr
    ) => {
        $crate::def_constant_message!(
            name = $name,
            size = $size,
            |buf| => $crate::messages::org_freedesktop_dbus::Subscribe::encode(
                buf,
                $destination,
                $path,
                $interface,
                Some("PropertiesChanged"),
            )
        );
    };
}

/// Defines known at compile-time encoded call to unubscribe from a given destination + path + interface + member
#[macro_export]
macro_rules! def_static_unsubscribe_to_properties_changed {
    (
        name = $name:ident,
        size = $size:expr,
        destination = $destination:expr,
        path = $path:expr,
        interface = $interface:expr
    ) => {
        $crate::def_constant_message!(
            name = $name,
            size = $size,
            |buf| => $crate::messages::org_freedesktop_dbus::Unsubscribe::encode(
                buf,
                $destination,
                $path,
                $interface,
                Some("PropertiesChanged"),
            )
        );
    };
}

/// Defines known at compile time struct that can do a `GetProperty` call
#[macro_export]
macro_rules! def_static_property_get {
    (
        name = $name:ident,
        size = $size:expr,
        destination = $destination:expr,
        path = $path:expr,
        interface = $interface:expr,
        property = $property:expr,
        |$var:ident| => $out:ty { $eval:expr }
    ) => {
        $crate::def_constant_message!(
            name = $name,
            size = $size,
            with-reply,
            |buf| => $crate::messages::org_freedesktop_dbus::GetProperty::encode(
                buf,
                $destination,
                $path,
                $interface,
                $property,
            )
        );

        impl $crate::messaging::reply_handler::HasReplyHandler for $name {
            type Output = $out;

            fn handle(&self, mut body: $crate::IncomingBody<'_>) -> Result<Self::Output, $crate::DBusError> {
                let item = body
                    .try_next()?
                    .ok_or(DBusError::Other("expected Body to have one value"))?;
                $crate::value_is!(item, $crate::IncomingValue::Variant(item));
                let $var = item.materialize()?;
                $eval
            }
        }

    };
}
