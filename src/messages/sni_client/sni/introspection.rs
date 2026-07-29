use crate::{
    EncodeError, IncomingMessage, IncomingValue, MessageType, SliceMessageEncoder, dbus_body,
    messages::{
        ErrorNoMethod,
        sni_client::sni::{Property, StatusNotifierItemData},
    },
    messaging::DBusEncode,
};

/// Helper struct to handle introspection and property requests for an SNI item
pub struct StatusNotifierItemHandler<'d> {
    destination: &'d str,
}

impl<'d> StatusNotifierItemHandler<'d> {
    /// Constructor
    #[must_use]
    pub const fn new(destination: &'d str) -> Self {
        Self { destination }
    }

    /// Tries to process an introspection or property request for `/StatusNotifierItem`
    ///
    /// # Errors
    ///
    /// Returns an error if the reply cannot be encoded into `buf`
    pub fn handle<'a>(
        &self,
        buf: &'a mut [u8],
        message: IncomingMessage<'_>,
        data: &impl StatusNotifierItemData,
    ) -> Result<Option<&'a [u8]>, EncodeError> {
        let Some(req) = Request::parse(message, self.destination) else {
            return Ok(None);
        };
        let Some(sender) = message.sender else {
            return Ok(None);
        };
        let serial = message.serial;

        match req {
            Request::IntrospectRoot => {
                let res = introspection_reply(buf, serial, sender, ROOT_XML)?;
                Ok(Some(res))
            }
            Request::IntrospectStatusNotifier => {
                let res = introspection_reply(buf, serial, sender, ITEM_XML)?;
                Ok(Some(res))
            }

            Request::GetProperty { property } => {
                let res = get_property_reply(buf, serial, sender, property, data)?;
                Ok(Some(res))
            }
            Request::GetAllProperties => {
                let res = get_all_properties_reply(buf, serial, sender, data)?;
                Ok(Some(res))
            }

            Request::Other => {
                let res = ErrorNoMethod::encode((sender, serial), buf)?;
                Ok(Some(res))
            }
        }
    }
}

fn introspection_reply<'a>(
    buf: &'a mut [u8],
    serial: u32,
    destination: &str,
    xml: &str,
) -> Result<&'a [u8], EncodeError> {
    reply_with_body(buf, serial, destination, |encoder| {
        dbus_body!(encoder, { str(xml) });
        Ok(())
    })
}

fn get_property_reply<'a>(
    buf: &'a mut [u8],
    serial: u32,
    destination: &str,
    property: Property,
    data: &impl StatusNotifierItemData,
) -> Result<&'a [u8], EncodeError> {
    reply_with_body(buf, serial, destination, |encoder| {
        encoder.set_body_signature("v")?;
        encoder.__dbus_begin_body()?;
        property.encode_value(encoder, data)
    })
}

fn get_all_properties_reply<'a>(
    buf: &'a mut [u8],
    serial: u32,
    destination: &str,
    data: &impl StatusNotifierItemData,
) -> Result<&'a [u8], EncodeError> {
    reply_with_body(buf, serial, destination, |encoder| {
        encoder.set_body_signature("a{sv}")?;
        encoder.__dbus_begin_body()?;
        let properties = encoder.__dbus_start_array(8)?;

        Property::Category.encode_key_value(encoder, data)?;
        Property::Id.encode_key_value(encoder, data)?;
        Property::Title.encode_key_value(encoder, data)?;
        Property::Status.encode_key_value(encoder, data)?;
        Property::IconName.encode_key_value(encoder, data)?;
        Property::IconPixmap.encode_key_value(encoder, data)?;
        Property::Menu.encode_key_value(encoder, data)?;
        Property::ItemIsMenu.encode_key_value(encoder, data)?;

        encoder.__dbus_finish_array(properties)?;
        Ok(())
    })
}

fn reply_with_body<'a>(
    buf: &'a mut [u8],
    serial: u32,
    destination: &str,
    write_body: impl FnOnce(&mut SliceMessageEncoder<'_>) -> Result<(), EncodeError>,
) -> Result<&'a [u8], EncodeError> {
    let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodReturn)?;
    encoder.set_reply_serial(serial)?;
    encoder.set_destination(destination)?;
    write_body(&mut encoder)?;
    let len = encoder.finish()?;
    buf.get(..len).ok_or(EncodeError::BufferTooSmall)
}

enum Request {
    IntrospectRoot,
    IntrospectStatusNotifier,
    GetProperty { property: Property },
    GetAllProperties,
    Other,
}

impl Request {
    fn parse(message: IncomingMessage<'_>, destination: &str) -> Option<Self> {
        if message.message_type != MessageType::MethodCall {
            return None;
        }

        let dst = message.destination?;
        if dst != destination && !dst.starts_with(':') {
            return None;
        }

        let path = message.path?;
        let member = message.member?;
        let interface = message.interface?;

        let req = match interface {
            "org.freedesktop.DBus.Introspectable" if member == "Introspect" => {
                if path == "/" {
                    Self::IntrospectRoot
                } else if path == "/StatusNotifierItem" {
                    Self::IntrospectStatusNotifier
                } else {
                    Self::Other
                }
            }
            "org.freedesktop.DBus.Properties" if member == "Get" => {
                let mut body = message.body?;
                let Ok(Some(IncomingValue::String(interface))) = body.try_next() else {
                    return Some(Self::Other);
                };
                let Ok(Some(IncomingValue::String(property_name))) = body.try_next() else {
                    return Some(Self::Other);
                };
                if path != "/StatusNotifierItem" {
                    return Some(Self::Other);
                }
                if !matches!(
                    interface,
                    "org.kde.StatusNotifierItem" | "org.freedesktop.StatusNotifierItem"
                ) {
                    return Some(Self::Other);
                }
                let property = Property::parse(property_name)?;
                Self::GetProperty { property }
            }
            "org.freedesktop.DBus.Properties" if member == "GetAll" => {
                let mut body = message.body?;
                let Ok(Some(IncomingValue::String(interface))) = body.try_next() else {
                    return Some(Self::Other);
                };
                if path != "/StatusNotifierItem" {
                    return Some(Self::Other);
                }
                if !matches!(
                    interface,
                    "org.kde.StatusNotifierItem" | "org.freedesktop.StatusNotifierItem"
                ) {
                    return Some(Self::Other);
                }
                Self::GetAllProperties
            }
            "org.freedesktop.DBus.Properties" | "org.freedesktop.DBus.Peer" => Self::Other,
            _ => return None,
        };

        Some(req)
    }
}

const ROOT_XML: &str = include_str!("root.xml");
const ITEM_XML: &str = include_str!("item.xml");
