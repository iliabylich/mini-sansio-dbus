use crate::{OutgoingMessage, OutgoingValue};

/// Represents a request to get all object properties
pub struct GetAllProperties;

impl GetAllProperties {
    /// constructor
    pub fn build(
        destination: impl Into<String>,
        path: impl Into<String>,
        interface: impl Into<String>,
    ) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: path.into(),
            member: String::from("GetAll"),
            interface: Some(String::from("org.freedesktop.DBus.Properties")),
            destination: Some(destination.into()),
            sender: None,
            unix_fds: None,
            body: vec![OutgoingValue::String(interface.into())],
        }
    }
}
