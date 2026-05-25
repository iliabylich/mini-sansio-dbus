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

pub(crate) const fn u32_from_usize(v: usize) -> Option<u32> {
    if v > u32::MAX as usize {
        return None;
    }
    #[expect(clippy::cast_possible_truncation)]
    Some(v as u32)
}

pub(crate) const fn u8_from_usize(v: usize) -> Option<u8> {
    if v > u8::MAX as usize {
        return None;
    }
    #[expect(clippy::cast_possible_truncation)]
    Some(v as u8)
}
