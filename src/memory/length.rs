//! Exact object lengths. A symbolic term is the access bound; its proved
//! upper bound is only for placement/scalability and finite byte loops.
#[derive(Clone, Debug)]
pub enum Length {
    Concrete(u64),
    Symbolic { term: String, upper: u64 },
}
impl From<u64> for Length {
    fn from(size: u64) -> Self {
        Self::Concrete(size)
    }
}
impl Length {
    pub fn term(&self) -> String {
        match self {
            Self::Concrete(n) => crate::value::bv(u128::from(*n), 64),
            Self::Symbolic { term, .. } => term.clone(),
        }
    }
    pub fn upper(&self) -> u64 {
        match self {
            Self::Concrete(n) => *n,
            Self::Symbolic { upper, .. } => *upper,
        }
    }
    pub fn concrete(&self) -> Option<u64> {
        match self {
            Self::Concrete(n) => Some(*n),
            _ => None,
        }
    }
    pub fn min(&self, other: &Self) -> Self {
        match (self.concrete(), other.concrete()) {
            (Some(a), Some(b)) => Self::Concrete(a.min(b)),
            _ => Self::Symbolic {
                term: format!(
                    "(ite (bvule {} {}) {} {})",
                    self.term(),
                    other.term(),
                    self.term(),
                    other.term()
                ),
                upper: self.upper().min(other.upper()),
            },
        }
    }
}
