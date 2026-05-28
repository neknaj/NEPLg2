extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, storage_size_bytes};
use crate::types::{TypeCtx, TypeId};

use super::super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::super::collection_slot_lifecycle::CollectionSlotLifecycleOp;
use super::super::collection_slot_summary_model::{
    CollectionSlotInitializedRangeDropTraversalCertificate,
    CollectionSlotInitializedRangeDropTraversalProof,
    CollectionSlotLifecycleSummaryDropTraversalCoverage, CollectionSlotLifecycleSummaryI32Operand,
    CollectionSlotLifecycleSummaryOp,
};
use super::super::model::{PlaceProjection, ResourceFunction, ResourceOffset};
use super::super::place_utils::projection_result_type;
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

/// 完全な leaf-only `DropTraversal + ForallInitializedRange` summary entry。
///
/// 個別 value の `Vec` では、同じ value が複数回現れる場合に multiplicity を失い、
/// replay 時に元の summary op 列を復元できない。この entry は function summary の
/// top-level op 列としての順序と重複をそのまま保存し、将来の fixed-point skip が
///「この関数 summary 全体を再現できる」ことを確認するための単位にする。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResourceSummaryStableDropTraversalForallLeafEntry {
    leaves: Vec<ResourceSummaryStableDropTraversalForallValue>,
}

impl ResourceSummaryStableDropTraversalForallLeafEntry {
    pub(super) fn len(&self) -> usize {
        self.leaves.len()
    }
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

/// Stable summary value を現在の compile session の `TypeId` へ戻すための境界。
///
/// Resource summary value cache の長寿命 value には arena slot である `TypeId` を
/// 保存しない。この context は現在の `ResourceFunction` signature と
/// function-local type parameter boundary から、stable type key と現在の `TypeId` の
/// 対応を再構築する。対応が一意に決まらない場合は cache replay 側が miss として
/// 現行の summary build に戻れるよう、構築時点で `None` を返す。
pub(super) struct ResourceSummaryTypeReprojection<'a> {
    types: &'a TypeCtx,
    function: &'a ResourceFunction,
    type_map: Vec<(ResourceSummaryStableTypeKey, TypeId)>,
}

impl<'a> ResourceSummaryTypeReprojection<'a> {
    pub(super) fn new(
        types: &'a TypeCtx,
        function: &'a ResourceFunction,
        type_params: &[TypeId],
    ) -> Option<Self> {
        let mut out = Self {
            types,
            function,
            type_map: Vec::new(),
        };
        out.insert_required_type(types.unit())?;
        out.insert_required_type(types.i32())?;
        out.insert_required_type(types.u8())?;
        out.insert_required_type(types.f32())?;
        out.insert_required_type(types.bool())?;
        out.insert_required_type(types.char())?;
        out.insert_required_type(types.str())?;
        out.insert_required_type(types.never())?;
        out.insert_type(function.result)?;
        for param in &function.params {
            out.insert_type(param.ty)?;
        }
        for ty in &function.type_params {
            out.insert_required_type(*ty)?;
        }
        for ty in type_params {
            out.insert_required_type(*ty)?;
        }
        Some(out)
    }

    fn insert_type(&mut self, ty: TypeId) -> Option<()> {
        let Some(key) = ResourceSummaryStableTypeKey::from_type(self.types, ty) else {
            return Some(());
        };
        self.insert_type_key(ty, key)
    }

    fn insert_required_type(&mut self, ty: TypeId) -> Option<()> {
        let key = ResourceSummaryStableTypeKey::from_type(self.types, ty)?;
        self.insert_type_key(ty, key)
    }

    fn insert_type_key(&mut self, ty: TypeId, key: ResourceSummaryStableTypeKey) -> Option<()> {
        let resolved = self.types.resolve_id(ty);
        match self
            .type_map
            .iter()
            .find(|(existing_key, _)| existing_key == &key)
        {
            Some((_, existing_ty)) if self.types.resolve_id(*existing_ty) != resolved => None,
            Some(_) => Some(()),
            None => {
                self.type_map.push((key, resolved));
                Some(())
            }
        }
    }

