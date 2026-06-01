use crate::{
    EncodeError, IncomingMessage, MessageType, OutgoingQueue,
    messages::org_freedesktop_dbus::{Subscribe, Unsubscribe},
    messaging::DBusEncode,
};

struct LayoutUpdatedArgs<'a> {
    address: &'a str,
    path: &'a str,
}

struct SubscribeToLayoutUpdated;
impl DBusEncode for SubscribeToLayoutUpdated {
    type Args<'a> = LayoutUpdatedArgs<'a>;

    fn encode<'a>(
        LayoutUpdatedArgs { address, path }: Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        Subscribe::encode(
            buf,
            Some(address),
            Some(path),
            Some("com.canonical.dbusmenu"),
            Some("LayoutUpdated"),
        )
    }
}

struct UnsubscribeFromLayoutUpdated;
impl DBusEncode for UnsubscribeFromLayoutUpdated {
    type Args<'a> = LayoutUpdatedArgs<'a>;

    fn encode<'a>(
        LayoutUpdatedArgs { address, path }: Self::Args<'_>,
        buf: &'a mut [u8],
    ) -> Result<&'a [u8], EncodeError> {
        Unsubscribe::encode(
            buf,
            Some(address),
            Some(path),
            Some("com.canonical.dbusmenu"),
            Some("LayoutUpdated"),
        )
    }
}

/// A helper struct to subscribe, unsubscribe, and handle `LayoutUpdatedSignal` signal
pub struct LayoutUpdatedSignal;

impl LayoutUpdatedSignal {
    /// Subscribes
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given `buf`
    pub fn subscribe<Q>(
        buf: &mut [u8],
        q: &mut Q,
        address: &str,
        path: &str,
    ) -> Result<(), EncodeError>
    where
        Q: OutgoingQueue,
    {
        let args = LayoutUpdatedArgs { address, path };
        let buf = SubscribeToLayoutUpdated::encode(args, buf)?;
        q.push_raw_buf(buf);
        Ok(())
    }

    /// Unsubscribes
    ///
    /// # Errors
    ///
    /// Returns an error if message doesn't fit into given `buf`
    pub fn unsubscribe<Q>(
        buf: &mut [u8],
        q: &mut Q,
        address: &str,
        path: &str,
    ) -> Result<(), EncodeError>
    where
        Q: OutgoingQueue,
    {
        let args = LayoutUpdatedArgs { address, path };
        let buf = UnsubscribeFromLayoutUpdated::encode(args, buf)?;
        q.push_raw_buf(buf);
        Ok(())
    }

    /// Returns true if given message represents a `LayoutUpdatedSignal` signal
    #[must_use]
    pub fn matches(message: IncomingMessage<'_>, address: &str, path: &str) -> bool {
        message.message_type == MessageType::Signal
            && message.interface == Some("com.canonical.dbusmenu")
            && message.sender == Some(address)
            && message.path == Some(path)
    }
}
