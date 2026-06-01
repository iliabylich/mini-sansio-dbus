use crate::{Conf, DBusError, IncomingValue, messaging::property::Property, value_is};

/// The first item of `Devices` property of a connection
#[derive(Clone)]
#[must_use]
pub struct PrimaryDevice<P>
where
    P: AsRef<str> + Clone,
{
    path: P,
}

impl<P> PrimaryDevice<P>
where
    P: AsRef<str> + Clone,
{
    /// Constructor
    pub const fn new(path: P) -> Self {
        Self { path }
    }
}

impl<P> Property for PrimaryDevice<P>
where
    P: AsRef<str> + Clone,
{
    type Output<'a> = &'a str;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PATH: Conf<str, Self> = Conf::dynamic(|this| this.path.as_ref());
    const INTERFACE: Conf<str, Self> =
        Conf::constant("org.freedesktop.NetworkManager.Connection.Active");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("Devices");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        value_is!(value, IncomingValue::Array(devices));
        let mut iter = devices.items_iter();
        let device = iter
            .try_next()?
            .ok_or(DBusError::Other("expected at least one device"))?;
        value_is!(device, IncomingValue::ObjectPath(device));

        Ok(device)
    }
}
