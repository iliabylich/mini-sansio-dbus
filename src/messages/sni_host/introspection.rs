use crate::{
    DBusError, EncodeError, IncomingMessage, IntrospectibleObjectAt, IntrospectibleObjectAtRequest,
    MessageType, OutgoingQueue, SliceMessageEncoder, dbus_body, messages::ErrorNoMethod,
};

/// Helper struct to handle introspection requests for (K)SNI host
#[must_use]
pub struct StatusNotifierWatcherIntrospection {
    introspection: IntrospectibleObjectAt,
}

impl Default for StatusNotifierWatcherIntrospection {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusNotifierWatcherIntrospection {
    /// Constructor
    pub const fn new() -> Self {
        Self {
            introspection: IntrospectibleObjectAt::new("org.kde.StatusNotifierWatcher"),
        }
    }

    fn reply_err<Q>(q: &mut Q, serial: u32, destination: &str) -> Result<(), EncodeError>
    where
        Q: OutgoingQueue,
    {
        let mut buf = [0; 1_024];
        let buf = ErrorNoMethod::encode(&mut buf, destination, serial)?;
        q.push_raw_buf(buf);
        Ok(())
    }

    fn reply_with_body<Q>(
        q: &mut Q,
        serial: u32,
        destination: &str,
        write_body: impl FnOnce(&mut SliceMessageEncoder<'_>) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError>
    where
        Q: OutgoingQueue,
    {
        let mut buf = [0; 10 * 1_024];
        let mut encoder = SliceMessageEncoder::new(&mut buf, MessageType::MethodReturn)?;
        encoder.set_reply_serial(serial)?;
        encoder.set_destination(destination)?;
        write_body(&mut encoder)?;
        let len = encoder.finish()?;
        let buf = buf.get(..len).ok_or(EncodeError::BufferTooSmall)?;
        q.push_raw_buf(buf);
        Ok(())
    }

    fn reply_protocol_version<Q>(
        q: &mut Q,
        serial: u32,
        destination: &str,
    ) -> Result<(), EncodeError>
    where
        Q: OutgoingQueue,
    {
        Self::reply_with_body(q, serial, destination, |encoder| {
            dbus_body!(encoder, { variant<i32>(42) });
            Ok(())
        })
    }

    fn reply_is_host_registered<Q>(
        q: &mut Q,
        serial: u32,
        destination: &str,
    ) -> Result<(), EncodeError>
    where
        Q: OutgoingQueue,
    {
        Self::reply_with_body(q, serial, destination, |encoder| {
            dbus_body!(encoder, { variant<bool>(true) });
            Ok(())
        })
    }

    fn reply_registered_items<Q>(
        q: &mut Q,
        serial: u32,
        destination: &str,
    ) -> Result<(), EncodeError>
    where
        Q: OutgoingQueue,
    {
        Self::reply_with_body(q, serial, destination, |encoder| {
            dbus_body!(encoder, { variant<array<str>> [] });
            Ok(())
        })
    }

    fn reply_get_all<Q>(q: &mut Q, serial: u32, destination: &str) -> Result<(), EncodeError>
    where
        Q: OutgoingQueue,
    {
        Self::reply_with_body(q, serial, destination, |encoder| {
            encoder.set_body_signature("a{sv}")?;
            encoder.__dbus_begin_body()?;

            let properties = encoder.__dbus_start_array(8)?;

            encoder.__dbus_align(8)?;
            encoder.__dbus_write_string_like("ProtocolVersion")?;
            encoder.__dbus_write_signature_value("i")?;
            encoder.__dbus_write_i32(42)?;

            encoder.__dbus_align(8)?;
            encoder.__dbus_write_string_like("IsStatusNotifierHostRegistered")?;
            encoder.__dbus_write_signature_value("b")?;
            encoder.__dbus_write_bool(true)?;

            encoder.__dbus_align(8)?;
            encoder.__dbus_write_string_like("RegisteredStatusNotifierItems")?;
            encoder.__dbus_write_signature_value("as")?;
            let items = encoder.__dbus_start_array(4)?;
            encoder.__dbus_finish_array(items)?;

            encoder.__dbus_finish_array(properties)?;
            Ok(())
        })
    }

    /// Tries to process given message, and if it's an introspection request:
    ///
    /// 1. encodes reply
    /// 2. pushes it to the queue
    ///
    /// # Errors
    ///
    /// Message encoding can't fail here, but the request may be malformed or invalid.
    pub fn handle<Q>(&self, q: &mut Q, message: IncomingMessage<'_>) -> Result<bool, DBusError>
    where
        Q: OutgoingQueue,
    {
        let Some((serial, sender, req)) = self.introspection.handle(message) else {
            return Ok(false);
        };

        match req {
            IntrospectibleObjectAtRequest::Introspect { path } => match path {
                "/" => {
                    let mut buf = [0; 10 * 1_024];
                    let mut encoder =
                        SliceMessageEncoder::new(&mut buf, MessageType::MethodReturn)?;
                    encoder.set_reply_serial(serial)?;
                    encoder.set_destination(sender)?;
                    dbus_body!(&mut encoder, { str(ROOT_INTROSPECTION_XML) });
                    let len = encoder.finish()?;
                    let buf = buf.get(..len).ok_or(EncodeError::BufferTooSmall)?;
                    q.push_raw_buf(buf);
                }
                "/StatusNotifierWatcher" => {
                    let mut buf = [0; 10 * 1_024];
                    let mut encoder =
                        SliceMessageEncoder::new(&mut buf, MessageType::MethodReturn)?;
                    encoder.set_reply_serial(serial)?;
                    encoder.set_destination(sender)?;
                    dbus_body!(&mut encoder, { str(KSNI_INTROSPECTION_XML) });
                    let len = encoder.finish()?;
                    let buf = buf.get(..len).ok_or(EncodeError::BufferTooSmall)?;
                    q.push_raw_buf(buf);
                }
                _ => Self::reply_err(q, serial, sender)?,
            },

            IntrospectibleObjectAtRequest::GetAllProperties { path, interface } => {
                match (path, interface) {
                    ("/StatusNotifierWatcher", "org.kde.StatusNotifierWatcher") => {
                        Self::reply_get_all(q, serial, sender)?;
                    }

                    _ => Self::reply_err(q, serial, sender)?,
                }
            }

            IntrospectibleObjectAtRequest::GetProperty {
                path,
                interface,
                property_name,
            } => match (path, interface, property_name) {
                ("/StatusNotifierWatcher", "org.kde.StatusNotifierWatcher", "ProtocolVersion") => {
                    Self::reply_protocol_version(q, serial, sender)?;
                }

                (
                    "/StatusNotifierWatcher",
                    "org.kde.StatusNotifierWatcher",
                    "IsStatusNotifierHostRegistered",
                ) => Self::reply_is_host_registered(q, serial, sender)?,

                (
                    "/StatusNotifierWatcher",
                    "org.kde.StatusNotifierWatcher",
                    "RegisteredStatusNotifierItems",
                ) => Self::reply_registered_items(q, serial, sender)?,

                _ => Self::reply_err(q, serial, sender)?,
            },

            IntrospectibleObjectAtRequest::Ping
            | IntrospectibleObjectAtRequest::GetMachineId
            | IntrospectibleObjectAtRequest::SetProperty => Self::reply_err(q, serial, sender)?,

            IntrospectibleObjectAtRequest::Error(err) => {
                Self::reply_err(q, serial, sender)?;
                return Err(err);
            }
        }

        Ok(true)
    }
}

const BUILTIN_INTERFACES: &[u8] = br#"
<interface name="org.freedesktop.DBus.Introspectable">
    <method name="Introspect">
        <arg type="s" direction="out"/>
    </method>
</interface>

<interface name="org.freedesktop.DBus.Properties">
    <method name="Get">
        <arg name="interface_name" type="s" direction="in"/>
        <arg name="property_name" type="s" direction="in"/>
        <arg type="v" direction="out"/>
    </method>
    <method name="Set">
        <arg name="interface_name" type="s" direction="in"/>
        <arg name="property_name" type="s" direction="in"/>
        <arg name="value" type="v" direction="in"/>
    </method>
    <method name="GetAll">
        <arg name="interface_name" type="s" direction="in"/>
        <arg type="a{sv}" direction="out"/>
    </method>
    <signal name="PropertiesChanged">
        <arg name="interface_name" type="s"/>
        <arg name="changed_properties" type="a{sv}"/>
        <arg name="invalidated_properties" type="as"/>
    </signal>
</interface>

<interface name="org.freedesktop.DBus.Peer">
    <method name="Ping">
    </method>
    <method name="GetMachineId">
        <arg type="s" direction="out"/>
    </method>
</interface>
"#;

const KSNI_INTERFACE: &[u8] = br#"
<interface name="org.kde.StatusNotifierWatcher">
    <method name="RegisterStatusNotifierItem">
        <arg name="service" type="s" direction="in" />
    </method>

    <method name="RegisterStatusNotifierHost">
        <arg name="service" type="s" direction="in" />
    </method>

    <property name="RegisteredStatusNotifierItems" type="as" access="read" />
    <property name="IsStatusNotifierHostRegistered" type="b" access="read" />
    <property name="ProtocolVersion" type="i" access="read" />

    <signal name="StatusNotifierItemRegistered">
        <arg type="s" />
    </signal>
    <signal name="StatusNotifierItemUnregistered">
        <arg type="s" />
    </signal>
    <signal name="StatusNotifierHostRegistered" />
    <signal name="StatusNotifierHostUnregistered" />

</interface>
"#;

const XML_HEADER: &[u8] = br#"
<!DOCTYPE node PUBLIC "-//freedesktop//DTD D-BUS Object Introspection 1.0//EN" "http://www.freedesktop.org/standards/dbus/1.0/introspect.dtd">
"#;

const fn build_ksni_introspection_xml() -> [u8; 10 * 1_024] {
    let mut storage = [0; 10 * 1_024];
    let buf = &mut storage;

    let mut offset = 0;

    macro_rules! push {
        ($s:expr) => {{
            let buflen = buf.len();
            let Some(rest) = get_range_mut(buf, offset, buflen) else {
                panic!("buffer is too small");
            };
            let Some(slice) = get_range_mut(rest, 0, $s.len()) else {
                panic!("buffer is too small");
            };
            slice.copy_from_slice($s);
            let bytes_pushed = $s.len();
            if let Some(new_offset) = offset.checked_add(bytes_pushed) {
                offset = new_offset;
            } else {
                panic!("buffer is too small");
            }
        }};
    }

    push!(XML_HEADER);
    push!(b"\n<node>\n    ");
    push!(KSNI_INTERFACE);
    push!(b"\n    ");
    push!(BUILTIN_INTERFACES);
    push!(b"\n");
    push!(b"</node>\n");
    let _ = offset;

    storage
}

const KSNI_INTROSPECTION_XML: &str = match core::str::from_utf8(&build_ksni_introspection_xml()) {
    Ok(s) => s,
    Err(_) => panic!("malformed root DBus introspection XML"),
};

const fn build_root_introspection_xml() -> [u8; 10 * 1_024] {
    let mut storage = [0; 10 * 1_024];
    let buf = &mut storage;

    let mut offset = 0;

    macro_rules! push {
        ($s:expr) => {{
            let buflen = buf.len();
            let Some(rest) = get_range_mut(buf, offset, buflen) else {
                panic!("buffer is too small");
            };
            let Some(slice) = get_range_mut(rest, 0, $s.len()) else {
                panic!("buffer is too small");
            };
            slice.copy_from_slice($s);
            let bytes_pushed = $s.len();
            if let Some(new_offset) = offset.checked_add(bytes_pushed) {
                offset = new_offset;
            } else {
                panic!("buffer is too small");
            }
        }};
    }

    push!(XML_HEADER);
    push!(b"\n<node>\n    ");
    push!(BUILTIN_INTERFACES);
    push!(b"\n    <node name=\"StatusNotifierWatcher\">\n        ");
    push!(KSNI_INTERFACE);
    push!(b"\n        ");
    push!(BUILTIN_INTERFACES);
    push!(b"\n    </node>\n</node>\n");
    let _ = offset;

    storage
}
const ROOT_INTROSPECTION_XML: &str = match core::str::from_utf8(&build_root_introspection_xml()) {
    Ok(s) => s,
    Err(_) => panic!("malformed root DBus introspection XML"),
};

const fn get_range_mut(buf: &mut [u8], start: usize, end: usize) -> Option<&mut [u8]> {
    let Some((_, tail)) = buf.split_at_mut_checked(start) else {
        return None;
    };
    let Some(offset) = end.checked_sub(start) else {
        return None;
    };
    let Some((head, _)) = tail.split_at_mut_checked(offset) else {
        return None;
    };
    Some(head)
}
