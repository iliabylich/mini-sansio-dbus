use crate::{
    DBusError,
    incoming::{IncomingMessage, IncomingValue},
    types::MessageType,
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
    #[must_use]
    pub fn handle<'a>(
        &self,
        message: IncomingMessage<'a>,
    ) -> Option<(u32, &'a str, IntrospectibleObjectAtRequest<'a>)> {
        if message.message_type != MessageType::MethodCall {
            return None;
        }
        let serial = message.serial;
        let path = message.path?;
        let member = message.member?;
        let interface = message.interface?;
        let destination = message.destination?;
        let sender = message.sender?;
        let mut body = message.body?;

        if destination != self.destination {
            return None;
        }

        let err = |s: &'static str| {
            Some((
                serial,
                destination,
                IntrospectibleObjectAtRequest::Error(DBusError::Other(s)),
            ))
        };

        let req = match interface {
            "org.freedesktop.DBus.Introspectable" => match member {
                "Introspect" => IntrospectibleObjectAtRequest::Introspect { path },
                _ => return err("unknown method"),
            },

            "org.freedesktop.DBus.Peer" => match member {
                "GetMachinId" => IntrospectibleObjectAtRequest::GetMachineId,
                "Ping" => IntrospectibleObjectAtRequest::Ping,
                _ => return err("unknown method"),
            },

            "org.freedesktop.DBus.Properties" => match member {
                "Get" => {
                    let Ok(Some(IncomingValue::String(interface))) = body.try_next() else {
                        return err("missing or malformed Interface in Body");
                    };

                    let Ok(Some(IncomingValue::String(property_name))) = body.try_next() else {
                        return err("missing or malformed PropertyName in Body");
                    };

                    IntrospectibleObjectAtRequest::GetProperty {
                        path,
                        interface,
                        property_name,
                    }
                }
                "GetAll" => {
                    let Ok(Some(IncomingValue::String(interface))) = body.try_next() else {
                        return err("missing or malformed Interface in Body");
                    };

                    IntrospectibleObjectAtRequest::GetAllProperties { path, interface }
                }
                "Set" => IntrospectibleObjectAtRequest::SetProperty,
                _ => return err("unknown method"),
            },

            _ => return None,
        };

        Some((serial, sender, req))
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

    /// A request that is either invalid or unknown.
    Error(DBusError),
}
