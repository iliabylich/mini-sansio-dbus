mod cursor;
mod error;
mod message;

pub use error::EncodeError;
pub use message::MessageEncoder;

type EncodeResult<T> = Result<T, EncodeError>;
