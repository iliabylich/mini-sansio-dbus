use crate::{OutgoingMessage, OutgoingValue};

pub struct GetProperty;

impl GetProperty {
    pub fn build(
        destination: impl Into<String>,
        path: impl Into<String>,
        interface: impl Into<String>,
        property: impl Into<String>,
    ) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: path.into(),
            member: String::from("Get"),
            interface: Some(String::from("org.freedesktop.DBus.Properties")),
            destination: Some(destination.into()),
            sender: None,
            unix_fds: None,
            body: vec![
                OutgoingValue::String(interface.into()),
                OutgoingValue::String(property.into()),
            ],
        }
    }
}
