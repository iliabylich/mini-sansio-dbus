// #![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::type_complexity)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_qualifications)]
#![warn(deprecated_in_future)]
#![warn(unused_lifetimes)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::arithmetic_side_effects)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![expect(clippy::redundant_pub_crate)]
#![expect(clippy::cast_possible_truncation)]
#![expect(clippy::arithmetic_side_effects)]
#![doc = include_str!("../README.md")]

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
/// A module with many known message types
pub mod messages;
pub use error::DBusError;
pub use outgoing::{OutgoingCompleteType, OutgoingMessage, OutgoingSignature, OutgoingValue};
pub use requests::{IncompleteMethodCall, MethodCall, Subscription};
pub use sansio::{DBusConnection, DBusQueue};
pub use satisfy::DBusSatisfy;
pub use types::MessageType;
pub use wants::DBusWants;
