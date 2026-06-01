use crate::{Conf, DBusError, IncomingValue, messaging::property::Property, value_is};

/// `Ssid` property of an access point
#[derive(Clone)]
pub struct SSID<const N: usize, P>
where
    P: AsRef<str> + Clone,
{
    path: P,
}

impl<const N: usize, P> SSID<N, P>
where
    P: AsRef<str> + Clone,
{
    /// Constructor
    pub const fn new(path: P) -> Self {
        Self { path }
    }
}

impl<const N: usize, P> Property for SSID<N, P>
where
    P: AsRef<str> + Clone,
{
    type Output<'a> = ([u8; N], usize);

    const DESTINATION: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager");
    const PATH: Conf<str, Self> = Conf::dynamic(|this| this.path.as_ref());
    const INTERFACE: Conf<str, Self> = Conf::constant("org.freedesktop.NetworkManager.AccessPoint");
    const PROPERTY_NAME: Conf<str, Self> = Conf::constant("Ssid");

    fn map(value: IncomingValue<'_>) -> Result<Self::Output<'_>, DBusError> {
        value_is!(value, IncomingValue::Array(value));
        let mut iter = value.items_iter();

        let mut bytes = [0; N];
        let mut len = 0;

        while let Some(byte) = iter.try_next()? {
            const ERR: DBusError = DBusError::Other("SSID is too long");

            value_is!(byte, IncomingValue::Byte(byte));
            *bytes.get_mut(len).ok_or(ERR)? = byte;
            len = len.checked_add(1).ok_or(ERR)?;
        }
        Ok((bytes, len))
    }
}
