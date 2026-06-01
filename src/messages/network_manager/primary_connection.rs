use crate::{Conf, DBusError, IncomingValue, messaging::property::Property, value_is};

/// `PrimaryConnection` property of `NetworkManager`
#[derive(Clone)]
pub struct PrimaryConnection;

impl Property for PrimaryConnection {
    type Output<'a> = &'a str;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PATH: Conf<str, Self> = Conf::constant("/org/freedesktop/NetworkManager");
    const INTERFACE: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("PrimaryConnection");

    fn map(value: IncomingValue<'_>) -> Result<&str, DBusError> {
        value_is!(value, IncomingValue::ObjectPath(value));
        Ok(value)
    }
}
