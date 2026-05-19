extern crate alloc;

use alloc::boxed::Box;

use super::model::{Place, PlaceRoot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RawAddressOffset {
    Known(i64),
    Symbolic { place: Box<Place> },
    SymbolicPlusKnown { place: Box<Place>, bytes: i64 },
    Unknown,
}

impl RawAddressOffset {
    pub(super) fn symbolic(place: &Place) -> Self {
        if matches!(place.root, PlaceRoot::Unknown) {
            RawAddressOffset::Unknown
        } else {
            RawAddressOffset::Symbolic {
                place: Box::new(place.clone()),
            }
        }
    }

    pub(super) fn add(self, rhs: RawAddressOffset) -> Self {
        match (self, rhs) {
            (offset, RawAddressOffset::Known(0)) => offset,
            (RawAddressOffset::Known(0), offset) => offset,
            (RawAddressOffset::Known(lhs), RawAddressOffset::Known(rhs)) => lhs
                .checked_add(rhs)
                .map(RawAddressOffset::Known)
                .unwrap_or(RawAddressOffset::Unknown),
            (RawAddressOffset::Symbolic { place }, RawAddressOffset::Known(bytes))
            | (RawAddressOffset::Known(bytes), RawAddressOffset::Symbolic { place }) => {
                RawAddressOffset::SymbolicPlusKnown { place, bytes }
            }
            (
                RawAddressOffset::SymbolicPlusKnown { place, bytes },
                RawAddressOffset::Known(rhs),
            )
            | (
                RawAddressOffset::Known(rhs),
                RawAddressOffset::SymbolicPlusKnown { place, bytes },
            ) => bytes
                .checked_add(rhs)
                .map(|bytes| RawAddressOffset::SymbolicPlusKnown { place, bytes })
                .unwrap_or(RawAddressOffset::Unknown),
            _ => RawAddressOffset::Unknown,
        }
    }

    pub(super) fn sub(self, rhs: RawAddressOffset) -> Self {
        match (self, rhs) {
            (offset, RawAddressOffset::Known(0)) => offset,
            (RawAddressOffset::Known(lhs), RawAddressOffset::Known(rhs)) => lhs
                .checked_sub(rhs)
                .map(RawAddressOffset::Known)
                .unwrap_or(RawAddressOffset::Unknown),
            (RawAddressOffset::Symbolic { place }, RawAddressOffset::Known(bytes)) => bytes
                .checked_neg()
                .map(|bytes| RawAddressOffset::SymbolicPlusKnown { place, bytes })
                .unwrap_or(RawAddressOffset::Unknown),
            (
                RawAddressOffset::SymbolicPlusKnown { place, bytes },
                RawAddressOffset::Known(rhs),
            ) => bytes
                .checked_sub(rhs)
                .map(|bytes| RawAddressOffset::SymbolicPlusKnown { place, bytes })
                .unwrap_or(RawAddressOffset::Unknown),
            _ => RawAddressOffset::Unknown,
        }
    }
}
