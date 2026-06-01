use crate::{Conf, DBusError, IncomingValue, messaging::property::Property, value_is};

/// `ActiveAccessPoint` property of a wireless connection
#[derive(Clone)]
pub struct ActiveAccessPoint<P>
where
    P: AsRef<str> + Clone,
{
    path: P,
}

impl<P> ActiveAccessPoint<P>
where
    P: AsRef<str> + Clone,
{
    /// Constructor
    pub const fn new(path: P) -> Self {
        Self { path }
    }
}

impl<P> Property for ActiveAccessPoint<P>
where
    P: AsRef<str> + Clone,
{
    type Output<'a> = &'a str;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PATH: Conf<str, Self> = Conf::dynamic(|this| this.path.as_ref());
    const INTERFACE: Conf<str, Self> =
        Conf::constant("org.freedesktop.NetworkManager.Device.Wireless");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("ActiveAccessPoint");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        value_is!(value, IncomingValue::ObjectPath(value));
        Ok(value)
    }
}
