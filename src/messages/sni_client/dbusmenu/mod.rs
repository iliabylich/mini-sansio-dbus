mod item;
pub use item::DBusMenuItem;

mod event;
pub use event::{DBusMenuEvent, DBusMenuEventKind};

mod property;
use property::Property;

mod request;
use request::Request;

mod get_layout;
use get_layout::GetLayout;

mod layout_updated;
pub use layout_updated::LayoutUpdatedSignal;

mod get_group_properties;
use get_group_properties::GetGroupProperties;

mod data;
pub use data::{DBusMenuData, DBusMenuList};

use crate::{
    DBusError, EncodeError, IncomingArrayValue, IncomingMessage, IncomingValue, MessageType,
    SliceMessageEncoder, messages::ErrorNoMethod, messaging::DBusEncode, value_is,
};

/// Handles a `com.canonical.dbusmenu` object
pub struct StatusNotifierMenuHandler;

impl StatusNotifierMenuHandler {
    /// Tries to process a dbusmenu request
    ///
    /// # Errors
    ///
    /// Returns an error if the reply cannot be encoded into `buf`
    pub fn handle<'a>(
        buf: &'a mut [u8],
        message: IncomingMessage<'_>,
        destination: &str,
        path: &str,
        data: &mut impl DBusMenuData,
    ) -> Result<Option<&'a [u8]>, DBusError> {
        let Some(sender) = message.sender else {
            return Ok(None);
        };

        let request = match Request::parse(message, destination, path) {
            Ok(Some(request)) => request,
            Ok(None) => return Ok(None),
            Err(_) => {
                let reply = ErrorNoMethod::encode((sender, message.serial), buf)?;
                return Ok(Some(reply));
            }
        };

        let reply = match request {
            Request::GetAllProperties => get_all_properties_reply(buf, message.serial, sender)?,
            Request::GetProperty { property } => {
                get_property_reply(buf, message.serial, sender, property)?
            }
            Request::GetLayout { parent, depth } => {
                GetLayout::reply(buf, message.serial, sender, data, parent, depth)?
            }
            Request::GetGroupProperties { ids } => {
                GetGroupProperties::reply(buf, message.serial, sender, data, ids)?
            }
            Request::AboutToShow { id } => {
                let need_update = data.about_to_show(id);
                about_to_show_reply(buf, message.serial, sender, need_update)?
            }
            Request::AboutToShowGroup { ids } => {
                about_to_show_group_reply(buf, message.serial, sender, ids, data)?
            }
            Request::Event { event } => {
                data.event(event);
                empty_reply(buf, message.serial, sender)?
            }
            Request::EventGroup { events } => {
                handle_event_group(events, data)?;
                event_group_reply(buf, message.serial, sender)?
            }
        };

        Ok(Some(reply))
    }
}

fn about_to_show_group(
    ids: IncomingArrayValue<'_>,
    data: &mut impl DBusMenuData,
    mut write_update: impl FnMut(i32) -> Result<(), EncodeError>,
) -> Result<(), DBusError> {
    let mut iter = ids.items_iter();
    while let Some(value) = iter.try_next()? {
        value_is!(value, IncomingValue::Int32(id));
        if data.about_to_show(id) {
            write_update(id)?;
        }
    }

    Ok(())
}

fn handle_event_group(
    events: IncomingArrayValue<'_>,
    data: &mut impl DBusMenuData,
) -> Result<(), DBusError> {
    let mut events = events.items_iter();
    while let Some(value) = events.try_next()? {
        value_is!(value, IncomingValue::Struct(event));
        let mut fields = event.fields_iter()?;

        let id = fields.try_next()?.ok_or(DBusError::WrongValue)?;
        value_is!(id, IncomingValue::Int32(id));
        let event_id = fields.try_next()?.ok_or(DBusError::WrongValue)?;
        value_is!(event_id, IncomingValue::String(event_id));
        let ignore = fields.try_next()?.ok_or(DBusError::WrongValue)?;
        value_is!(ignore, IncomingValue::Variant(_));
        let timestamp = fields.try_next()?.ok_or(DBusError::WrongValue)?;
        value_is!(timestamp, IncomingValue::UInt32(timestamp));

        data.event(DBusMenuEvent {
            id,
            kind: DBusMenuEventKind::from_str(event_id),
            timestamp,
        });
    }
    Ok(())
}