    fn reproject_type(&self, key: &ResourceSummaryStableTypeKey) -> Option<TypeId> {
        self.type_map
            .iter()
            .find(|(existing_key, _)| existing_key == key)
            .map(|(_, ty)| *ty)
    }
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

pub(super) fn stable_drop_traversal_forall_leaf_entry(
    types: &TypeCtx,
    ops: &[CollectionSlotLifecycleSummaryOp],
) -> Option<ResourceSummaryStableDropTraversalForallLeafEntry> {
    if ops.is_empty() {
        return None;
    }
    let leaves = ops
        .iter()
        .map(|op| stable_drop_traversal_forall_value(types, op))
        .collect::<Option<Vec<_>>>()?;
    Some(ResourceSummaryStableDropTraversalForallLeafEntry { leaves })
}

pub(super) fn reproject_drop_traversal_forall_value(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    value: &ResourceSummaryStableDropTraversalForallValue,
) -> Option<CollectionSlotLifecycleSummaryOp> {
    let storage = reproject_summary_place(ctx, &value.storage)?;
    let initialized_count = reproject_i32_operand(ctx, &value.initialized_count)?;
    let expected_ty = ctx.reproject_type(&value.expected_ty)?;
    if value.element_stride != storage_size_bytes(ctx.types, expected_ty) {
        return None;
    }
    Some(CollectionSlotLifecycleSummaryOp::DropTraversal {
        storage,
        initialized_count,
        expected_ty,
        coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
            CollectionSlotInitializedRangeDropTraversalCertificate {
                element_stride: value.element_stride,
                drop_proof: reproject_drop_traversal_proof(ctx, &value.drop_proof)?,
            },
        ),
    })
}

pub(super) fn reproject_drop_traversal_forall_leaf_entry(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    entry: &ResourceSummaryStableDropTraversalForallLeafEntry,
) -> Option<Vec<CollectionSlotLifecycleSummaryOp>> {
    if entry.leaves.is_empty() {
        return None;
    }
    entry
        .leaves
        .iter()
        .map(|value| reproject_drop_traversal_forall_value(ctx, value))
        .collect()
}

fn reproject_i32_operand(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    operand: &ResourceSummaryStableI32Operand,
) -> Option<CollectionSlotLifecycleSummaryI32Operand> {
    match operand {
        ResourceSummaryStableI32Operand::Place(place) => {
            let place = reproject_summary_place(ctx, place)?;
            (ctx.types.resolve_id(place.ty) == ctx.types.i32())
                .then_some(CollectionSlotLifecycleSummaryI32Operand::Place(place))
        }
        ResourceSummaryStableI32Operand::KnownI32 { value, ty } => {
            let ty = ctx.reproject_type(ty)?;
            (ctx.types.resolve_id(ty) == ctx.types.i32())
                .then_some(CollectionSlotLifecycleSummaryI32Operand::KnownI32 { value: *value, ty })
        }
    }
}

fn reproject_drop_traversal_proof(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    proof: &ResourceSummaryStableDropTraversalProof,
) -> Option<CollectionSlotInitializedRangeDropTraversalProof> {
    match proof {
        ResourceSummaryStableDropTraversalProof::StateOnly => {
            Some(CollectionSlotInitializedRangeDropTraversalProof::StateOnly)
        }
        ResourceSummaryStableDropTraversalProof::LoadedValueDrop(obligation) => Some(
            CollectionSlotInitializedRangeDropTraversalProof::LoadedValueDrop(
                reproject_drop_obligation(ctx, obligation)?,
            ),
        ),
    }
}

fn reproject_drop_obligation(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    obligation: &ResourceSummaryStableDropObligation,
) -> Option<CollectionSlotDropObligation> {
    Some(CollectionSlotDropObligation::DropLoadedValue {
        operation: reproject_lifecycle_op(obligation.operation),
        value_ty: ctx.reproject_type(&obligation.value_ty)?,
    })
}

