#![allow(clippy::type_complexity)]

mod error;
mod incoming;
mod introspectible_object_at;
mod outgoing;
mod requests;
mod sansio;
mod satisfy;
mod types;
mod wants;

pub use incoming::{
    IncomingArrayValue, IncomingArrayValueIter, IncomingBody, IncomingDictEntryValue,
    IncomingMessage, IncomingStructValue, IncomingStructValueIter, IncomingValue,
    IncomingVariantValue,
};
pub use introspectible_object_at::{IntrospectibleObjectAt, IntrospectibleObjectAtRequest};
pub mod messages;
pub use error::DBusError;
pub use outgoing::{OutgoingCompleteType, OutgoingMessage, OutgoingSignature, OutgoingValue};
pub use requests::{MethodCall, Subscription};
pub use sansio::{DBusConnection, DBusQueue};
pub use satisfy::Satisfy;
pub use types::MessageType;
pub use wants::Wants;
