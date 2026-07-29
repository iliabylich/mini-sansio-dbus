use crate::{
    DBusError, IncomingArrayValue, IncomingMessage, IncomingValue, MessageType, interface_is,
    messages::sni_client::dbusmenu::{DBusMenuEvent, DBusMenuEventKind, property::Property},
    value_is,
};

#[derive(Clone, Copy)]
pub enum Request<'a> {
    GetAllProperties,
    GetProperty { property: Property },
    GetLayout { parent: i32, depth: i32 },
    GetGroupProperties { ids: IncomingArrayValue<'a> },
    AboutToShow { id: i32 },
    AboutToShowGroup { ids: IncomingArrayValue<'a> },
    Event { event: DBusMenuEvent<'a> },
    EventGroup { events: IncomingArrayValue<'a> },
}

impl<'a> Request<'a> {
    pub(crate) fn parse(
        message: IncomingMessage<'a>,
        destination: &str,
        path: &str,
    ) -> Result<Option<Self>, DBusError> {
        use DBusError::{NoBody, WrongValue};

        let destination_matches = message
            .destination
            .is_some_and(|actual| actual == destination || actual.starts_with(':'));

        if message.message_type != MessageType::MethodCall
            || message.path != Some(path)
            || !destination_matches
        {
            return Ok(None);
        }

        let interface = message.interface.ok_or(DBusError::NoInterface)?;
        let member = message.member.ok_or(DBusError::NoMember)?;

        match (interface, member) {
            ("org.freedesktop.DBus.Properties", "GetAll") => {
                let mut body = message.body.ok_or(NoBody)?;
                let interface = body.try_next()?.ok_or(WrongValue)?;
                value_is!(interface, IncomingValue::String(interface));
                interface_is!(interface, "com.canonical.dbusmenu");
                Ok(Some(Self::GetAllProperties))
            }
            ("org.freedesktop.DBus.Properties", "Get") => {
                let mut body = message.body.ok_or(NoBody)?;
                let interface = body.try_next()?.ok_or(WrongValue)?;
                value_is!(interface, IncomingValue::String(interface));
                let property = body.try_next()?.ok_or(WrongValue)?;
                value_is!(property, IncomingValue::String(property));
                interface_is!(interface, "com.canonical.dbusmenu");
                let property = Property::parse(property).ok_or(DBusError::NoPropertyName)?;
                Ok(Some(Self::GetProperty { property }))
            }

            ("com.canonical.dbusmenu", "GetLayout") => {
                let mut body = message.body.ok_or(NoBody)?;
                let parent = body.try_next()?.ok_or(WrongValue)?;
                value_is!(parent, IncomingValue::Int32(parent));
                let depth = body.try_next()?.ok_or(WrongValue)?;
                value_is!(depth, IncomingValue::Int32(depth));
                let property_names = body.try_next()?.ok_or(WrongValue)?;
                value_is!(property_names, IncomingValue::Array(_));
                Ok(Some(Self::GetLayout { parent, depth }))
            }
            ("com.canonical.dbusmenu", "GetGroupProperties") => {
                let mut body = message.body.ok_or(NoBody)?;
                let ids = body.try_next()?.ok_or(WrongValue)?;
                value_is!(ids, IncomingValue::Array(ids));
                let property_names = body.try_next()?.ok_or(WrongValue)?;
                value_is!(property_names, IncomingValue::Array(_));
                Ok(Some(Self::GetGroupProperties { ids }))
            }
            ("com.canonical.dbusmenu", "AboutToShow") => {
                let mut body = message.body.ok_or(NoBody)?;
                let id = body.try_next()?.ok_or(WrongValue)?;
                value_is!(id, IncomingValue::Int32(id));
                Ok(Some(Self::AboutToShow { id }))
            }
            ("com.canonical.dbusmenu", "AboutToShowGroup") => {
                let mut body = message.body.ok_or(NoBody)?;
                let ids = body.try_next()?.ok_or(WrongValue)?;
                value_is!(ids, IncomingValue::Array(ids));
                Ok(Some(Self::AboutToShowGroup { ids }))
            }
            ("com.canonical.dbusmenu", "Event") => {
                let mut body = message.body.ok_or(NoBody)?;
                let id = body.try_next()?.ok_or(WrongValue)?;
                value_is!(id, IncomingValue::Int32(id));
                let event_id = body.try_next()?.ok_or(WrongValue)?;
                value_is!(event_id, IncomingValue::String(event_id));
                let data = body.try_next()?.ok_or(WrongValue)?;
                value_is!(data, IncomingValue::Variant(_));
                let timestamp = body.try_next()?.ok_or(WrongValue)?;
                value_is!(timestamp, IncomingValue::UInt32(timestamp));
                let event = DBusMenuEvent {
                    id,
                    kind: DBusMenuEventKind::from_str(event_id),
                    timestamp,
                };
                Ok(Some(Self::Event { event }))
            }
            ("com.canonical.dbusmenu", "EventGroup") => {
                let mut body = message.body.ok_or(NoBody)?;
                let events = body.try_next()?.ok_or(WrongValue)?;
                value_is!(events, IncomingValue::Array(events));
                Ok(Some(Self::EventGroup { events }))
            }

            (
                "com.canonical.dbusmenu"
                | "org.freedesktop.DBus.Properties"
                | "org.freedesktop.DBus.Peer",
                _,
            ) => Err(DBusError::UnknownMember),
            _ => Ok(None),
        }
    }
}
