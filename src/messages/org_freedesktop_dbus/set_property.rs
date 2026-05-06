use crate::{OutgoingMessage, OutgoingValue};

/// Represets a request to set a single property on a given `DBus` object
pub struct SetProperty;

impl SetProperty {
    /// Constructor
    pub fn build(
        destination: impl Into<String>,
        path: impl Into<String>,
        interface: impl Into<String>,
        property: impl Into<String>,
        value: OutgoingValue,
    ) -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: path.into(),
            member: String::from("Set"),
            interface: Some(String::from("org.freedesktop.DBus.Properties")),
            destination: Some(destination.into()),
            sender: None,
            unix_fds: None,
            body: vec![
                OutgoingValue::String(interface.into()),
                OutgoingValue::String(property.into()),
                OutgoingValue::Variant(Box::new(value)),
            ],
        }
    }
}
