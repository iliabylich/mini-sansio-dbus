use crate::const_helpers::{get_at_mut, t_opt, u8_from_usize};

/// A very basic compile-time formatter, only supports `usize` and `&str`
#[must_use]
pub struct ConstFormatter<const CAP: usize> {
    buf: [u8; CAP],
    len: usize,
}

impl<const CAP: usize> ConstFormatter<CAP> {
    /// Constructor
    #[expect(clippy::new_without_default)]
    pub const fn new() -> Self {
        Self {
            buf: [0; CAP],
            len: 0,
        }
    }

    const fn try_push_str(&mut self, value: &str) -> Option<()> {
        let bytes = value.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let (_, tail) = t_opt!(bytes.split_at_checked(i));
            let (byte, _) = t_opt!(tail.split_first());
            t_opt!(self.push_byte(*byte));
            i = t_opt!(i.checked_add(1));
        }
        Some(())
    }

    /// Pushes given string, panics if configured capacity is exceeded
    ///
    /// # Panics
    ///
    /// Panics on internal buffer overflow
    pub const fn push_str(&mut self, value: &str) {
        assert!(
            self.try_push_str(value).is_some(),
            "failed to format buffer length error"
        );
    }

    const fn try_push_usize(&mut self, value: usize) -> Option<()> {
        let digits = t_opt!(usize_decimal_digits(value));
        let end = t_opt!(self.len.checked_add(digits));
        if end > CAP {
            return None;
        }

        let mut pos = end;
        let mut rem = value;

        loop {
            pos = t_opt!(pos.checked_sub(1));
            let digit = t_opt!(u8_from_usize(rem % 10));
            let slot = t_opt!(get_at_mut(&mut self.buf, pos));
            *slot = t_opt!(b'0'.checked_add(digit));
            rem /= 10;
            if pos == self.len {
                break;
            }
        }

        self.len = end;
        Some(())
    }

    /// Pushes given usize, panics if configured capacity is exceeded
    ///
    /// # Panics
    ///
    /// Panics on internal buffer overflow
    pub const fn push_usize(&mut self, value: usize) {
        assert!(
            self.try_push_usize(value).is_some(),
            "failed to format buffer length error"
        );
    }

    const fn try_as_str(&self) -> Option<&str> {
        let (bytes, _) = t_opt!(self.buf.split_at_checked(self.len));
        match core::str::from_utf8(bytes) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    /// Converts all pushed content to string, panics if it's not a valid UTF-8 byte sequence.
    ///
    /// # Panics
    ///
    /// Panics if pushed data doesn't form a valid UTF-8 string
    #[must_use]
    #[expect(clippy::panic)]
    pub const fn as_str(&self) -> &str {
        match self.try_as_str() {
            Some(s) => s,
            None => {
                panic!("failed to format buffer length error")
            }
        }
    }

    const fn push_byte(&mut self, value: u8) -> Option<()> {
        if self.len == CAP {
            return None;
        }
        let slot = t_opt!(get_at_mut(&mut self.buf, self.len));
        *slot = value;
        self.len = t_opt!(self.len.checked_add(1));
        Some(())
    }
}

const fn usize_decimal_digits(mut value: usize) -> Option<usize> {
    let mut digits: usize = 1;
    while value >= 10 {
        value /= 10;
        digits = t_opt!(digits.checked_add(1));
    }
    Some(digits)
}

#[cfg(test)]
mod tests {
    use super::ConstFormatter;

    #[test]
    fn test_fmt() {
        let mut message = ConstFormatter::<96>::new();
        message.push_str("foo ");
        message.push_usize(42);
        assert_eq!(message.as_str(), "foo 42");
    }
}
