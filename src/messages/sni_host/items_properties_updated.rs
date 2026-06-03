use crate::{
    EncodeError, IncomingMessage, MessageType,
    messages::org_freedesktop_dbus::{Subscribe, Unsubscribe},
    messaging::DBusEncode,
};

/// Subscribes to `ItemsPropertiesUpdated` signal
///
/// # Errors
///
/// Returns an error if message doesn't fit into given `buf`
pub struct ItemsPropertiesUpdatedSubscribe;
impl DBusEncode for ItemsPropertiesUpdatedSubscribe {
    type Args<'a> = (&'a str, &'a str);

    fn encode<'a>(
        (address, path): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        Subscribe::encode(
            buf,
            Some(address),
            Some(path),
            Some("com.canonical.dbusmenu"),
            Some("ItemsPropertiesUpdated"),
        )
    }
}

/// Unsubscribes from `ItemsPropertiesUpdated` signal
///
/// # Errors
///
/// Returns an error if message doesn't fit into given `buf`
pub struct ItemsPropertiesUpdatedUnsubscribe;
impl DBusEncode for ItemsPropertiesUpdatedUnsubscribe {
    type Args<'a> = (&'a str, &'a str);

    fn encode<'a>(
        (address, path): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        Unsubscribe::encode(
            buf,
            Some(address),
            Some(path),
            Some("com.canonical.dbusmenu"),
            Some("ItemsPropertiesUpdated"),
        )
    }
}

/// A helper struct to handle `ItemsPropertiesUpdated` signal
pub struct ItemsPropertiesUpdatedSignal;
impl ItemsPropertiesUpdatedSignal {
    /// Returns true if given message represents an `ItemsPropertiesUpdated` signal
    #[must_use]
    pub fn matches(message: IncomingMessage<'_>, address: &str, path: &str) -> bool {
        message.message_type == MessageType::Signal
            && message.interface == Some("com.canonical.dbusmenu")
            && message.sender == Some(address)
            && message.path == Some(path)
    }
}
