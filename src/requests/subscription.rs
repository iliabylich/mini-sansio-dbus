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

    fn subscribe(&mut self, sender: String, path: String, queue: &mut DBusQueue) {
        let message = AddMatch::build(sender, path.clone());
        queue.push_back(message);
        self.state = SubscriptionState::Subscribed(path);
    }

    pub fn start(
        &mut self,
        sender: impl Into<String>,
        path: impl Into<String>,
        queue: &mut DBusQueue,
    ) {
        self.unsubscribe(queue);
        self.subscribe(sender.into(), path.into(), queue);
    }

    pub fn reset(&mut self, queue: &mut DBusQueue) {
        self.unsubscribe(queue)
    }

    fn try_process(&self, message: IncomingMessage<'_>) -> Result<T, Box<dyn core::error::Error>> {
        if message.message_type != MessageType::Signal {
            return Err(DBusError::WrongMessageType(format!(
                "expected: {:?}, got: {:?}",
                MessageType::Signal,
                message.message_type
            ))
            .into());
        }

        let interface = message.interface.ok_or(DBusError::NoInterface)?;
        interface_is!(interface, "org.freedesktop.DBus.Properties");
        let path = message.path.ok_or(DBusError::NoPath)?;
        let body = message.body.ok_or(DBusError::NoBody)?;

        let SubscriptionState::Subscribed(subscribed_to) = self.state.clone() else {
            return Err(DBusError::InternalError("not subscribed".to_string()).into());
        };

        (self.try_process)(body, path.to_string(), subscribed_to)
    }

    pub fn process(&self, message: IncomingMessage<'_>) -> Option<T> {
        self.try_process(message).ok()
    }

    pub const fn new(
        try_process: &'static dyn Fn(
            IncomingBody<'_>,
            String,
            String,
        ) -> Result<T, Box<dyn core::error::Error>>,
    ) -> Self {
        Subscription {
            try_process,
            state: SubscriptionState::None,
        }
    }
}
