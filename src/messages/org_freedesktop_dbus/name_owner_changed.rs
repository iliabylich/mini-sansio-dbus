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
            (
                Some("org.freedesktop.DBus"),
                Some("/org/freedesktop/DBus"),
                Some("org.freedesktop.DBus"),
                Some("NameOwnerChanged"),
            ),
            buf,
        )
    }
}

/// Unsubscribes from `NameOwnerChanged` signal
pub struct NameOwnerChangedUnsubscribe;

impl DBusEncode for NameOwnerChangedUnsubscribe {
    type Args<'a> = ();

    fn encode<'a>((): Self::Args<'_>, buf: &'a mut [u8]) -> Result<&'a [u8], EncodeError> {
        Unsubscribe::encode(
            (
                Some("org.freedesktop.DBus"),
                Some("/org/freedesktop/DBus"),
                Some("org.freedesktop.DBus"),
                Some("NameOwnerChanged"),
            ),
            buf,
        )
    }
}

/// Ownership transition reported by a `NameOwnerChanged` signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameOwnerChangedSignal<'a> {
    /// A well-known name got an owner.
    Appeared {
        /// Well-known bus name.
        name: &'a str,
        /// New unique owner name.
        owner: &'a str,
    },
    /// A well-known name lost its owner.
    Disappeared {
        /// Well-known bus name.
        name: &'a str,
        /// Previous unique owner name.
        owner: &'a str,
    },
    /// A well-known name changed from one owner to another.
    Changed {
        /// Well-known bus name.
        name: &'a str,
        /// Previous unique owner name.
        old_owner: &'a str,
        /// New unique owner name.
        new_owner: &'a str,
    },
}

impl<'a> NameOwnerChangedSignal<'a> {
    /// Parses `messages` and returns the ownership transition if it represents a
    /// `NameOwnerChanged` signal.
    ///
    /// # Errors
    ///
    /// Returns an error if the message is malformed.
    pub fn handle(message: IncomingMessage<'a>) -> Result<Option<Self>, DBusError> {
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
        value_is!(from, IncomingValue::String(from));
        value_is!(to, IncomingValue::String(to));

        if from.is_empty() && !to.is_empty() {
            Ok(Some(Self::Appeared {
                name: alias,
                owner: to,
            }))
        } else if !from.is_empty() && to.is_empty() {
            Ok(Some(Self::Disappeared {
                name: alias,
                owner: from,
            }))
        } else if !from.is_empty() && !to.is_empty() {
            Ok(Some(Self::Changed {
                name: alias,
                old_owner: from,
                new_owner: to,
            }))
        } else {
            Ok(None)
        }
    }

    /// Returns `name` field of the signal that is present in each variant
    #[must_use]
    pub const fn name(&self) -> &str {
        match self {
            NameOwnerChangedSignal::Appeared { name, .. }
            | NameOwnerChangedSignal::Disappeared { name, .. }
            | NameOwnerChangedSignal::Changed { name, .. } => name,
        }
    }
}
