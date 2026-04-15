use crate::{
    DBusError, destination_is,
    incoming::{IncomingMessage, IncomingValue},
    types::MessageType,
    value_is,
};

pub struct IntrospectibleObjectAt {
    destination: &'static str,
}

impl IntrospectibleObjectAt {
    pub fn new(destination: &'static str) -> Self {
        Self { destination }
    }

    pub fn handle<'a>(
        &self,
        message: IncomingMessage<'a>,
    ) -> Result<(u32, &'a str, IntrospectibleObjectAtRequest<'a>), DBusError> {
        if message.message_type != MessageType::MethodCall {
            return Err(DBusError::WrongMessageType(format!(
                "expected: {:?}, got: {:?}",
                MessageType::MethodCall,
                message.message_type
            )));
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
                _ => return Err(DBusError::UnknownMember(member.to_string())),
            },

            "org.freedesktop.DBus.Peer" => match member {
                "GetMachinId" => IntrospectibleObjectAtRequest::GetMachineId,
                "Ping" => IntrospectibleObjectAtRequest::Ping,
                _ => return Err(DBusError::UnknownMember(member.to_string())),
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
                _ => return Err(DBusError::UnknownMember(member.to_string())),
            },

            _ => return Err(DBusError::UnknownInterface(interface.to_string())),
        };

        Ok((serial, sender, req))
    }
}

#[derive(Debug)]
pub enum IntrospectibleObjectAtRequest<'a> {
    Introspect {
        path: &'a str,
    },

    Ping,
    GetMachineId,

    GetProperty {
        path: &'a str,
        interface: &'a str,
        property_name: &'a str,
    },
    GetAllProperties {
        path: &'a str,
        interface: &'a str,
    },
    SetProperty,
}
