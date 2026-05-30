/// Helpers to build reply handlers.
pub mod reply_handler;

/// Helpers to subscribe, unsubscribe, and handle **static** property changes
/// (at fixed, known at compile-time destination, path, and interface)
pub mod static_property;

/// Helpers to subscribe, unsubscribe, and handle **dynamic** property changes
/// (at unknown at compile-time destination, path, and interface)
pub mod dynamic_property;

mod statically_encoded_message;
pub use statically_encoded_message::StaticallyEncodedMessage;

mod dbus_send;
pub use dbus_send::DBusSend;
