use crate::{OutgoingQueue, messaging::DBusSend};

/// A common trait for all statically encoded DBus messages.
pub trait StaticallyEncodedMessage {
    /// Byte representation
    const ENCODED: &'static [u8];

    // /// Sends a static message to a given queue, without processing reply
    // fn send<'q, Q>(q: &mut Q) -> u32
    // where
    //     Q: OutgoingQueue<'q>,
    // {
    //     q.push(&Self::ENCODED)
    // }
}

impl<T> DBusSend for T
where
    T: StaticallyEncodedMessage,
{
    type Error = core::convert::Infallible;

    fn send<'q, Q>(q: &mut Q) -> Result<u32, Self::Error>
    where
        Q: OutgoingQueue<'q>,
    {
        let serial = q.push(Self::ENCODED);
        Ok(serial)
    }
}

/// A helper macro to encode a message
#[macro_export]
macro_rules! encode_message {
    ($size:expr, |$var:ident| => $eval:expr) => {{
        let mut buf = [0; $size];
        let $var = &mut buf;
        let len = match $eval {
            Ok(len) => len,
            Err(err) => panic!("{}", err.display()),
        };
        if len != $size {
            let mut fmt = $crate::ConstFormatter::<96>::new();
            fmt.push_str("buffer is too long, can be just ");
            fmt.push_usize(len);
            fmt.push_str(" bytes, not ");
            fmt.push_usize($size);
            panic!("{}", fmt.as_str())
        }
        buf
    }};
}
