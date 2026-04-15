use crate::OutgoingCompleteType;

#[derive(PartialEq, Eq)]
pub struct OutgoingSignature {
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
