use crate::{Conf, DBusError, IncomingValue, messaging::property::Property, value_is};

/// `Type` property of a connection
#[derive(Clone)]
pub struct ActiveConnectionType<P>
where
    P: AsRef<str> + Clone,
{
    path: P,
}

impl<P> ActiveConnectionType<P>
where
    P: AsRef<str> + Clone,
{
    /// Constructor
    pub const fn new(path: P) -> Self {
        Self { path }
    }
}

impl<P> Property for ActiveConnectionType<P>
where
    P: AsRef<str> + Clone,
{
    type Output<'a> = &'a str;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PATH: Conf<str, Self> = Conf::dynamic(|this| this.path.as_ref());
    const INTERFACE: Conf<str, Self> =
        Conf::constant("org.freedesktop.NetworkManager.Connection.Active");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("Type");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        value_is!(value, IncomingValue::String(value));
        Ok(value)
    }
}
