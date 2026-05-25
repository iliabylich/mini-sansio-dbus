#![allow(
    clippy::missing_const_for_fn,
    clippy::missing_errors_doc,
    clippy::use_self
)]

mod cursor;
mod error;
mod message;
mod slot;
mod types;

pub use error::EncodeError;
pub use message::MessageEncoder;
pub use slot::{ArraySlot, DictEntrySlot, Slot, Struct2Slot, VariantSlot};
pub use types::{
    Array, DbusType, DictEntry, ObjectPath, Signature, Str, Struct2, UnixFd, Variant, WriteValue,
};

type EncodeResult<T> = Result<T, EncodeError>;
