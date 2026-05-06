use crate::{OutgoingMessage, OutgoingValue};

/// Represents a request to subscribe to something in `DBus`
pub struct AddMatch;

impl AddMatch {
    /// constructor
    pub fn build(sender: &str, path: &str) -> OutgoingMessage {
        Self::build_from_rule(format!(
            "type='signal',sender='{sender}',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{path}'"
        ))
    }

    /// low-level constructor
    pub fn build_from_rule(rule: String) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: String::from("/org/freedesktop/DBus"),
            member: String::from("AddMatch"),
            interface: Some(String::from("org.freedesktop.DBus")),
            destination: Some(String::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![OutgoingValue::String(rule)],
        }
    }
}
