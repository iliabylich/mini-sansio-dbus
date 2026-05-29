macro_rules! try_ {
    ($e:expr) => {
        match $e {
            Ok(value) => value,
            Err(err) => return Err(err),
        }
    };
}
pub(crate) use try_;

pub(crate) const fn get_range_mut(buf: &mut [u8], start: usize, end: usize) -> Option<&mut [u8]> {
    let Some((_, tail)) = buf.split_at_mut_checked(start) else {
        return None;
    };
    let Some(offset) = end.checked_sub(start) else {
        return None;
    };
    let Some((head, _)) = tail.split_at_mut_checked(offset) else {
        return None;
    };
    Some(head)
}

pub(crate) const fn get_range(buf: &[u8], start: usize, end: usize) -> Option<&[u8]> {
    let Some((_, tail)) = buf.split_at_checked(start) else {
        return None;
    };
    let Some(offset) = end.checked_sub(start) else {
        return None;
    };
    let Some((head, _)) = tail.split_at_checked(offset) else {
        return None;
    };
    Some(head)
}

pub(crate) const fn u32_from_usize(v: usize) -> Option<u32> {
    let bytes = v.to_le_bytes();
    let Some((b0, tail)) = bytes.split_first() else {
        return None;
    };
    let Some((b1, tail)) = tail.split_first() else {
        return None;
    };
    let Some((b2, tail)) = tail.split_first() else {
        return None;
    };
    let Some((b3, tail)) = tail.split_first() else {
        return None;
    };
    if !bytes_are_zero(tail) {
        return None;
    }
    Some(u32::from_le_bytes([*b0, *b1, *b2, *b3]))
}

pub(crate) const fn u8_from_usize(v: usize) -> Option<u8> {
    let bytes = v.to_le_bytes();
    let Some((byte, tail)) = bytes.split_first() else {
        return None;
    };
    if !bytes_are_zero(tail) {
        return None;
    }
    Some(*byte)
}

const fn bytes_are_zero(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        let Some((_, tail)) = bytes.split_at_checked(i) else {
            return false;
        };
        let Some((byte, _)) = tail.split_first() else {
            return false;
        };
        if *byte != 0 {
            return false;
        }
        i = match i.checked_add(1) {
            Some(i) => i,
            None => return false,
        };
    }
    true
}

pub(crate) struct ConstMessage<const CAP: usize> {
    buf: [u8; CAP],
    len: usize,
}

impl<const CAP: usize> ConstMessage<CAP> {
    pub(crate) const fn new() -> Self {
        Self {
            buf: [0; CAP],
            len: 0,
        }
    }

    pub(crate) const fn push_str(mut self, value: &str) -> Option<Self> {
        let bytes = value.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let Some((_, tail)) = bytes.split_at_checked(i) else {
                return None;
            };
            let Some((byte, _)) = tail.split_first() else {
                return None;
            };
            let Some(next) = self.push_byte(*byte) else {
                return None;
            };
            self = next;
            i = match i.checked_add(1) {
                Some(i) => i,
                None => return None,
            };
        }
        Some(self)
    }

    pub(crate) const fn push_usize(mut self, value: usize) -> Option<Self> {
        let Some(digits) = usize_decimal_digits(value) else {
            return None;
        };
        let Some(end) = self.len.checked_add(digits) else {
            return None;
        };
        if end > CAP {
            return None;
        }

        let mut pos = end;
        let mut rem = value;

        loop {
            pos = match pos.checked_sub(1) {
                Some(pos) => pos,
                None => return None,
            };
            let Some(digit) = u8_from_usize(rem % 10) else {
                return None;
            };
            let Some(slot) = get_array_mut(&mut self.buf, pos) else {
                return None;
            };
            *slot = match b'0'.checked_add(digit) {
                Some(byte) => byte,
                None => return None,
            };
            rem /= 10;
            if pos == self.len {
                break;
            }
        }

        self.len = end;
        Some(self)
    }

    pub(crate) const fn as_str(&self) -> Option<&str> {
        let Some((bytes, _)) = self.buf.split_at_checked(self.len) else {
            return None;
        };
        match core::str::from_utf8(bytes) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    const fn push_byte(mut self, value: u8) -> Option<Self> {
        if self.len == CAP {
            return None;
        }
        let Some(slot) = get_array_mut(&mut self.buf, self.len) else {
            return None;
        };
        *slot = value;
        self.len = match self.len.checked_add(1) {
            Some(len) => len,
            None => return None,
        };
        Some(self)
    }
}

const fn usize_decimal_digits(mut value: usize) -> Option<usize> {
    let mut digits: usize = 1;
    while value >= 10 {
        value /= 10;
        digits = match digits.checked_add(1) {
            Some(digits) => digits,
            None => return None,
        };
    }
    Some(digits)
}

const fn get_array_mut<const LEN: usize>(buf: &mut [u8; LEN], index: usize) -> Option<&mut u8> {
    let Some((_, tail)) = buf.split_at_mut_checked(index) else {
        return None;
    };
    let Some((slot, _)) = tail.split_first_mut() else {
        return None;
    };
    Some(slot)
}

/// Constructs a static message encoded at compile time
#[macro_export]
macro_rules! def_constant_message {
    (name = $name:ident, size = $size:expr, |$var:ident| => $eval:expr) => {
        /// A static message
        pub struct $name;
        impl $name {
            /// Size of the encoded message.
            pub const SIZE: usize = $size;

            const fn encode($var: &mut [u8]) -> Result<usize, $crate::EncodeError> {
                $eval
            }

            /// Actual encoded byte sequence.
            pub const ENCODED: [u8; Self::SIZE] = {
                let mut buf = [0; Self::SIZE];
                let len = match Self::encode(&mut buf) {
                    Ok(len) => len,
                    Err(err) => panic!("{}", err.display()),
                };
                if len != Self::SIZE {
                    $crate::panic_size_mismatch_message(Self::SIZE, len);
                }
                buf
            };

            /// Sends a static message to a given queue, without processing reply
            pub fn send<'q, Q>(q: &mut Q) -> u32
            where
                Q: $crate::OutgoingQueue<'q>,
            {
                q.push(&Self::ENCODED)
            }


        }
    };

    (name = $name:ident, size = $size:expr, with-reply, |$var:ident| => $eval:expr) => {
        $crate::def_constant_message!(name = $name, size = $size, |$var| => $eval);

        impl $name {
            pub fn send_and_prepare_for_reply<'q, Q, E>(
                q: &mut Q,
                e: E,
            ) -> $crate::messaging::reply_handler::ReplyHandler<Self, E>
            where
                Self: $crate::messaging::reply_handler::HasReplyHandler,
                Q: $crate::OutgoingQueue<'q>,
                E: $crate::messaging::reply_handler::ReplyErrorHandler,
            {
                let serial = Self::send(q);
                ReplyHandler::new(serial, Self, e)
            }
        }
    };
}

#[doc(hidden)]
#[expect(clippy::panic)]
pub const fn panic_size_mismatch_message(declared: usize, got: usize) -> ! {
    let message = ConstMessage::<96>::new();
    let Some(message) = message.push_str("buffer is too long, can be just ") else {
        panic!("failed to format buffer length error");
    };
    let Some(message) = message.push_usize(got) else {
        panic!("failed to format buffer length error");
    };
    let Some(message) = message.push_str(" bytes, not ") else {
        panic!("failed to format buffer length error");
    };
    let Some(message) = message.push_usize(declared) else {
        panic!("failed to format buffer length error");
    };
    let Some(message) = message.as_str() else {
        panic!("failed to format buffer length error");
    };
    panic!("{}", message)
}
