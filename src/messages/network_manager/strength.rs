use crate::{Conf, DBusError, IncomingValue, messaging::property::Property, value_is};

/// `Strength` propertty of an access point
#[derive(Clone)]
pub struct Strength<P>
where
    P: AsRef<str> + Clone,
{
    path: P,
}

impl<P> Strength<P>
where
    P: AsRef<str> + Clone,
{
    /// Constructor
    pub const fn new(path: P) -> Self {
        Self { path }
    }
}

impl<P> Property for Strength<P>
where
    P: AsRef<str> + Clone,
{
    type Output<'a> = u8;

    const DESTINATION: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PATH: Conf<str, Self> = Conf::dynamic(|this| this.path.as_ref());
    const INTERFACE: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager.AccessPoint");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("Strength");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        value_is!(value, IncomingValue::Byte(value));
        Ok(value)
    }
}
