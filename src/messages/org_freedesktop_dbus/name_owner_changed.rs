use crate::{
    DBusError, EncodeError, IncomingMessage, IncomingValue, MessageType,
    messages::org_freedesktop_dbus::{Subscribe, Unsubscribe},
    messaging::DBusEncode,
    value_is,
};

/// Subscribes to `NameOwnerChanged` signal
pub struct NameOwnerChangedSubscribe;

impl DBusEncode for NameOwnerChangedSubscribe {
    type Args<'a> = ();

    fn encode<'a>((): Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        Subscribe::encode(
            buf,
            Some("org.freedesktop.DBus"),
            Some("/org/freedesktop/DBus"),
            Some("org.freedesktop.DBus"),
            Some("NameOwnerChanged"),
        )
    }
}

/// Unsubscribes from `NameOwnerChanged` signal
pub struct NameOwnerChangedUnsubscribe;

impl DBusEncode for NameOwnerChangedUnsubscribe {
    type Args<'a> = ();

    fn encode<'a>((): Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        Unsubscribe::encode(
            buf,
            Some("org.freedesktop.DBus"),
            Some("/org/freedesktop/DBus"),
            Some("org.freedesktop.DBus"),
            Some("NameOwnerChanged"),
        )
    }
}

/// A helper struct to handle incoming `NameOwnerChanged` signals
pub struct NameOwnerChangedSignal;

impl NameOwnerChangedSignal {
    /// Parses `messages` and if it represents a `NameOwnedChanged` signal returns freed name.
    ///
    /// # Errors
    ///
    /// Returns an error if the message is malformed.
    pub fn handle(message: IncomingMessage<'_>) -> Result<Option<&str>, DBusError> {
        const ERR: DBusError = DBusError::Other("can't parse NameOwnerChanged signal");

        if message.message_type != MessageType::Signal
            || message.path != Some("/org/freedesktop/DBus")
            || message.interface != Some("org.freedesktop.DBus")
            || message.member != Some("NameOwnerChanged")
        {
            return Ok(None);
        }

        let mut body = message.body.ok_or(DBusError::NoBody)?;
        let alias = body.try_next()?.ok_or(ERR)?;
        let from = body.try_next()?.ok_or(ERR)?;
        let to = body.try_next()?.ok_or(ERR)?;

        value_is!(alias, IncomingValue::String(alias));
        value_is!(from, IncomingValue::String(_));
        value_is!(to, IncomingValue::String(to));

        if to.is_empty() {
            Ok(Some(alias))
        } else {
            Ok(None)
        }
    }
}
