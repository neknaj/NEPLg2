extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::super::collection_slot_lifecycle::CollectionSlotLifecycleOp;
use super::super::collection_slot_summary_model::{
    CollectionSlotInitializedRangeDropTraversalProof,
    CollectionSlotLifecycleSummaryDropTraversalCoverage, CollectionSlotLifecycleSummaryI32Operand,
    CollectionSlotLifecycleSummaryOp,
};
use super::super::summary_projection::{SummaryOffset, SummaryPlace, SummaryProjection};
use super::stable_type_key::ResourceSummaryStableTypeKey;

/// Resource summary cache に保存できる `SummaryPlace` の mirror。
///
/// parameter index と projection は関数 signature に対する相対表現として保持し、
/// 型は `ResourceSummaryStableTypeKey` に変換する。これにより、cache hit 後に現在の
/// compile の Resource IR parameter / TypeCtx へ再投影する余地を残す。
#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStablePlace {
    parameter_index: usize,
    suffix: Vec<ResourceSummaryStableProjection>,
    ty: ResourceSummaryStableTypeKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceSummaryStableProjection {
    Field { index: usize, offset_bytes: usize },
    TupleField { index: usize, offset_bytes: usize },
    EnumPayload { variant: String },
    Deref,
    StorageOffset(ResourceSummaryStableOffset),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceSummaryStableOffset {
    Known(usize),
    Symbolic {
        place: Box<ResourceSummaryStablePlace>,
    },
    ScaledSymbolic {
        place: Box<ResourceSummaryStablePlace>,
        scale: usize,
    },
    Offset {
        place: Box<ResourceSummaryStablePlace>,
        offset: i64,
    },
    ScaledOffset {
        place: Box<ResourceSummaryStablePlace>,
        offset: i64,
        scale: usize,
    },
}

/// `DropTraversal + ForallInitializedRange` の stable mirror value。
///
/// これは初期 Resource summary value cache の最小保存単位である。現 checkpoint では
/// まだ cache map に保存しないが、bypass 計測はこの value へ変換できる候補だけを
/// 数える。変換できない場合は、`TypeId` など session-local な値が残っているため、
/// 後続の store/hit 実装でも保存対象にしてはならない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResourceSummaryStableDropTraversalForallValue {
    storage: ResourceSummaryStablePlace,
    initialized_count: ResourceSummaryStableI32Operand,
    expected_ty: ResourceSummaryStableTypeKey,
    element_stride: usize,
    drop_proof: ResourceSummaryStableDropTraversalProof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceSummaryStableI32Operand {
    Place(ResourceSummaryStablePlace),
    KnownI32 {
        value: i32,
        ty: ResourceSummaryStableTypeKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceSummaryStableDropTraversalProof {
    StateOnly,
    LoadedValueDrop(ResourceSummaryStableDropObligation),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableDropObligation {
    operation: ResourceSummaryStableLifecycleOp,
    value_ty: ResourceSummaryStableTypeKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceSummaryStableLifecycleOp {
    InitializeEmpty,
    BorrowRead,
    MoveOut,
    ReplaceInitialized,
    DropInitialized,
    DropTraversal,
    StorageDealloc,
    StorageRelocate,
    ValueTransfer,
}

pub(super) fn stable_drop_traversal_forall_value(
    types: &TypeCtx,
    op: &CollectionSlotLifecycleSummaryOp,
) -> Option<ResourceSummaryStableDropTraversalForallValue> {
    let CollectionSlotLifecycleSummaryOp::DropTraversal {
        storage,
        initialized_count,
        expected_ty,
        coverage:
            CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(certificate),
    } = op
    else {
        return None;
    };
    Some(ResourceSummaryStableDropTraversalForallValue {
        storage: stable_summary_place(types, storage)?,
        initialized_count: stable_i32_operand(types, initialized_count)?,
        expected_ty: ResourceSummaryStableTypeKey::from_type(types, *expected_ty)?,
        element_stride: certificate.element_stride,
        drop_proof: stable_drop_traversal_proof(types, certificate.drop_proof)?,
    })
}

fn stable_i32_operand(
    types: &TypeCtx,
    operand: &CollectionSlotLifecycleSummaryI32Operand,
) -> Option<ResourceSummaryStableI32Operand> {
    match operand {
        CollectionSlotLifecycleSummaryI32Operand::Place(place) => Some(
            ResourceSummaryStableI32Operand::Place(stable_summary_place(types, place)?),
        ),
        CollectionSlotLifecycleSummaryI32Operand::KnownI32 { value, ty } => {
            Some(ResourceSummaryStableI32Operand::KnownI32 {
                value: *value,
                ty: ResourceSummaryStableTypeKey::from_type(types, *ty)?,
            })
        }
    }
}

fn stable_drop_traversal_proof(
    types: &TypeCtx,
    proof: CollectionSlotInitializedRangeDropTraversalProof,
) -> Option<ResourceSummaryStableDropTraversalProof> {
    match proof {
        CollectionSlotInitializedRangeDropTraversalProof::StateOnly => {
            Some(ResourceSummaryStableDropTraversalProof::StateOnly)
        }
        CollectionSlotInitializedRangeDropTraversalProof::LoadedValueDrop(obligation) => {
            Some(ResourceSummaryStableDropTraversalProof::LoadedValueDrop(
                stable_drop_obligation(types, obligation)?,
            ))
        }
    }
}

fn stable_drop_obligation(
    types: &TypeCtx,
    obligation: CollectionSlotDropObligation,
) -> Option<ResourceSummaryStableDropObligation> {
    match obligation {
        CollectionSlotDropObligation::DropLoadedValue {
            operation,
            value_ty,
        } => Some(ResourceSummaryStableDropObligation {
            operation: stable_lifecycle_op(operation),
            value_ty: ResourceSummaryStableTypeKey::from_type(types, value_ty)?,
        }),
    }
}

fn stable_lifecycle_op(operation: CollectionSlotLifecycleOp) -> ResourceSummaryStableLifecycleOp {
    match operation {
        CollectionSlotLifecycleOp::InitializeEmpty => {
            ResourceSummaryStableLifecycleOp::InitializeEmpty
        }
        CollectionSlotLifecycleOp::BorrowRead => ResourceSummaryStableLifecycleOp::BorrowRead,
        CollectionSlotLifecycleOp::MoveOut => ResourceSummaryStableLifecycleOp::MoveOut,
        CollectionSlotLifecycleOp::ReplaceInitialized => {
            ResourceSummaryStableLifecycleOp::ReplaceInitialized
        }
        CollectionSlotLifecycleOp::DropInitialized => {
            ResourceSummaryStableLifecycleOp::DropInitialized
        }
        CollectionSlotLifecycleOp::DropTraversal => ResourceSummaryStableLifecycleOp::DropTraversal,
        CollectionSlotLifecycleOp::StorageDealloc => {
            ResourceSummaryStableLifecycleOp::StorageDealloc
        }
        CollectionSlotLifecycleOp::StorageRelocate => {
            ResourceSummaryStableLifecycleOp::StorageRelocate
        }
        CollectionSlotLifecycleOp::ValueTransfer => ResourceSummaryStableLifecycleOp::ValueTransfer,
    }
}

fn stable_summary_place(
    types: &TypeCtx,
    place: &SummaryPlace,
) -> Option<ResourceSummaryStablePlace> {
    Some(ResourceSummaryStablePlace {
        parameter_index: place.parameter_index,
        suffix: place
            .suffix
            .iter()
            .map(|projection| stable_summary_projection(types, projection))
            .collect::<Option<Vec<_>>>()?,
        ty: ResourceSummaryStableTypeKey::from_type(types, place.ty)?,
    })
}

fn stable_summary_projection(
    types: &TypeCtx,
    projection: &SummaryProjection,
) -> Option<ResourceSummaryStableProjection> {
    Some(match projection {
        SummaryProjection::Field {
            index,
            offset_bytes,
        } => ResourceSummaryStableProjection::Field {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        SummaryProjection::TupleField {
            index,
            offset_bytes,
        } => ResourceSummaryStableProjection::TupleField {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        SummaryProjection::EnumPayload { variant } => {
            ResourceSummaryStableProjection::EnumPayload {
                variant: variant.clone(),
            }
        }
        SummaryProjection::Deref => ResourceSummaryStableProjection::Deref,
        SummaryProjection::StorageOffset(offset) => {
            ResourceSummaryStableProjection::StorageOffset(stable_summary_offset(types, offset)?)
        }
    })
}

fn stable_summary_offset(
    types: &TypeCtx,
    offset: &SummaryOffset,
) -> Option<ResourceSummaryStableOffset> {
    Some(match offset {
        SummaryOffset::Known(value) => ResourceSummaryStableOffset::Known(*value),
        SummaryOffset::Symbolic { place } => ResourceSummaryStableOffset::Symbolic {
            place: Box::new(stable_summary_place(types, place)?),
        },
        SummaryOffset::ScaledSymbolic { place, scale } => {
            ResourceSummaryStableOffset::ScaledSymbolic {
                place: Box::new(stable_summary_place(types, place)?),
                scale: *scale,
            }
        }
        SummaryOffset::Offset { place, offset } => ResourceSummaryStableOffset::Offset {
            place: Box::new(stable_summary_place(types, place)?),
            offset: *offset,
        },
        SummaryOffset::ScaledOffset {
            place,
            offset,
            scale,
        } => ResourceSummaryStableOffset::ScaledOffset {
            place: Box::new(stable_summary_place(types, place)?),
            offset: *offset,
            scale: *scale,
        },
        SummaryOffset::Unknown => return None,
    })
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::types::TypeCtx;

    use super::super::super::collection_slot_summary_model::{
        CollectionSlotInitializedRangeDropTraversalCertificate,
        CollectionSlotInitializedRangeDropTraversalProof,
    };
    use super::*;

    #[test]
    fn stable_drop_traversal_forall_value_rejects_non_forall_coverage() {
        let types = TypeCtx::new();
        let op = CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: SummaryPlace {
                parameter_index: 0,
                suffix: Vec::new(),
                ty: types.i32(),
            },
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: 1,
                ty: types.i32(),
            },
            expected_ty: types.i32(),
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::CertifiedSlots(
                Vec::new(),
            ),
        };

        assert!(stable_drop_traversal_forall_value(&types, &op).is_none());
    }

    #[test]
    fn stable_drop_traversal_forall_value_rejects_unknown_offsets() {
        let types = TypeCtx::new();
        let op = CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: SummaryPlace {
                parameter_index: 0,
                suffix: vec![SummaryProjection::StorageOffset(SummaryOffset::Unknown)],
                ty: types.i32(),
            },
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: 1,
                ty: types.i32(),
            },
            expected_ty: types.i32(),
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                CollectionSlotInitializedRangeDropTraversalCertificate {
                    element_stride: 4,
                    drop_proof: CollectionSlotInitializedRangeDropTraversalProof::StateOnly,
                },
            ),
        };

        assert!(stable_drop_traversal_forall_value(&types, &op).is_none());
    }

    #[test]
    fn stable_drop_traversal_forall_value_converts_state_only_certificate() {
        let types = TypeCtx::new();
        let op = CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: SummaryPlace {
                parameter_index: 0,
                suffix: Vec::new(),
                ty: types.i32(),
            },
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: 1,
                ty: types.i32(),
            },
            expected_ty: types.i32(),
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                CollectionSlotInitializedRangeDropTraversalCertificate {
                    element_stride: 4,
                    drop_proof: CollectionSlotInitializedRangeDropTraversalProof::StateOnly,
                },
            ),
        };

        let value = stable_drop_traversal_forall_value(&types, &op)
            .expect("state-only forall drop traversal should convert to stable mirror");

        assert_eq!(value.expected_ty.as_str(), "i32");
        assert_eq!(value.element_stride, 4);
        assert_eq!(
            value.drop_proof,
            ResourceSummaryStableDropTraversalProof::StateOnly
        );
    }
}
