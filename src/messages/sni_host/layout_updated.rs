use crate::{
    EncodeError, IncomingMessage, MessageType,
    messages::org_freedesktop_dbus::{Subscribe, Unsubscribe},
    messaging::DBusEncode,
};

/// Subscribes to `LayoutUpdated` signal
///
/// # Errors
///
/// Returns an error if message doesn't fit into given `buf`
pub struct LayoutUpdatedSubscribe;
impl DBusEncode for LayoutUpdatedSubscribe {
    type Args<'a> = (&'a str, &'a str);

    fn encode<'a>(
        (address, path): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        Subscribe::encode(
            (
                Some(address),
                Some(path),
                Some("com.canonical.dbusmenu"),
                Some("LayoutUpdated"),
            ),
            buf,
        )
    }
}

/// Unsubscribes from `LayoutUpdated` signal
///
/// # Errors
///
/// Returns an error if message doesn't fit into given `buf`
pub struct LayoutUpdatedUnsubscribe;
impl DBusEncode for LayoutUpdatedUnsubscribe {
    type Args<'a> = (&'a str, &'a str);

    fn encode<'a>(
        (address, path): Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        Unsubscribe::encode(
            (
                Some(address),
                Some(path),
                Some("com.canonical.dbusmenu"),
                Some("LayoutUpdated"),
            ),
            buf,
        )
    }
}

/// A helper struct to subscribe, unsubscribe, and handle `LayoutUpdatedSignal` signal
pub struct LayoutUpdatedSignal;

impl LayoutUpdatedSignal {
    /// Returns true if given message represents a `LayoutUpdatedSignal` signal
    #[must_use]
    pub fn matches(message: IncomingMessage<'_>, address: &str, path: &str) -> bool {
        message.message_type == MessageType::Signal
            && message.interface == Some("com.canonical.dbusmenu")
            && message.sender == Some(address)
            && message.path == Some(path)
    }
}
