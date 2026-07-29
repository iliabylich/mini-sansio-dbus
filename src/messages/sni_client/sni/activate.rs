use crate::{IncomingMessage, MessageType};

/// Represents "Activate" event that is sent from SNI host
pub struct StatusNotifierActivateEvent;

impl StatusNotifierActivateEvent {
    /// Returns true if given message matches "Activate" message
    #[must_use]
    pub fn handle(message: IncomingMessage<'_>) -> bool {
        message.message_type == MessageType::MethodCall
            && message.path == Some("/StatusNotifierItem")
            && matches!(
                message.interface,
                Some("org.kde.StatusNotifierItem" | "org.freedesktop.StatusNotifierItem")
            )
            && message.member == Some("Activate")
    }
}
