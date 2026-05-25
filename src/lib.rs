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
#![warn(clippy::panic)]
#![warn(clippy::arithmetic_side_effects)]
#![warn(clippy::indexing_slicing)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![expect(clippy::redundant_pub_crate)]
#![doc = include_str!("../README.md")]

mod body_macro;
mod const_helpers;
mod encoder;
mod error;
mod incoming;
mod introspectible_object_at;
mod sansio;
mod satisfy;
#[cfg(test)]
mod tests;
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
pub use encoder::{EncodeError, MessageEncoder as SliceMessageEncoder};
pub use error::DBusError;
pub use sansio::{DBusConnection, DBusSerial, EncodeMessage, OutgoingQueue};
pub use satisfy::DBusSatisfy;
pub use types::MessageType;
pub use wants::DBusWants;
