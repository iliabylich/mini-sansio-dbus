use crate::{
    DBusError, destination_is,
    incoming::{IncomingMessage, IncomingValue},
    types::MessageType,
    value_is,
};

/// A helper object to provide `DBus` introspection
#[must_use]
pub struct IntrospectibleObjectAt {
    destination: &'static str,
}

impl IntrospectibleObjectAt {
    /// constructor
    pub const fn new(destination: &'static str) -> Self {
        Self { destination }
    }

    /// Tries to process incoming message
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't belong to instrospection protocol or invalid.
    pub fn handle<'a>(
        &self,
        message: IncomingMessage<'a>,
    ) -> Result<(u32, &'a str, IntrospectibleObjectAtRequest<'a>), DBusError> {
        if message.message_type != MessageType::MethodCall {
            return Err(DBusError::WrongMessageType);
        }

        let serial = message.serial;
        let path = message.path.ok_or(DBusError::NoPath)?;
        let member = message.member.ok_or(DBusError::NoMember)?;
        let interface = message.interface.ok_or(DBusError::NoInterface)?;
        let destination = message.destination.ok_or(DBusError::NoDestination)?;
        let sender = message.sender.ok_or(DBusError::NoSender)?;
        let mut body = message.body.ok_or(DBusError::NoBody)?;

        destination_is!(destination, self.destination);

        let req = match interface {
            "org.freedesktop.DBus.Introspectable" => match member {
                "Introspect" => IntrospectibleObjectAtRequest::Introspect { path },
                _ => return Err(DBusError::UnknownMember),
            },

            "org.freedesktop.DBus.Peer" => match member {
                "GetMachinId" => IntrospectibleObjectAtRequest::GetMachineId,
                "Ping" => IntrospectibleObjectAtRequest::Ping,
                _ => return Err(DBusError::UnknownMember),
            },

            "org.freedesktop.DBus.Properties" => match member {
                "Get" => {
                    let interface = body.try_next()?.ok_or(DBusError::NoInterface)?;
                    value_is!(interface, IncomingValue::String(interface));

                    let property_name = body.try_next()?.ok_or(DBusError::NoPropertyName)?;
                    value_is!(property_name, IncomingValue::String(property_name));

                    IntrospectibleObjectAtRequest::GetProperty {
                        path,
                        interface,
                        property_name,
                    }
                }
                "GetAll" => {
                    let interface = body.try_next()?.ok_or(DBusError::NoInterface)?;
                    value_is!(interface, IncomingValue::String(interface));

                    IntrospectibleObjectAtRequest::GetAllProperties { path, interface }
                }
                "Set" => IntrospectibleObjectAtRequest::SetProperty,
                _ => return Err(DBusError::UnknownMember),
            },

            _ => return Err(DBusError::UnknownInterface),
        };

        Ok((serial, sender, req))
    }
}

/// An incoming introspection request, must be handled by you
#[derive(Debug)]
pub enum IntrospectibleObjectAtRequest<'a> {
    /// A request to introspect object at `/<path>`
    Introspect {
        /// Path to introspect
        path: &'a str,
    },

    /// Ping request
    Ping,
    /// Get machine ID request
    GetMachineId,

    /// A request to introspect individual property
    GetProperty {
        /// Path of the object
        path: &'a str,
        /// Interface of the object
        interface: &'a str,
        /// Property name to introspect
        property_name: &'a str,
    },
    /// A request to introspect all properties
    GetAllProperties {
        /// Path of the object
        path: &'a str,
        /// Interface of the object
        interface: &'a str,
    },
    /// A request to set property. Not implemented because it makes no sense.
    SetProperty,
}
