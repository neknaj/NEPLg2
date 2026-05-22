extern crate alloc;

use super::model::{Place, PlaceProjection, ResourceOffset};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum NormalizedRawOffset {
    Known(usize),
    Symbolic {
        place: Place,
        known: usize,
    },
    ScaledSymbolic {
        place: Place,
        scale: usize,
        known: usize,
    },
}

impl NormalizedRawOffset {
    pub(super) fn from_suffix(suffix: &[PlaceProjection]) -> Option<Self> {
        let mut known = 0usize;
        let mut symbolic = None;
        let mut scaled = None;
        for projection in suffix {
            let PlaceProjection::StorageOffset(offset) = projection else {
                return None;
            };
            match offset {
                ResourceOffset::Known(bytes) => {
                    known = known.checked_add(*bytes)?;
                }
                ResourceOffset::Symbolic { place } => {
                    if symbolic.is_some() || scaled.is_some() {
                        return None;
                    }
                    symbolic = Some((**place).clone());
                }
                ResourceOffset::ScaledSymbolic { place, scale } => {
                    if symbolic.is_some() || scaled.is_some() {
                        return None;
                    }
                    scaled = Some(((**place).clone(), *scale));
                }
                ResourceOffset::Offset { place, offset } => {
                    if symbolic.is_some() || scaled.is_some() {
                        return None;
                    }
                    let offset = usize::try_from(*offset).ok()?;
                    known = known.checked_add(offset)?;
                    symbolic = Some((**place).clone());
                }
                ResourceOffset::ScaledOffset {
                    place,
                    offset,
                    scale,
                } => {
                    if symbolic.is_some() || scaled.is_some() {
                        return None;
                    }
                    let offset = usize::try_from(*offset).ok()?;
                    known = known.checked_add(offset)?;
                    scaled = Some(((**place).clone(), *scale));
                }
                ResourceOffset::Unknown => return None,
            }
        }
        if let Some(place) = symbolic {
            Some(Self::Symbolic { place, known })
        } else if let Some((place, scale)) = scaled {
            Some(Self::ScaledSymbolic {
                place,
                scale,
                known,
            })
        } else {
            Some(Self::Known(known))
        }
    }
}
