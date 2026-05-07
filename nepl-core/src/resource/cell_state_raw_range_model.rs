use crate::types::TypeId;

use super::model::Place;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InitializedRawByteRange {
    pub(super) address: Place,
    pub(super) count: Place,
    pub(super) unit: InitializedRawRangeUnit,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InitializedRawRangeUnit {
    Bytes,
    Elements { stride: usize },
}

impl InitializedRawByteRange {
    pub(super) fn address(&self) -> &Place {
        &self.address
    }

    pub(super) fn count(&self) -> &Place {
        &self.count
    }

    pub(super) fn unit(&self) -> InitializedRawRangeUnit {
        self.unit
    }

    pub(super) fn ty(&self) -> TypeId {
        self.ty
    }
}
