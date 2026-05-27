use crate::{EncodeError, const_helpers::get_range_mut};

pub(crate) struct Rule;

impl Rule {
    pub(crate) const fn fmt(
        buf: &mut [u8],
        sender: Option<&str>,
        interface: Option<&str>,
        path: Option<&str>,
        member: Option<&str>,
    ) -> Result<usize, EncodeError> {
        let mut offset = 0;

        macro_rules! push {
            ($s:expr) => {{
                let buflen = buf.len();
                let Some(rest) = get_range_mut(buf, offset, buflen) else {
                    return Err(EncodeError::BufferTooSmall);
                };
                let bytes_pushed = match push(rest, $s) {
                    Ok(len) => len,
                    Err(err) => return Err(err),
                };
                if let Some(new_offset) = offset.checked_add(bytes_pushed) {
                    offset = new_offset;
                } else {
                    return Err(EncodeError::ValueTooLong);
                }
            }};
        }

        push!("type='signal'");

        if let Some(sender) = sender {
            push!(",sender='");
            push!(sender);
            push!("'");
        }

        if let Some(interface) = interface {
            push!(",interface='");
            push!(interface);
            push!("'");
        }

        if let Some(path) = path {
            push!(",path='");
            push!(path);
            push!("'");
        }

        if let Some(member) = member {
            push!(",member='");
            push!(member);
            push!("'");
        }

        Ok(offset)
    }
}

const fn push(buf: &mut [u8], s: &str) -> Result<usize, EncodeError> {
    let Some(slice) = get_range_mut(buf, 0, s.len()) else {
        return Err(EncodeError::BufferTooSmall);
    };
    slice.copy_from_slice(s.as_bytes());
    Ok(s.len())
}

#[cfg(test)]
mod tests {
    use super::Rule;

    #[test]
    fn test_fmt_rule() {
        let mut buf = [0; 1_024];
        let len = Rule::fmt(&mut buf, Some("SENDER"), None, Some("PATH"), Some("MEMBER")).unwrap();
        let rule = core::str::from_utf8(&buf[..len]).unwrap();
        assert_eq!(
            rule,
            "type='signal',sender='SENDER',path='PATH',member='MEMBER'"
        );
    }
}
