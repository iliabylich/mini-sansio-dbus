use crate::{Conf, DBusError, IncomingValue, messaging::property::Property, value_is};

/// `IconName` property of the (K)SNI item
#[derive(Clone)]
pub struct IconName<D>
where
    D: AsRef<str> + Clone,
{
    destination: D,
}

impl<D> Property for IconName<D>
where
    D: AsRef<str> + Clone,
{
    type Output<'a> = &'a str;

    const DESTINATION: Conf<str, Self> = Conf::dynamic(|this| this.destination.as_ref());
    const PATH: Conf<str, Self> = Conf::constant("/StatusNotifierItem");
    const INTERFACE: Conf<str, Self> = Conf::constant("org.kde.StatusNotifierItem");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("IconName");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        value_is!(value, IncomingValue::String(value));
        Ok(value)
    }
}

impl<D> IconName<D>
where
    D: AsRef<str> + Clone,
{
    /// Constructor
    pub const fn new(destination: D) -> Self {
        Self { destination }
    }
}
