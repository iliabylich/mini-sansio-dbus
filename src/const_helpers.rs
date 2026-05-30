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
