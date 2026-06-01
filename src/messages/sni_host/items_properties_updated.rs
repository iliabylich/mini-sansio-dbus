use crate::{
    DBusError, EncodeError, IncomingMessage, MessageType, OutgoingQueue,
    messages::org_freedesktop_dbus::{Subscribe, Unsubscribe},
    messaging::DBusEncode,
};

struct ItemsPropertiesUpdatedArgs<'a> {
    address: &'a str,
    path: &'a str,
}

struct SubscribeToItemsPropertiesUpdated;
impl DBusEncode for SubscribeToItemsPropertiesUpdated {
    type Args<'a> = ItemsPropertiesUpdatedArgs<'a>;

    fn encode<'a>(
        ItemsPropertiesUpdatedArgs { address, path }: Self::Args<'_>,
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

struct UnsubscribeFromItemsPropertiesUpdated;
impl DBusEncode for UnsubscribeFromItemsPropertiesUpdated {
    type Args<'a> = ItemsPropertiesUpdatedArgs<'a>;

    fn encode<'a>(
        ItemsPropertiesUpdatedArgs { address, path }: Self::Args<'_>,
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

/// A helper struct to subscribe, unsubscribe, and handle `ItemsPropertiesUpdated` signal
pub struct ItemsPropertiesUpdatedSignal;
impl ItemsPropertiesUpdatedSignal {
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
    ) -> Result<(), DBusError>
    where
        Q: OutgoingQueue,
    {
        let args = ItemsPropertiesUpdatedArgs { address, path };
        let buf = SubscribeToItemsPropertiesUpdated::encode(args, buf)?;
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
    ) -> Result<(), DBusError>
    where
        Q: OutgoingQueue,
    {
        let args = ItemsPropertiesUpdatedArgs { address, path };
        let buf = UnsubscribeFromItemsPropertiesUpdated::encode(args, buf)?;
        q.push_raw_buf(buf);
        Ok(())
    }

    /// Returns true if given message represents an `ItemsPropertiesUpdated` signal
    #[must_use]
    pub fn matches(message: IncomingMessage<'_>, address: &str, path: &str) -> bool {
        message.message_type == MessageType::Signal
            && message.interface == Some("com.canonical.dbusmenu")
            && message.sender == Some(address)
            && message.path == Some(path)
    }
}
