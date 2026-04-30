use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeId;

use super::model::{Place, PlaceProjection, ResourceOffset, ResourceOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawAddressSource {
    pub(super) base: Place,
    pub(super) offset: RawAddressOffset,
    pub(super) explicit_offset: bool,
}

pub(super) struct RawAddressPlace {
    pub(super) place: Place,
    pub(super) is_view: bool,
}

impl RawAddressSource {
    pub(super) fn into_place_and_view(self, raw_ty: TypeId) -> RawAddressPlace {
        let is_view = self.explicit_offset;
        let place = match self.offset {
            RawAddressOffset::Known(0) if !self.explicit_offset => self.base,
            RawAddressOffset::Known(0) => self.base.with_projection(
                PlaceProjection::StorageOffset(ResourceOffset { bytes: Some(0) }),
                raw_ty,
            ),
            RawAddressOffset::Known(bytes) if bytes > 0 => match usize::try_from(bytes) {
                Ok(bytes) => self.base.with_projection(
                    PlaceProjection::StorageOffset(ResourceOffset { bytes: Some(bytes) }),
                    raw_ty,
                ),
                Err(_) => self.base.with_projection(
                    PlaceProjection::StorageOffset(ResourceOffset { bytes: None }),
                    raw_ty,
                ),
            },
            RawAddressOffset::Known(_) | RawAddressOffset::Unknown => self.base.with_projection(
                PlaceProjection::StorageOffset(ResourceOffset { bytes: None }),
                raw_ty,
            ),
        };
        RawAddressPlace { place, is_view }
    }

    pub(super) fn with_added_offset(mut self, offset: Option<i64>) -> Self {
        self.offset = self.offset.add(offset);
        self.explicit_offset = true;
        self
    }

    pub(super) fn with_subtracted_offset(mut self, offset: Option<i64>) -> Self {
        self.offset = self.offset.sub(offset);
        self.explicit_offset = true;
        self
    }
}

pub(super) fn push_raw_address_op(
    source: Place,
    target: Place,
    is_view: bool,
    ops: &mut Vec<ResourceOp>,
    span: Span,
) {
    if is_view {
        ops.push(ResourceOp::RawAddressView {
            source,
            target,
            span,
        });
    } else {
        ops.push(ResourceOp::RawAddressAlias {
            source,
            target,
            span,
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawAddressOffset {
    Known(i64),
    Unknown,
}

impl RawAddressOffset {
    fn add(self, rhs: Option<i64>) -> Self {
        match (self, rhs) {
            (RawAddressOffset::Known(lhs), Some(rhs)) => lhs
                .checked_add(rhs)
                .map(RawAddressOffset::Known)
                .unwrap_or(RawAddressOffset::Unknown),
            _ => RawAddressOffset::Unknown,
        }
    }

    fn sub(self, rhs: Option<i64>) -> Self {
        match (self, rhs) {
            (RawAddressOffset::Known(lhs), Some(rhs)) => lhs
                .checked_sub(rhs)
                .map(RawAddressOffset::Known)
                .unwrap_or(RawAddressOffset::Unknown),
            _ => RawAddressOffset::Unknown,
        }
    }
}
