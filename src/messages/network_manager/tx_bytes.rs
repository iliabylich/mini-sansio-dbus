use crate::{Conf, DBusError, IncomingValue, messaging::property::Property, value_is};

/// `TxBytes` propertty of an access point
#[derive(Clone)]
pub struct TxBytes<P>
where
    P: AsRef<str> + Clone,
{
    path: P,
}

impl<P> TxBytes<P>
where
    P: AsRef<str> + Clone,
{
    /// Constructor
    pub const fn new(path: P) -> Self {
        Self { path }
    }
}

impl<P> Property for TxBytes<P>
where
    P: AsRef<str> + Clone,
{
    type Output<'a> = u64;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PATH: Conf<str, Self> = Conf::dynamic(|this| this.path.as_ref());
    const INTERFACE: Conf<str, Self> =
        Conf::constant("org.freedesktop.NetworkManager.Device.Statistics");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("TxBytes");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        value_is!(value, IncomingValue::UInt64(value));
        Ok(value)
    }
}
