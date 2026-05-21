use crate::{
    DBusError, IncomingBody, IncomingMessage, interface_is,
    messages::org_freedesktop_dbus::{AddMatch, RemoveMatch},
    sansio::DBusQueue,
    types::MessageType,
};

#[derive(Default, Clone)]
enum SubscriptionState {
    #[default]
    None,
    Subscribed(String),
}

/// A helper to subscribe to a `PropertiesChanged` signal and process a stream of incoming updates
#[must_use]
pub struct Subscription<T>
where
    T: 'static,
{
    try_process:
        &'static dyn Fn(IncomingBody<'_>, String, String) -> Result<T, Box<dyn core::error::Error>>,
    state: SubscriptionState,
}

impl<T> Subscription<T> {
    fn unsubscribe(&mut self, queue: &mut DBusQueue) {
        let SubscriptionState::Subscribed(path) = core::mem::take(&mut self.state) else {
            return;
        };

        let message = RemoveMatch::build(path);
        queue.push_back(message);
    }

    fn subscribe(&mut self, sender: &str, path: String, queue: &mut DBusQueue) {
        let message = AddMatch::build(sender, &path);
        queue.push_back(message);
        self.state = SubscriptionState::Subscribed(path);
    }

    /// Sends a "subscribe" request
    pub fn start(
        &mut self,
        sender: impl AsRef<str>,
        path: impl Into<String>,
        queue: &mut DBusQueue,
    ) {
        self.unsubscribe(queue);
        self.subscribe(sender.as_ref(), path.into(), queue);
    }

    /// Unsubscribes and resets internal state
    pub fn reset(&mut self, queue: &mut DBusQueue) {
        self.unsubscribe(queue);
    }

    fn try_process(&self, message: IncomingMessage<'_>) -> Result<T, Box<dyn core::error::Error>> {
        if message.message_type != MessageType::Signal {
            return Err(DBusError::WrongMessageType.into());
        }

        let interface = message.interface.ok_or(DBusError::NoInterface)?;
        interface_is!(interface, "org.freedesktop.DBus.Properties");
        let path = message.path.ok_or(DBusError::NoPath)?;
        let body = message.body.ok_or(DBusError::NoBody)?;

        let SubscriptionState::Subscribed(subscribed_to) = self.state.clone() else {
            return Err(DBusError::InternalError.into());
        };

        (self.try_process)(body, path.to_string(), subscribed_to)
    }

    /// Processes incoming message, return whatever is returned from a given `try_process` function.
    #[must_use]
    pub fn process(&self, message: IncomingMessage<'_>) -> Option<T> {
        self.try_process(message).ok()
    }

    /// A builder patter:
    ///
    /// ```ignore
    /// let sub = Subscription::new(|body, path, subscribed_to| { /* process message and return anything */ 42 })
    /// sub.start()
    /// loop {
    ///     if let Some(output) = sub.try_process(stream.read_message()) {
    ///         assert_eq!(output, 42)
    ///     }
    /// }
    /// ```
    pub const fn new(
        try_process: &'static dyn Fn(
            IncomingBody<'_>,
            String,
            String,
        ) -> Result<T, Box<dyn core::error::Error>>,
    ) -> Self {
        Self {
            try_process,
            state: SubscriptionState::None,
        }
    }
}
