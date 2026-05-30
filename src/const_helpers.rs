macro_rules! t_err {
    ($e:expr) => {
        match $e {
            Ok(value) => value,
            Err(err) => return Err(err),
        }
    };
}
pub(crate) use t_err;

macro_rules! t_opt {
    ($e:expr) => {
        match $e {
            Some(value) => value,
            None => return None,
        }
    };
}
pub(crate) use t_opt;

pub(crate) const fn get_range_mut(buf: &mut [u8], start: usize, end: usize) -> Option<&mut [u8]> {
    let (_, tail) = t_opt!(buf.split_at_mut_checked(start));
    let offset = t_opt!(end.checked_sub(start));
    let (head, _) = t_opt!(tail.split_at_mut_checked(offset));
    Some(head)
}

pub(crate) const fn get_range(buf: &[u8], start: usize, end: usize) -> Option<&[u8]> {
    let (_, tail) = t_opt!(buf.split_at_checked(start));
    let offset = t_opt!(end.checked_sub(start));
    let (head, _) = t_opt!(tail.split_at_checked(offset));
    Some(head)
}

pub(crate) const fn u32_from_usize(v: usize) -> Option<u32> {
    let bytes = v.to_le_bytes();
    let (b0, tail) = t_opt!(bytes.split_first());
    let (b1, tail) = t_opt!(tail.split_first());
    let (b2, tail) = t_opt!(tail.split_first());
    let (b3, tail) = t_opt!(tail.split_first());
    if !bytes_are_zero(tail) {
        return None;
    }
    Some(u32::from_le_bytes([*b0, *b1, *b2, *b3]))
}

pub(crate) const fn u8_from_usize(v: usize) -> Option<u8> {
    let bytes = v.to_le_bytes();
    let (byte, tail) = t_opt!(bytes.split_first());
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

pub(crate) const fn get_at_mut(buf: &mut [u8], index: usize) -> Option<&mut u8> {
    let (_, tail) = t_opt!(buf.split_at_mut_checked(index));
    let (slot, _) = t_opt!(tail.split_first_mut());
    Some(slot)
}

// /// Constructs a static message encoded at compile time
// #[macro_export]
// macro_rules! def_constant_message {
//     (name = $name:ident, size = $size:expr, |$var:ident| => $eval:expr) => {
//         /// A static message
//         pub struct $name;
//         impl $name {
//             const fn encode($var: &mut [u8]) -> Result<usize, $crate::EncodeError> {
//                 $eval
//             }

//             /// Actual encoded byte sequence.
//             pub const ENCODED: [u8; $size] = {
//                 let mut buf = [0; $size];
//                 let len = match Self::encode(&mut buf) {
//                     Ok(len) => len,
//                     Err(err) => panic!("{}", err.display()),
//                 };
//                 if len != $size {
//                     let mut fmt = $crate::ConstFormatter::<96>::new();
//                     fmt.push_str("buffer is too long, can be just ");
//                     fmt.push_usize(len);
//                     fmt.push_str(" bytes, not ");
//                     fmt.push_usize($size);
//                     panic!("{}", fmt.as_str())
//                 }
//                 buf
//             };

//             /// Sends a static message to a given queue, without processing reply
//             pub fn send<'q, Q>(q: &mut Q) -> u32
//             where
//                 Q: $crate::OutgoingQueue<'q>,
//             {
//                 q.push(&Self::ENCODED)
//             }

//         }
//     };

//     (name = $name:ident, size = $size:expr, with-reply, |$var:ident| => $eval:expr) => {
//         $crate::def_constant_message!(name = $name, size = $size, |$var| => $eval);

//         impl $name {
//             pub fn send_and_prepare_for_reply<'q, Q, E>(
//                 q: &mut Q,
//                 e: E,
//             ) -> $crate::messaging::reply_handler::ReplyHandler<Self, E>
//             where
//                 Self: $crate::messaging::reply_handler::HasReplyHandler,
//                 Q: $crate::OutgoingQueue<'q>,
//                 E: $crate::messaging::reply_handler::ReplyErrorHandler,
//             {
//                 let serial = Self::send(q);
//                 ReplyHandler::new(serial, Self, e)
//             }
//         }
//     };
// }