fn reproject_lifecycle_op(
    operation: ResourceSummaryStableLifecycleOp,
) -> CollectionSlotLifecycleOp {
    match operation {
        ResourceSummaryStableLifecycleOp::InitializeEmpty => {
            CollectionSlotLifecycleOp::InitializeEmpty
        }
        ResourceSummaryStableLifecycleOp::BorrowRead => CollectionSlotLifecycleOp::BorrowRead,
        ResourceSummaryStableLifecycleOp::MoveOut => CollectionSlotLifecycleOp::MoveOut,
        ResourceSummaryStableLifecycleOp::ReplaceInitialized => {
            CollectionSlotLifecycleOp::ReplaceInitialized
        }
        ResourceSummaryStableLifecycleOp::DropInitialized => {
            CollectionSlotLifecycleOp::DropInitialized
        }
        ResourceSummaryStableLifecycleOp::DropTraversal => CollectionSlotLifecycleOp::DropTraversal,
        ResourceSummaryStableLifecycleOp::StorageDealloc => {
            CollectionSlotLifecycleOp::StorageDealloc
        }
        ResourceSummaryStableLifecycleOp::StorageRelocate => {
            CollectionSlotLifecycleOp::StorageRelocate
        }
        ResourceSummaryStableLifecycleOp::ValueTransfer => CollectionSlotLifecycleOp::ValueTransfer,
    }
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

fn reproject_summary_place(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    place: &ResourceSummaryStablePlace,
) -> Option<SummaryPlace> {
    let base = ctx.function.params.get(place.parameter_index)?.ty;
    let mut suffix = Vec::new();
    let mut current_ty = base;
    for stable_projection in &place.suffix {
        let projection = reproject_summary_projection(ctx, stable_projection)?;
        validate_projection_layout(ctx.types, current_ty, &projection)?;
        current_ty = summary_projection_result_type(ctx.types, current_ty, &projection)?;
        suffix.push(projection);
    }
    if !place.ty.matches_type(ctx.types, current_ty) {
        return None;
    }
    Some(SummaryPlace {
        parameter_index: place.parameter_index,
        suffix,
        ty: current_ty,
    })
}

fn reproject_summary_projection(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    projection: &ResourceSummaryStableProjection,
) -> Option<SummaryProjection> {
    Some(match projection {
        ResourceSummaryStableProjection::Field {
            index,
            offset_bytes,
        } => SummaryProjection::Field {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        ResourceSummaryStableProjection::TupleField {
            index,
            offset_bytes,
        } => SummaryProjection::TupleField {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        ResourceSummaryStableProjection::EnumPayload { variant } => {
            SummaryProjection::EnumPayload {
                variant: variant.clone(),
            }
        }
        ResourceSummaryStableProjection::Deref => SummaryProjection::Deref,
        ResourceSummaryStableProjection::StorageOffset(offset) => {
            SummaryProjection::StorageOffset(reproject_summary_offset(ctx, offset)?)
        }
    })
}

fn reproject_summary_offset(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    offset: &ResourceSummaryStableOffset,
) -> Option<SummaryOffset> {
    Some(match offset {
        ResourceSummaryStableOffset::Known(value) => SummaryOffset::Known(*value),
        ResourceSummaryStableOffset::Symbolic { place } => SummaryOffset::Symbolic {
            place: Box::new(reproject_summary_place(ctx, place)?),
        },
        ResourceSummaryStableOffset::ScaledSymbolic { place, scale } => {
            SummaryOffset::ScaledSymbolic {
                place: Box::new(reproject_summary_place(ctx, place)?),
                scale: *scale,
            }
        }
        ResourceSummaryStableOffset::Offset { place, offset } => SummaryOffset::Offset {
            place: Box::new(reproject_summary_place(ctx, place)?),
            offset: *offset,
        },
        ResourceSummaryStableOffset::ScaledOffset {
            place,
            offset,
            scale,
        } => SummaryOffset::ScaledOffset {
            place: Box::new(reproject_summary_place(ctx, place)?),
            offset: *offset,
            scale: *scale,
        },
    })
}

fn validate_projection_layout(
    types: &TypeCtx,
    base_ty: TypeId,
    projection: &SummaryProjection,
) -> Option<()> {
    match projection {
        SummaryProjection::Field {
            index,
            offset_bytes,
        }
        | SummaryProjection::TupleField {
            index,
            offset_bytes,
        } => {
            let field = aggregate_fields_with_offsets(types, base_ty)
                .get(*index)
                .copied()?;
            (field.offset == *offset_bytes).then_some(())
        }
        SummaryProjection::EnumPayload { .. }
        | SummaryProjection::Deref
        | SummaryProjection::StorageOffset(_) => {
            summary_projection_result_type(types, base_ty, projection).map(|_| ())
        }
    }
}

fn summary_projection_result_type(
    types: &TypeCtx,
    base_ty: TypeId,
    projection: &SummaryProjection,
) -> Option<TypeId> {
    match projection {
        SummaryProjection::Field {
            index,
            offset_bytes,
        } => projection_result_type(
            types,
            base_ty,
            &PlaceProjection::Field {
                index: *index,
                offset_bytes: *offset_bytes,
            },
        ),
        SummaryProjection::TupleField {
            index,
            offset_bytes,
        } => projection_result_type(
            types,
            base_ty,
            &PlaceProjection::TupleField {
                index: *index,
                offset_bytes: *offset_bytes,
            },
        ),
        SummaryProjection::EnumPayload { variant } => projection_result_type(
            types,
            base_ty,
            &PlaceProjection::EnumPayload {
                variant: variant.clone(),
            },
        ),
        SummaryProjection::Deref => projection_result_type(types, base_ty, &PlaceProjection::Deref),
        SummaryProjection::StorageOffset(_) => projection_result_type(
            types,
            base_ty,
            &PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        ),
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::ast::Effect;
    use crate::span::Span;
    use crate::types::TypeCtx;

    use super::super::super::collection_slot_summary_model::{
        CollectionSlotInitializedRangeDropTraversalCertificate,
        CollectionSlotInitializedRangeDropTraversalProof,
    };
    use super::super::super::model::{Place, ResourceBlockId, ResourceFunction, ResourceLocal};
    use super::*;

    fn function_with_param(param_ty: crate::types::TypeId) -> ResourceFunction {
        ResourceFunction {
            name: "summary_cache_subject".to_string(),
            origin_name: "summary_cache_subject".to_string(),
            type_params: Vec::new(),
            params: vec![ResourceLocal {
                name: "storage".to_string(),
                ty: param_ty,
                mutable: false,
                place: Place::local("storage".to_string(), param_ty),
            }],
            result: param_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: Vec::new(),
            span: Span::dummy(),
        }
    }

    fn function_with_type_param(type_param: crate::types::TypeId) -> ResourceFunction {
        let mut function = function_with_param(type_param);
        function.type_params.push(type_param);
        function
    }

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

    #[test]
    fn stable_drop_traversal_forall_value_reprojects_state_only_certificate() {
        let types = TypeCtx::new();
        let function = function_with_param(types.i32());
        let op = CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: SummaryPlace {
                parameter_index: 0,
                suffix: Vec::new(),
                ty: types.i32(),
            },
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: 3,
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
            .expect("state-only forall drop traversal should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_drop_traversal_forall_value(&ctx, &value)
            .expect("stable state-only value should reproject");

        assert_eq!(reprojected, op);
    }

    #[test]
    fn stable_drop_traversal_forall_value_reprojects_loaded_value_drop_certificate() {
        let types = TypeCtx::new();
        let function = function_with_param(types.i32());
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
                    drop_proof: CollectionSlotInitializedRangeDropTraversalProof::LoadedValueDrop(
                        CollectionSlotDropObligation::DropLoadedValue {
                            operation: CollectionSlotLifecycleOp::DropInitialized,
                            value_ty: types.i32(),
                        },
                    ),
                },
            ),
        };
        let value = stable_drop_traversal_forall_value(&types, &op)
            .expect("loaded-value certificate should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_drop_traversal_forall_value(&ctx, &value)
            .expect("loaded-value certificate should reproject");

        assert_eq!(reprojected, op);
    }

    #[test]
    fn stable_drop_traversal_forall_value_reprojects_symbolic_storage_offset() {
        let types = TypeCtx::new();
        let function = function_with_param(types.i32());
        let op = CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: SummaryPlace {
                parameter_index: 0,
                suffix: vec![SummaryProjection::StorageOffset(
                    SummaryOffset::ScaledOffset {
                        place: Box::new(SummaryPlace {
                            parameter_index: 0,
                            suffix: Vec::new(),
                            ty: types.i32(),
                        }),
                        offset: 2,
                        scale: 4,
                    },
                )],
                ty: types.i32(),
            },
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::Place(SummaryPlace {
                parameter_index: 0,
                suffix: Vec::new(),
                ty: types.i32(),
            }),
            expected_ty: types.i32(),
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                CollectionSlotInitializedRangeDropTraversalCertificate {
                    element_stride: 4,
                    drop_proof: CollectionSlotInitializedRangeDropTraversalProof::StateOnly,
                },
            ),
        };
        let value = stable_drop_traversal_forall_value(&types, &op)
            .expect("symbolic storage offset should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_drop_traversal_forall_value(&ctx, &value)
            .expect("symbolic storage offset should reproject");

        assert_eq!(reprojected, op);
    }

    #[test]
    fn stable_drop_traversal_forall_leaf_entry_preserves_order_and_duplicates() {
        let types = TypeCtx::new();
        let function = function_with_param(types.i32());
        let first = CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: SummaryPlace {
                parameter_index: 0,
                suffix: Vec::new(),
                ty: types.i32(),
            },
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: 2,
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
        let second = CollectionSlotLifecycleSummaryOp::DropTraversal {
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
        let ops = vec![first.clone(), second.clone(), first.clone()];
        let entry = stable_drop_traversal_forall_leaf_entry(&types, &ops)
            .expect("complete leaf entry should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_drop_traversal_forall_leaf_entry(&ctx, &entry)
            .expect("complete leaf entry should reproject");

        assert_eq!(entry.len(), 3);
        assert_eq!(reprojected, ops);
    }

    #[test]
    fn stable_drop_traversal_forall_value_rejects_out_of_range_parameter_on_reprojection() {
        let types = TypeCtx::new();
        let storing_function = function_with_param(types.i32());
        let restore_function = ResourceFunction {
            params: Vec::new(),
            ..function_with_param(types.i32())
        };
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
            .expect("storing function should produce a stable value");
        let _ = storing_function;
        let ctx = ResourceSummaryTypeReprojection::new(&types, &restore_function, &[])
            .expect("restore boundary itself should be reprojectable");

        assert!(reproject_drop_traversal_forall_value(&ctx, &value).is_none());
    }

    #[test]
    fn stable_drop_traversal_forall_value_rejects_stride_mismatch_on_reprojection() {
        let types = TypeCtx::new();
        let function = function_with_param(types.i32());
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
        let mut value = stable_drop_traversal_forall_value(&types, &op)
            .expect("state-only forall drop traversal should convert");
        value.element_stride = 8;
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        assert!(reproject_drop_traversal_forall_value(&ctx, &value).is_none());
    }

    #[test]
    fn stable_drop_traversal_forall_value_rejects_projection_layout_mismatch() {
        let mut types = TypeCtx::new();
        let tuple_ty = types.tuple(vec![types.i32(), types.u8()]);
        let function = function_with_param(tuple_ty);
        let op = CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: SummaryPlace {
                parameter_index: 0,
                suffix: vec![SummaryProjection::TupleField {
                    index: 1,
                    offset_bytes: 4,
                }],
                ty: types.u8(),
            },
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: 1,
                ty: types.i32(),
            },
            expected_ty: types.u8(),
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                CollectionSlotInitializedRangeDropTraversalCertificate {
                    element_stride: 1,
                    drop_proof: CollectionSlotInitializedRangeDropTraversalProof::StateOnly,
                },
            ),
        };
        let mut value = stable_drop_traversal_forall_value(&types, &op)
            .expect("tuple field drop traversal should convert");
        value.storage.suffix[0] = ResourceSummaryStableProjection::TupleField {
            index: 1,
            offset_bytes: 8,
        };
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("tuple function boundary should be reprojectable");

        assert!(reproject_drop_traversal_forall_value(&ctx, &value).is_none());
    }

    #[test]
    fn stable_drop_traversal_forall_value_reprojects_labelled_generic_boundary() {
        let mut storing_types = TypeCtx::new();
        let storing_generic = storing_types.fresh_var(Some("T".to_string()));
        let storing_function = function_with_type_param(storing_generic);
        let op = CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: SummaryPlace {
                parameter_index: 0,
                suffix: Vec::new(),
                ty: storing_generic,
            },
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: 1,
                ty: storing_types.i32(),
            },
            expected_ty: storing_generic,
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                CollectionSlotInitializedRangeDropTraversalCertificate {
                    element_stride: 4,
                    drop_proof: CollectionSlotInitializedRangeDropTraversalProof::StateOnly,
                },
            ),
        };
        let value = stable_drop_traversal_forall_value(&storing_types, &op)
            .expect("labelled generic boundary should convert");
        let mut restoring_types = TypeCtx::new();
        let restoring_generic = restoring_types.fresh_var(Some("T".to_string()));
        let restoring_function = function_with_type_param(restoring_generic);
        let ctx = ResourceSummaryTypeReprojection::new(
            &restoring_types,
            &restoring_function,
            &[restoring_generic],
        )
        .expect("same labelled generic boundary should reproject");

        let reprojected = reproject_drop_traversal_forall_value(&ctx, &value)
            .expect("generic stable value should reproject");

        assert_eq!(
            reprojected,
            CollectionSlotLifecycleSummaryOp::DropTraversal {
                storage: SummaryPlace {
                    parameter_index: 0,
                    suffix: Vec::new(),
                    ty: restoring_generic,
                },
                initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                    value: 1,
                    ty: restoring_types.i32(),
                },
                expected_ty: restoring_generic,
                coverage:
                    CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                        CollectionSlotInitializedRangeDropTraversalCertificate {
                            element_stride: 4,
                            drop_proof: CollectionSlotInitializedRangeDropTraversalProof::StateOnly,
                        },
                    ),
            }
        );
        let _ = storing_function;
    }

    #[test]
    fn stable_drop_traversal_forall_value_rejects_ambiguous_generic_boundary() {
        let mut types = TypeCtx::new();
        let first = types.fresh_var(Some("T".to_string()));
        let second = types.fresh_var(Some("T".to_string()));
        let function = function_with_type_param(first);

        assert!(
            ResourceSummaryTypeReprojection::new(&types, &function, &[first, second]).is_none()
        );
    }
}