fn get_all_properties_reply<'a>(
    buf: &'a mut [u8],
    serial: u32,
    destination: &str,
) -> Result<&'a [u8], DBusError> {
    reply(buf, serial, destination, |encoder| {
        encoder.set_body_signature("a{sv}")?;
        encoder.__dbus_begin_body()?;
        let properties = encoder.__dbus_start_array(8)?;
        Property::Version.encode_key_value(encoder)?;
        Property::TextDirection.encode_key_value(encoder)?;
        Property::Status.encode_key_value(encoder)?;
        Property::IconThemePath.encode_key_value(encoder)?;
        encoder.__dbus_finish_array(properties)?;
        Ok(())
    })
}

fn get_property_reply<'a>(
    buf: &'a mut [u8],
    serial: u32,
    destination: &str,
    property: Property,
) -> Result<&'a [u8], DBusError> {
    reply(buf, serial, destination, |encoder| {
        encoder.set_body_signature("v")?;
        encoder.__dbus_begin_body()?;
        property.encode_value(encoder)?;
        Ok(())
    })
}

fn about_to_show_reply<'a>(
    buf: &'a mut [u8],
    serial: u32,
    destination: &str,
    need_update: bool,
) -> Result<&'a [u8], DBusError> {
    reply(buf, serial, destination, |encoder| {
        encoder.set_body_signature("b")?;
        encoder.__dbus_begin_body()?;
        encoder.__dbus_write_bool(need_update)?;
        Ok(())
    })
}

fn about_to_show_group_reply<'a>(
    buf: &'a mut [u8],
    serial: u32,
    destination: &str,
    ids: IncomingArrayValue<'_>,
    data: &mut impl DBusMenuData,
) -> Result<&'a [u8], DBusError> {
    reply(buf, serial, destination, |encoder| {
        encoder.set_body_signature("aiai")?;
        encoder.__dbus_begin_body()?;

        let updates = encoder.__dbus_start_array(4)?;
        about_to_show_group(ids, data, |id| encoder.__dbus_write_i32(id))?;
        encoder.__dbus_finish_array(updates)?;

        let errors = encoder.__dbus_start_array(4)?;
        encoder.__dbus_finish_array(errors)?;
        Ok(())
    })
}

fn event_group_reply<'a>(
    buf: &'a mut [u8],
    serial: u32,
    destination: &str,
) -> Result<&'a [u8], DBusError> {
    reply(buf, serial, destination, |encoder| {
        encoder.set_body_signature("ai")?;
        encoder.__dbus_begin_body()?;
        let errors = encoder.__dbus_start_array(4)?;
        encoder.__dbus_finish_array(errors)?;
        Ok(())
    })
}

fn empty_reply<'a>(
    buf: &'a mut [u8],
    serial: u32,
    destination: &str,
) -> Result<&'a [u8], DBusError> {
    reply(buf, serial, destination, |encoder| {
        encoder.set_body_signature("")?;
        Ok(())
    })
}

fn reply<'a>(
    buf: &'a mut [u8],
    serial: u32,
    destination: &str,
    write_body: impl FnOnce(&mut SliceMessageEncoder<'_>) -> Result<(), DBusError>,
) -> Result<&'a [u8], DBusError> {
    let mut encoder = SliceMessageEncoder::new(buf, MessageType::MethodReturn)?;
    encoder.set_reply_serial(serial)?;
    encoder.set_destination(destination)?;
    write_body(&mut encoder)?;
    let len = encoder.finish()?;
    buf.get(..len)
        .ok_or(DBusError::EncodeError(EncodeError::BufferTooSmall))
}
