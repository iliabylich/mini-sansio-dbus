use crate::{
    DBusError, EncodeError, IncomingMessage, MessageType, OutgoingQueue,
    messages::org_freedesktop_dbus::{Subscribe, Unsubscribe},
    messaging::DBusEncode,
};

struct SubscribeToNewIcon;
impl DBusEncode for SubscribeToNewIcon {
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

struct UnsubscribeFromNewIcon;
impl DBusEncode for UnsubscribeFromNewIcon {
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

/// A helper struct to subscribe, unsubscribe, and handle `NewIconSignal` signal
pub struct NewIconSignal;

impl NewIconSignal {
    /// Subscribes
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given `buf`
    pub fn subscribe<Q>(buf: &mut [u8], q: &mut Q, address: &str) -> Result<(), DBusError>
    where
        Q: OutgoingQueue,
    {
        let buf = SubscribeToNewIcon::encode(address, buf)?;
        q.push_raw_buf(buf);
        Ok(())
    }

    /// Unsubscribes
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given `buf`
    pub fn unsubscribe<Q>(buf: &mut [u8], q: &mut Q, address: &str) -> Result<(), DBusError>
    where
        Q: OutgoingQueue,
    {
        let buf = UnsubscribeFromNewIcon::encode(address, buf)?;
        q.push_raw_buf(buf);
        Ok(())
    }

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
