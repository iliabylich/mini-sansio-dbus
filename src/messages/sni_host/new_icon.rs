use crate::{
    EncodeError, IncomingMessage, MessageType,
    messages::org_freedesktop_dbus::{Subscribe, Unsubscribe},
    messaging::DBusEncode,
};

/// Subscribes to `NewIcon` signal
///
/// # Errors
///
/// Returns an error if message doesn't fit into given `buf`
pub struct NewIconSubscribe;
impl DBusEncode for NewIconSubscribe {
    type Args<'a> = &'a str;

    fn encode<'a>(address: Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        Subscribe::encode(
            buf,
            Some(address),
            Some("/StatusNotifierItem"),
            Some("org.kde.StatusNotifierItem"),
            Some("NewIcon"),
        )
    }
}

/// Unsubscribes from `NewIcon` signal
///
/// # Errors
///
/// Returns an error if message doesn't fit into given `buf`
pub struct NewIconUnsubscribe;
impl DBusEncode for NewIconUnsubscribe {
    type Args<'a> = &'a str;

    fn encode<'a>(address: Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        Unsubscribe::encode(
            buf,
            Some(address),
            Some("/StatusNotifierItem"),
            Some("org.kde.StatusNotifierItem"),
            Some("NewIcon"),
        )
    }
}

/// A helper struct to handle `NewIconSignal` signal
pub struct NewIconSignal;

impl NewIconSignal {
    /// Returns true if given message represents a `NewIconSignal` signal
    #[must_use]
    pub fn matches(message: IncomingMessage<'_>, address: &str) -> bool {
        message.message_type == MessageType::Signal
            && message.interface == Some("org.kde.StatusNotifierItem")
            && message.path == Some("/StatusNotifierItem")
            && message.member == Some("NewIcon")
            && message.sender == Some(address)
    }
}
