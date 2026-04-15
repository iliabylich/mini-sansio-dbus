use crate::{OutgoingMessage, OutgoingValue};

pub struct RequestName;

impl RequestName {
    pub fn build(name: impl Into<String>) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: String::from("/org/freedesktop/DBus"),
            member: String::from("RequestName"),
            interface: Some(String::from("org.freedesktop.DBus")),
            destination: Some(String::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![OutgoingValue::String(name.into()), OutgoingValue::UInt32(7)],
        }
    }
}
