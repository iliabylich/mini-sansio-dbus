use crate::{
    DBusError, DBusSerial, IncomingBody, IncomingMessage, interface_is,
    messages::org_freedesktop_dbus::{AddMatch, RemoveMatch},
    sansio::OutgoingQueue,
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
    fn unsubscribe<Q, B>(
        &mut self,
        serial: &mut DBusSerial,
        queue: &mut Q,
        buf: B,
    ) -> Result<(), DBusError>
    where
        Q: OutgoingQueue,
        B: AsMut<[u8]>,
    {
        let SubscriptionState::Subscribed(path) = core::mem::take(&mut self.state) else {
            return Ok(());
        };

        let rule = format!(
            "type='signal',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{path}'",
        );
        encode_and_queue(serial, queue, buf, &RemoveMatch::new_from_rule(&rule))?;
        Ok(())
    }

    fn subscribe<Q, B>(
        &mut self,
        sender: &str,
        path: String,
        serial: &mut DBusSerial,
        queue: &mut Q,
        buf: B,
    ) -> Result<(), DBusError>
    where
        Q: OutgoingQueue,
        B: AsMut<[u8]>,
    {
        let rule = format!(
            "type='signal',sender='{sender}',interface='org.freedesktop.DBus.Properties',member='PropertiesChanged',path='{path}'",
        );
        encode_and_queue(serial, queue, buf, &AddMatch::new_from_rule(&rule))?;
        self.state = SubscriptionState::Subscribed(path);
        Ok(())
    }

    /// Sends a "subscribe" request
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails or the queue rejects a message.
    pub fn start<Q, B>(
        &mut self,
        sender: impl AsRef<str>,
        path: impl Into<String>,
        serial: &mut DBusSerial,
        queue: &mut Q,
        unsubscribe_buf: B,
        subscribe_buf: B,
    ) -> Result<(), DBusError>
    where
        Q: OutgoingQueue,
        B: AsMut<[u8]>,
    {
        self.unsubscribe(serial, queue, unsubscribe_buf)?;
        self.subscribe(sender.as_ref(), path.into(), serial, queue, subscribe_buf)
    }

    /// Unsubscribes and resets internal state
    ///
    /// # Errors
    ///
    /// Returns an error if encoding fails or the queue rejects the unsubscribe message.
    pub fn reset<Q, B>(
        &mut self,
        serial: &mut DBusSerial,
        queue: &mut Q,
        buf: B,
    ) -> Result<(), DBusError>
    where
        Q: OutgoingQueue,
        B: AsMut<[u8]>,
    {
        self.unsubscribe(serial, queue, buf)
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

fn encode_and_queue<Q, B, M>(
    serial: &mut DBusSerial,
    queue: &mut Q,
    mut buf: B,
    message: &M,
) -> Result<u32, DBusError>
where
    Q: OutgoingQueue,
    B: AsMut<[u8]>,
    M: crate::EncodeMessage,
{
    let next_serial = serial.current();
    let len = message.encode_message(buf.as_mut())?;
    let message = buf
        .as_mut()
        .get_mut(..len)
        .ok_or(DBusError::InternalError)?;
    queue.push(message, next_serial)?;
    serial.advance();
    Ok(next_serial)
}
