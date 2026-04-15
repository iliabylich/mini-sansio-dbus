use crate::{OutgoingMessage, OutgoingValue};

pub struct RemoveMatch;

impl RemoveMatch {
    pub fn build(path: impl AsRef<str>) -> OutgoingMessage {
        Self::build_from_rule(format!(
            "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{}'",
            path.as_ref()
        ))
    }

    pub fn build_from_rule(rule: impl Into<String>) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: String::from("/org/freedesktop/DBus"),
            member: String::from("RemoveMatch"),
            interface: Some(String::from("org.freedesktop.DBus")),
            destination: Some(String::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![OutgoingValue::String(rule.into())],
        }
    }
}
