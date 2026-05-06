use crate::OutgoingCompleteType;

/// Signature of the value that is sent to `DBus`
#[derive(PartialEq, Eq)]
pub struct OutgoingSignature {
    /// a body is a list, so a signature is a list as well that is mapped to values 1-to-1
    pub items: Vec<OutgoingCompleteType>,
}

impl core::fmt::Debug for OutgoingSignature {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Signature(")?;
        let mut started = false;
        for item in &self.items {
            write!(f, "{}{:?}", if started { " -> " } else { "" }, item)?;
            started = true;
        }
        write!(f, ")")
    }
}
