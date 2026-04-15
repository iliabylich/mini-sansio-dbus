use crate::OutgoingMessage;

pub struct Hello;

impl Hello {
    pub fn build() -> OutgoingMessage {
        OutgoingMessage::MethodCall {
            serial: 0,
            path: String::from("/org/freedesktop/DBus"),
            member: String::from("Hello"),
            interface: Some(String::from("org.freedesktop.DBus")),
            destination: Some(String::from("org.freedesktop.DBus")),
            sender: None,
            unix_fds: None,
            body: vec![],
        }
    }
}
