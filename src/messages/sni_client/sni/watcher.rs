use crate::{DBusError, IncomingMessage, messages::org_freedesktop_dbus::NameOwnerChangedSignal};

/// SNI watcher ownership event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusNotifierWatcher<'a> {
    /// The watcher appeared
    Appeared {
        /// New unique owner name
        owner: &'a str,
    },
    /// The watcher disappeared
    Disappeared {
        /// Previous unique owner name
        owner: &'a str,
    },
}

impl<'a> StatusNotifierWatcher<'a> {
    /// Parses a message and returns an SNI watcher event if it matches
    ///
    /// # Errors
    ///
    /// Returns an error if the message is malformed
    pub fn handle(message: IncomingMessage<'a>) -> Result<Option<Self>, DBusError> {
        use NameOwnerChangedSignal::{Appeared, Changed, Disappeared};

        let Some(name_owner_changed) = NameOwnerChangedSignal::handle(message)? else {
            return Ok(None);
        };

        if name_owner_changed.name() != "org.kde.StatusNotifierWatcher" {
            return Ok(None);
        }

        let event = match name_owner_changed {
            Appeared { owner, .. } => Self::Appeared { owner },
            Disappeared { owner, .. } => Self::Disappeared { owner },
            Changed { new_owner, .. } => Self::Appeared { owner: new_owner },
        };

        Ok(Some(event))
    }
}
