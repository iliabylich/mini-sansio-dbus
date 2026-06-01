use crate::EncodeError;

pub(crate) struct Rule;

impl Rule {
    pub(crate) fn fmt(
        buf: &mut [u8],
        sender: Option<&str>,
        interface: Option<&str>,
        path: Option<&str>,
        member: Option<&str>,
    ) -> Result<usize, EncodeError> {
        let mut offset = 0;

        macro_rules! push {
            ($s:expr) => {{
                let rest = buf.get_mut(offset..).ok_or(EncodeError::BufferTooSmall)?;
                let bytes_pushed = push(rest, $s)?;
                offset = offset
                    .checked_add(bytes_pushed)
                    .ok_or(EncodeError::ValueTooLong)?
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

fn push(buf: &mut [u8], s: &str) -> Result<usize, EncodeError> {
    let slice = buf.get_mut(0..s.len()).ok_or(EncodeError::BufferTooSmall)?;
    slice.copy_from_slice(s.as_bytes());
    Ok(s.len())
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::indexing_slicing)]
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
