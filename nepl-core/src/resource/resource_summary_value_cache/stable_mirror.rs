extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, storage_size_bytes};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::super::collection_slot_lifecycle::CollectionSlotLifecycleOp;
use super::super::collection_slot_summary_model::{
    CollectionSlotInitializedRangeDropTraversalCertificate,
    CollectionSlotInitializedRangeDropTraversalProof,
    CollectionSlotLifecycleSummaryDropTraversalCoverage, CollectionSlotLifecycleSummaryI32Operand,
    CollectionSlotLifecycleSummaryOp,
};
use super::super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellInitializationParamCell,
};
use super::super::initialized_summary_release_model::{
    RawCellReleaseParamRequirement, RawCellReleaseRequirementKind,
};
use super::super::model::{
    Place, PlaceProjection, ResourceFunction, ResourceLocal, ResourceOffset,
};
use super::super::place_utils::projection_result_type;
use super::super::summary_projection::{
    summary_place_for_params, SummaryOffset, SummaryPlace, SummaryProjection,
};
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
    Unknown,
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

/// raw initialization summary のうち、parameter に対する facts だけで完結する leaf entry。
///
/// `RawCellInitializationFunctionSummary` 全体には return facts、variant 条件、path-sensitive
/// release などが混在する。この entry は dependency-free 関数の param facts だけを
/// fixed-point worklist 前に再投影するための最小単位であり、`TypeId` は stable type key、
/// projection は layout を検証できる形式に落として保持する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResourceSummaryStableRawInitParamFactsLeafEntry {
    param_cells: Vec<ResourceSummaryStableRawInitParamCell>,
    param_release_requirements: Vec<ResourceSummaryStableRawCellReleaseParamRequirement>,
}

impl ResourceSummaryStableRawInitParamFactsLeafEntry {
    pub(super) fn len(&self) -> usize {
        self.param_cells.len() + self.param_release_requirements.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum ResourceSummaryStableRawInitParamFactsLeafEntryReject {
    Surface,
    ParamCellProjection,
    ParamCellType,
    ParamReleaseRequirementType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject {
    EmptyEntry,
    ParamCellProjection,
    ParamCellStableType,
    ParamCellResultType,
    ParamReleaseRequirementProjection,
    ParamReleaseRequirementType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableRawInitParamCell {
    param_index: usize,
    suffix: Vec<ResourceSummaryStableProjection>,
    ty: ResourceSummaryStableTypeKey,
    holds_raw_address: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableRawCellReleaseParamRequirement {
    param_index: usize,
    suffix: Vec<ResourceSummaryStablePlaceProjection>,
    ty: ResourceSummaryStableTypeKey,
    kind: ResourceSummaryStableRawCellReleaseRequirementKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceSummaryStablePlaceProjection {
    Field { index: usize, offset_bytes: usize },
    TupleField { index: usize, offset_bytes: usize },
    EnumPayload { variant: String },
    Deref,
    StorageOffset(ResourceSummaryStableOffset),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceSummaryStableRawCellReleaseRequirementKind {
    Store,
    Dealloc,
    Realloc,
    Fill,
    BulkDestination,
    BulkSource,
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
/// 対応を再構築する。呼び出し側は signature tree の奥にある open generic も含めた
/// owner summary boundary を渡す。対応が一意に決まらない場合は cache replay 側が
/// miss として現行の summary build に戻れるよう、構築時点で `None` を返す。
pub(super) struct ResourceSummaryTypeReprojection<'a> {
    types: &'a TypeCtx,
    function: &'a ResourceFunction,
    type_params: Vec<TypeId>,
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
            type_params: type_params.to_vec(),
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
        for ty in &function.type_params {
            out.insert_required_type(*ty)?;
        }
        for ty in type_params {
            out.insert_required_type(*ty)?;
        }
        out.insert_type(function.result)?;
        for param in &function.params {
            out.insert_type(param.ty)?;
        }
        Some(out)
    }

    fn insert_type(&mut self, ty: TypeId) -> Option<()> {
        let Some(key) = ResourceSummaryStableTypeKey::from_type(self.types, ty) else {
            return Some(());
        };
        self.insert_type_key_signature(ty, key)?;
        self.insert_type_children(ty, &mut BTreeSet::new())
    }

    fn insert_required_type(&mut self, ty: TypeId) -> Option<()> {
        let key = ResourceSummaryStableTypeKey::from_type(self.types, ty)?;
        self.insert_type_key_strict(ty, key)?;
        self.insert_type_children(ty, &mut BTreeSet::new())
    }

    fn insert_type_key_strict(
        &mut self,
        ty: TypeId,
        key: ResourceSummaryStableTypeKey,
    ) -> Option<()> {
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

    fn insert_type_key_signature(
        &mut self,
        ty: TypeId,
        key: ResourceSummaryStableTypeKey,
    ) -> Option<()> {
        let resolved = self.types.resolve_id(ty);
        match self
            .type_map
            .iter()
            .find(|(existing_key, _)| existing_key == &key)
        {
            Some((_, existing_ty))
                if self.types.resolve_id(*existing_ty) != resolved
                    && (self.type_is_open_generic(*existing_ty)
                        || self.type_is_open_generic(resolved)) =>
            {
                None
            }
            Some(_) => Some(()),
            None => {
                self.type_map.push((key, resolved));
                Some(())
            }
        }
    }

    fn insert_type_key_if_absent(&mut self, ty: TypeId, key: ResourceSummaryStableTypeKey) {
        let resolved = self.types.resolve_id(ty);
        if self
            .type_map
            .iter()
            .any(|(existing_key, _)| existing_key == &key)
        {
            return;
        }
        self.type_map.push((key, resolved));
    }

    fn type_is_open_generic(&self, ty: TypeId) -> bool {
        matches!(
            self.types.get_ref(self.types.resolve_id(ty)),
            TypeKind::Var(var) if var.binding.is_none()
        )
    }

    fn reproject_type(&self, key: &ResourceSummaryStableTypeKey) -> Option<TypeId> {
        if let Some(ty) = self
            .type_map
            .iter()
            .find(|(existing_key, _)| existing_key == key)
            .map(|(_, ty)| *ty)
        {
            return Some(ty);
        }
        self.reproject_type_from_current_type_context(key)
    }

    fn reproject_type_from_current_type_context(
        &self,
        key: &ResourceSummaryStableTypeKey,
    ) -> Option<TypeId> {
        // raw memory cell の値型は function signature に直接現れないことがある。
        // その型でも現在 session の TypeCtx に同じ stable key が存在するなら再投影できる。
        // ただし labelled open generic は同名衝突を stable key だけで解決できないため、
        // function/type-argument boundary で登録済みの場合にだけ利用する。
        if key.is_open_generic() {
            return None;
        }
        let mut found = None;
        for ty in self.types.type_ids() {
            let stable_key = ResourceSummaryStableTypeKey::from_type(self.types, ty)?;
            if &stable_key != key {
                continue;
            }
            let ty = self.types.resolve_id(ty);
            match found {
                Some(existing) if self.types.resolve_id(existing) != ty => {
                    if self.type_is_open_generic(existing) || self.type_is_open_generic(ty) {
                        return None;
                    }
                }
                Some(_) => {}
                None => found = Some(ty),
            }
        }
        found
    }

    fn insert_type_children(&mut self, ty: TypeId, seen: &mut BTreeSet<TypeId>) -> Option<()> {
        let resolved = self.types.resolve_named_type_id(ty);
        if !seen.insert(resolved) {
            return Some(());
        }
        let kind = self.types.get_ref(resolved).clone();
        match kind {
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Char
            | TypeKind::Str
            | TypeKind::Never
            | TypeKind::Named(_) => {}
            TypeKind::Var(var) => {
                if let Some(binding) = var.binding {
                    self.insert_type_tree_child(binding, seen)?;
                }
            }
            TypeKind::Enum {
                type_params,
                variants,
                ..
            } => {
                for type_param in type_params {
                    self.insert_type_tree_child(type_param, seen)?;
                }
                for variant in variants {
                    if let Some(payload) = variant.payload {
                        self.insert_type_tree_child(payload, seen)?;
                    }
                }
            }
            TypeKind::Struct {
                type_params,
                fields,
                ..
            } => {
                for type_param in type_params {
                    self.insert_type_tree_child(type_param, seen)?;
                }
                for field in fields {
                    self.insert_type_tree_child(field, seen)?;
                }
            }
            TypeKind::Tuple { items } => {
                for item in items {
                    self.insert_type_tree_child(item, seen)?;
                }
            }
            TypeKind::Function {
                type_params,
                params,
                result,
                ..
            } => {
                for type_param in type_params {
                    self.insert_type_tree_child(type_param, seen)?;
                }
                for param in params {
                    self.insert_type_tree_child(param, seen)?;
                }
                self.insert_type_tree_child(result, seen)?;
            }
            TypeKind::Apply { base, args } => {
                self.insert_type_tree_child(base, seen)?;
                for arg in args {
                    self.insert_type_tree_child(arg, seen)?;
                }
            }
            TypeKind::Box(inner) | TypeKind::Reference(inner, _) => {
                self.insert_type_tree_child(inner, seen)?;
            }
        }
        seen.remove(&resolved);
        Some(())
    }

    fn insert_type_tree_child(&mut self, ty: TypeId, seen: &mut BTreeSet<TypeId>) -> Option<()> {
        let key = ResourceSummaryStableTypeKey::from_type(self.types, ty)?;
        self.insert_type_key_if_absent(ty, key);
        self.insert_type_children(ty, seen)
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

pub(super) fn stable_raw_init_param_facts_leaf_entry(
    types: &TypeCtx,
    function: &ResourceFunction,
    summary: &RawCellInitializationFunctionSummary,
) -> Result<
    ResourceSummaryStableRawInitParamFactsLeafEntry,
    ResourceSummaryStableRawInitParamFactsLeafEntryReject,
> {
    if !raw_init_summary_is_param_facts_leaf(summary) {
        return Err(ResourceSummaryStableRawInitParamFactsLeafEntryReject::Surface);
    }
    let param_cells = summary
        .param_cells
        .iter()
        .map(|cell| stable_raw_init_param_cell(types, cell))
        .collect::<Result<Vec<_>, _>>()?;
    let param_release_requirements = summary
        .param_release_requirements
        .iter()
        .map(|requirement| {
            stable_raw_cell_release_param_requirement(types, &function.params, requirement)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ResourceSummaryStableRawInitParamFactsLeafEntry {
        param_cells,
        param_release_requirements,
    })
}

pub(super) fn reproject_raw_init_param_facts_leaf_entry(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    function_name: &str,
    entry: &ResourceSummaryStableRawInitParamFactsLeafEntry,
) -> Option<RawCellInitializationFunctionSummary> {
    reproject_raw_init_param_facts_leaf_entry_result(ctx, function_name, entry).ok()
}

/// stable raw-init param facts entry を現在の Resource IR summary に戻す。
///
/// この関数は cache 候補の自己再投影検査でも使うため、`Option` で失敗を潰さず、
/// projection の不一致と型 key の不一致を分けて返す。再投影できない entry は安全側で
/// store/replay しないが、失敗面を分けることで次の canonicalization 対象を測定できる。
pub(super) fn reproject_raw_init_param_facts_leaf_entry_result(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    function_name: &str,
    entry: &ResourceSummaryStableRawInitParamFactsLeafEntry,
) -> Result<
    RawCellInitializationFunctionSummary,
    ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject,
> {
    if entry.len() == 0 {
        return Err(ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::EmptyEntry);
    }
    Ok(RawCellInitializationFunctionSummary {
        function: function_name.to_string(),
        type_params: ctx.type_params.clone(),
        return_cells: Vec::new(),
        return_byte_ranges: Vec::new(),
        param_cells: entry
            .param_cells
            .iter()
            .map(|cell| reproject_raw_init_param_cell(ctx, cell))
            .collect::<Result<Vec<_>, _>>()?,
        param_byte_ranges: Vec::new(),
        param_release_requirements: entry
            .param_release_requirements
            .iter()
            .map(|requirement| reproject_raw_cell_release_param_requirement(ctx, requirement))
            .collect::<Result<Vec<_>, _>>()?,
        variant_param_cells: Vec::new(),
        variant_param_byte_ranges: Vec::new(),
        variant_required_param_cells: Vec::new(),
        variant_conditions: Vec::new(),
    })
}

fn raw_init_summary_is_param_facts_leaf(summary: &RawCellInitializationFunctionSummary) -> bool {
    (!summary.param_cells.is_empty() || !summary.param_release_requirements.is_empty())
        && summary.return_cells.is_empty()
        && summary.return_byte_ranges.is_empty()
        && summary.param_byte_ranges.is_empty()
        && summary.variant_param_cells.is_empty()
        && summary.variant_param_byte_ranges.is_empty()
        && summary.variant_required_param_cells.is_empty()
        && summary.variant_conditions.is_empty()
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

fn stable_raw_init_param_cell(
    types: &TypeCtx,
    cell: &RawCellInitializationParamCell,
) -> Result<
    ResourceSummaryStableRawInitParamCell,
    ResourceSummaryStableRawInitParamFactsLeafEntryReject,
> {
    let suffix = cell
        .suffix
        .iter()
        .map(|projection| {
            stable_summary_projection(types, projection)
                .ok_or(ResourceSummaryStableRawInitParamFactsLeafEntryReject::ParamCellProjection)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let ty = ResourceSummaryStableTypeKey::from_type(types, cell.ty)
        .ok_or(ResourceSummaryStableRawInitParamFactsLeafEntryReject::ParamCellType)?;
    Ok(ResourceSummaryStableRawInitParamCell {
        param_index: cell.param_index,
        suffix,
        ty,
        holds_raw_address: cell.holds_raw_address,
    })
}

fn stable_raw_cell_release_param_requirement(
    types: &TypeCtx,
    params: &[ResourceLocal],
    requirement: &RawCellReleaseParamRequirement,
) -> Result<
    ResourceSummaryStableRawCellReleaseParamRequirement,
    ResourceSummaryStableRawInitParamFactsLeafEntryReject,
> {
    let suffix = requirement
        .suffix
        .iter()
        .map(|projection| stable_place_projection(types, params, projection))
        .collect::<Result<Vec<_>, _>>()?;
    let ty = ResourceSummaryStableTypeKey::from_type(types, requirement.ty).ok_or(
        ResourceSummaryStableRawInitParamFactsLeafEntryReject::ParamReleaseRequirementType,
    )?;
    Ok(ResourceSummaryStableRawCellReleaseParamRequirement {
        param_index: requirement.param_index,
        suffix,
        ty,
        kind: stable_raw_cell_release_requirement_kind(requirement.kind),
    })
}

fn stable_place_projection(
    types: &TypeCtx,
    params: &[ResourceLocal],
    projection: &PlaceProjection,
) -> Result<
    ResourceSummaryStablePlaceProjection,
    ResourceSummaryStableRawInitParamFactsLeafEntryReject,
> {
    Ok(match projection {
        PlaceProjection::Field {
            index,
            offset_bytes,
        } => ResourceSummaryStablePlaceProjection::Field {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        PlaceProjection::TupleField {
            index,
            offset_bytes,
        } => ResourceSummaryStablePlaceProjection::TupleField {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        PlaceProjection::EnumPayload { variant } => {
            ResourceSummaryStablePlaceProjection::EnumPayload {
                variant: variant.clone(),
            }
        }
        PlaceProjection::Deref => ResourceSummaryStablePlaceProjection::Deref,
        PlaceProjection::StorageOffset(offset) => {
            ResourceSummaryStablePlaceProjection::StorageOffset(stable_resource_offset(
                types, params, offset,
            ))
        }
    })
}

fn stable_resource_offset(
    types: &TypeCtx,
    params: &[ResourceLocal],
    offset: &ResourceOffset,
) -> ResourceSummaryStableOffset {
    match offset {
        ResourceOffset::Known(value) => ResourceSummaryStableOffset::Known(*value),
        ResourceOffset::Symbolic { place } => {
            if let Some(place) = stable_resource_offset_place(types, params, place) {
                ResourceSummaryStableOffset::Symbolic {
                    place: Box::new(place),
                }
            } else {
                ResourceSummaryStableOffset::Unknown
            }
        }
        ResourceOffset::ScaledSymbolic { place, scale } => {
            if let Some(place) = stable_resource_offset_place(types, params, place) {
                ResourceSummaryStableOffset::ScaledSymbolic {
                    place: Box::new(place),
                    scale: *scale,
                }
            } else {
                ResourceSummaryStableOffset::Unknown
            }
        }
        ResourceOffset::Offset { place, offset } => {
            if let Some(place) = stable_resource_offset_place(types, params, place) {
                ResourceSummaryStableOffset::Offset {
                    place: Box::new(place),
                    offset: *offset,
                }
            } else {
                ResourceSummaryStableOffset::Unknown
            }
        }
        ResourceOffset::ScaledOffset {
            place,
            offset,
            scale,
        } => {
            if let Some(place) = stable_resource_offset_place(types, params, place) {
                ResourceSummaryStableOffset::ScaledOffset {
                    place: Box::new(place),
                    offset: *offset,
                    scale: *scale,
                }
            } else {
                ResourceSummaryStableOffset::Unknown
            }
        }
        ResourceOffset::Unknown => ResourceSummaryStableOffset::Unknown,
    }
}

fn stable_resource_offset_place(
    types: &TypeCtx,
    params: &[ResourceLocal],
    place: &Place,
) -> Option<ResourceSummaryStablePlace> {
    let summary_place = summary_place_for_params(params, place)?;
    stable_summary_place(types, &summary_place)
}

fn stable_raw_cell_release_requirement_kind(
    kind: RawCellReleaseRequirementKind,
) -> ResourceSummaryStableRawCellReleaseRequirementKind {
    match kind {
        RawCellReleaseRequirementKind::Store => {
            ResourceSummaryStableRawCellReleaseRequirementKind::Store
        }
        RawCellReleaseRequirementKind::Dealloc => {
            ResourceSummaryStableRawCellReleaseRequirementKind::Dealloc
        }
        RawCellReleaseRequirementKind::Realloc => {
            ResourceSummaryStableRawCellReleaseRequirementKind::Realloc
        }
        RawCellReleaseRequirementKind::Fill => {
            ResourceSummaryStableRawCellReleaseRequirementKind::Fill
        }
        RawCellReleaseRequirementKind::BulkDestination => {
            ResourceSummaryStableRawCellReleaseRequirementKind::BulkDestination
        }
        RawCellReleaseRequirementKind::BulkSource => {
            ResourceSummaryStableRawCellReleaseRequirementKind::BulkSource
        }
    }
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
        SummaryOffset::Unknown => ResourceSummaryStableOffset::Unknown,
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
        ResourceSummaryStableOffset::Unknown => SummaryOffset::Unknown,
    })
}

fn reproject_raw_init_param_cell(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    cell: &ResourceSummaryStableRawInitParamCell,
) -> Result<
    RawCellInitializationParamCell,
    ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject,
> {
    let base = ctx
        .function
        .params
        .get(cell.param_index)
        .ok_or(ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamCellProjection)?
        .ty;
    let (suffix, ty) =
        reproject_raw_init_param_cell_summary_suffix(ctx, base, &cell.ty, &cell.suffix)?;
    Ok(RawCellInitializationParamCell {
        param_index: cell.param_index,
        suffix,
        ty,
        holds_raw_address: cell.holds_raw_address,
    })
}

fn reproject_raw_init_param_cell_summary_suffix(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    base_ty: TypeId,
    stable_ty: &ResourceSummaryStableTypeKey,
    suffix: &[ResourceSummaryStableProjection],
) -> Result<
    (Vec<SummaryProjection>, TypeId),
    ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject,
> {
    let stable_key = stable_ty;
    let mut out = Vec::new();
    let mut current_ty = base_ty;
    let mut used_stored_cell_ty = false;
    for stable_projection in suffix {
        let projection = reproject_summary_projection(ctx, stable_projection).ok_or(
            ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamCellProjection,
        )?;
        current_ty = reproject_raw_init_param_cell_projection_result_type(
            ctx,
            current_ty,
            stable_key,
            &projection,
            &mut used_stored_cell_ty,
        )?;
        out.push(projection);
    }
    if !raw_init_param_cell_type_matches_reprojected_result(
        ctx,
        stable_key,
        current_ty,
        used_stored_cell_ty,
    ) {
        return Err(
            ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamCellResultType,
        );
    }
    Ok((out, current_ty))
}

fn reproject_raw_init_param_cell_projection_result_type(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    current_ty: TypeId,
    stable_cell_ty: &ResourceSummaryStableTypeKey,
    projection: &SummaryProjection,
    used_stored_cell_ty: &mut bool,
) -> Result<TypeId, ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject> {
    if matches!(projection, SummaryProjection::Deref) {
        if let Some(ty) = summary_projection_result_type(ctx.types, current_ty, projection) {
            return Ok(ty);
        }
        // raw-init の param cell は raw address から見た cell view を表せる。
        // その `Deref` は通常の参照型 dereference ではないため、ここでは保存済み
        // cell 型だけを復元先として採用し、field/tuple など通常projectionの検証は
        // 引き続き `summary_projection_result_type` に任せる。
        let stable_cell_ty = ctx.reproject_type(stable_cell_ty).ok_or(
            ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamCellStableType,
        )?;
        *used_stored_cell_ty = true;
        return Ok(stable_cell_ty);
    }
    validate_projection_layout(ctx.types, current_ty, projection)
        .ok_or(ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamCellProjection)?;
    if let Some(ty) = summary_projection_result_type(ctx.types, current_ty, projection) {
        return Ok(ty);
    }
    Err(ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamCellProjection)
}

fn raw_init_param_cell_type_matches_reprojected_result(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    stable_key: &ResourceSummaryStableTypeKey,
    current_ty: TypeId,
    used_stored_cell_ty: bool,
) -> bool {
    // base parameter と suffix から通常の layout 規則で型が決まる場合、保存済みの
    // cell 型 key は replay の根拠にしない。cache key は関数 signature と body を
    // 既に固定しており、現在の signature から計算した型が正しい replay surface になる。
    // raw address deref のように型付き projection では値型を得られない場合だけ、
    // 保存済み cell 型を proof boundary として使い、その型 key との一致を要求する。
    if !used_stored_cell_ty {
        return true;
    }
    if stable_key.matches_type(ctx.types, current_ty) {
        return true;
    }
    false
}

fn reproject_raw_cell_release_param_requirement(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    requirement: &ResourceSummaryStableRawCellReleaseParamRequirement,
) -> Result<
    RawCellReleaseParamRequirement,
    ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject,
> {
    let base = ctx
        .function
        .params
        .get(requirement.param_index)
        .ok_or(
            ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamReleaseRequirementProjection,
        )?
        .ty;
    let (suffix, ty) = reproject_place_projection_suffix(ctx, base, &requirement.suffix).ok_or(
        ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamReleaseRequirementProjection,
    )?;
    if !requirement.ty.matches_type(ctx.types, ty) {
        return Err(
            ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamReleaseRequirementType,
        );
    }
    Ok(RawCellReleaseParamRequirement {
        param_index: requirement.param_index,
        suffix,
        ty,
        kind: reproject_raw_cell_release_requirement_kind(requirement.kind),
    })
}

fn reproject_place_projection_suffix(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    base_ty: TypeId,
    suffix: &[ResourceSummaryStablePlaceProjection],
) -> Option<(Vec<PlaceProjection>, TypeId)> {
    let mut out = Vec::new();
    let mut current_ty = base_ty;
    for projection in suffix {
        let projection = reproject_place_projection(ctx, projection)?;
        validate_place_projection_layout(ctx.types, current_ty, &projection)?;
        current_ty = projection_result_type(ctx.types, current_ty, &projection)?;
        out.push(projection);
    }
    Some((out, current_ty))
}

fn reproject_place_projection(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    projection: &ResourceSummaryStablePlaceProjection,
) -> Option<PlaceProjection> {
    Some(match projection {
        ResourceSummaryStablePlaceProjection::Field {
            index,
            offset_bytes,
        } => PlaceProjection::Field {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        ResourceSummaryStablePlaceProjection::TupleField {
            index,
            offset_bytes,
        } => PlaceProjection::TupleField {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        ResourceSummaryStablePlaceProjection::EnumPayload { variant } => {
            PlaceProjection::EnumPayload {
                variant: variant.clone(),
            }
        }
        ResourceSummaryStablePlaceProjection::Deref => PlaceProjection::Deref,
        ResourceSummaryStablePlaceProjection::StorageOffset(offset) => {
            PlaceProjection::StorageOffset(reproject_resource_offset(ctx, offset)?)
        }
    })
}

fn reproject_resource_offset(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    offset: &ResourceSummaryStableOffset,
) -> Option<ResourceOffset> {
    Some(match offset {
        ResourceSummaryStableOffset::Known(value) => ResourceOffset::Known(*value),
        ResourceSummaryStableOffset::Symbolic { place } => ResourceOffset::Symbolic {
            place: Box::new(reproject_stable_place_to_place(ctx, place)?),
        },
        ResourceSummaryStableOffset::ScaledSymbolic { place, scale } => {
            ResourceOffset::ScaledSymbolic {
                place: Box::new(reproject_stable_place_to_place(ctx, place)?),
                scale: *scale,
            }
        }
        ResourceSummaryStableOffset::Offset { place, offset } => ResourceOffset::Offset {
            place: Box::new(reproject_stable_place_to_place(ctx, place)?),
            offset: *offset,
        },
        ResourceSummaryStableOffset::ScaledOffset {
            place,
            offset,
            scale,
        } => ResourceOffset::ScaledOffset {
            place: Box::new(reproject_stable_place_to_place(ctx, place)?),
            offset: *offset,
            scale: *scale,
        },
        ResourceSummaryStableOffset::Unknown => ResourceOffset::Unknown,
    })
}

fn reproject_stable_place_to_place(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    place: &ResourceSummaryStablePlace,
) -> Option<Place> {
    let base = ctx
        .function
        .params
        .get(place.parameter_index)?
        .place
        .clone();
    let (suffix, ty) = reproject_stable_projection_suffix_as_place(ctx, base.ty, &place.suffix)?;
    if !place.ty.matches_type(ctx.types, ty) {
        return None;
    }
    let mut out = base;
    out.projections.extend(suffix);
    out.ty = ty;
    Some(out)
}

fn reproject_stable_projection_suffix_as_place(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    base_ty: TypeId,
    suffix: &[ResourceSummaryStableProjection],
) -> Option<(Vec<PlaceProjection>, TypeId)> {
    let mut out = Vec::new();
    let mut current_ty = base_ty;
    for projection in suffix {
        let projection = reproject_stable_projection_as_place(ctx, projection)?;
        validate_place_projection_layout(ctx.types, current_ty, &projection)?;
        current_ty = projection_result_type(ctx.types, current_ty, &projection)?;
        out.push(projection);
    }
    Some((out, current_ty))
}

fn reproject_stable_projection_as_place(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    projection: &ResourceSummaryStableProjection,
) -> Option<PlaceProjection> {
    Some(match projection {
        ResourceSummaryStableProjection::Field {
            index,
            offset_bytes,
        } => PlaceProjection::Field {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        ResourceSummaryStableProjection::TupleField {
            index,
            offset_bytes,
        } => PlaceProjection::TupleField {
            index: *index,
            offset_bytes: *offset_bytes,
        },
        ResourceSummaryStableProjection::EnumPayload { variant } => PlaceProjection::EnumPayload {
            variant: variant.clone(),
        },
        ResourceSummaryStableProjection::Deref => PlaceProjection::Deref,
        ResourceSummaryStableProjection::StorageOffset(offset) => {
            PlaceProjection::StorageOffset(reproject_resource_offset(ctx, offset)?)
        }
    })
}

fn validate_place_projection_layout(
    types: &TypeCtx,
    base_ty: TypeId,
    projection: &PlaceProjection,
) -> Option<()> {
    match projection {
        PlaceProjection::Field {
            index,
            offset_bytes,
        }
        | PlaceProjection::TupleField {
            index,
            offset_bytes,
        } => {
            let field = aggregate_fields_with_offsets(types, base_ty)
                .get(*index)
                .copied()?;
            (field.offset == *offset_bytes).then_some(())
        }
        PlaceProjection::EnumPayload { .. }
        | PlaceProjection::Deref
        | PlaceProjection::StorageOffset(_) => {
            projection_result_type(types, base_ty, projection).map(|_| ())
        }
    }
}

fn reproject_raw_cell_release_requirement_kind(
    kind: ResourceSummaryStableRawCellReleaseRequirementKind,
) -> RawCellReleaseRequirementKind {
    match kind {
        ResourceSummaryStableRawCellReleaseRequirementKind::Store => {
            RawCellReleaseRequirementKind::Store
        }
        ResourceSummaryStableRawCellReleaseRequirementKind::Dealloc => {
            RawCellReleaseRequirementKind::Dealloc
        }
        ResourceSummaryStableRawCellReleaseRequirementKind::Realloc => {
            RawCellReleaseRequirementKind::Realloc
        }
        ResourceSummaryStableRawCellReleaseRequirementKind::Fill => {
            RawCellReleaseRequirementKind::Fill
        }
        ResourceSummaryStableRawCellReleaseRequirementKind::BulkDestination => {
            RawCellReleaseRequirementKind::BulkDestination
        }
        ResourceSummaryStableRawCellReleaseRequirementKind::BulkSource => {
            RawCellReleaseRequirementKind::BulkSource
        }
    }
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
    use crate::types::{NominalStableTypeIdentity, NominalStableTypeKind, TypeCtx, TypeKind};

    use super::super::super::collection_slot_summary_model::{
        CollectionSlotInitializedRangeDropTraversalCertificate,
        CollectionSlotInitializedRangeDropTraversalProof,
    };
    use super::super::super::initialized_summary::{
        RawCellInitializationFunctionSummary, RawCellInitializationParamCell,
    };
    use super::super::super::initialized_summary_release_model::{
        RawCellReleaseParamRequirement, RawCellReleaseRequirementKind,
    };
    use super::super::super::model::{
        Place, PlaceProjection, ResourceBlockId, ResourceFunction, ResourceLocal, ResourceOffset,
    };
    use super::*;

    fn function_with_param(param_ty: crate::types::TypeId) -> ResourceFunction {
        function_with_params(vec![("storage", param_ty)], param_ty)
    }

    fn function_with_params(
        params: Vec<(&str, crate::types::TypeId)>,
        result_ty: crate::types::TypeId,
    ) -> ResourceFunction {
        ResourceFunction {
            name: "summary_cache_subject".to_string(),
            origin_name: "summary_cache_subject".to_string(),
            type_params: Vec::new(),
            params: params
                .into_iter()
                .map(|(name, ty)| ResourceLocal {
                    name: name.to_string(),
                    ty,
                    mutable: false,
                    place: Place::local(name.to_string(), ty),
                })
                .collect(),
            result: result_ty,
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

    fn empty_raw_init_summary(function: &ResourceFunction) -> RawCellInitializationFunctionSummary {
        RawCellInitializationFunctionSummary {
            function: function.name.clone(),
            type_params: function.type_params.clone(),
            return_cells: Vec::new(),
            return_byte_ranges: Vec::new(),
            param_cells: Vec::new(),
            param_byte_ranges: Vec::new(),
            param_release_requirements: Vec::new(),
            variant_param_cells: Vec::new(),
            variant_param_byte_ranges: Vec::new(),
            variant_required_param_cells: Vec::new(),
            variant_conditions: Vec::new(),
        }
    }

    fn nominal_struct_identity(name: &str) -> NominalStableTypeIdentity {
        NominalStableTypeIdentity::new(
            NominalStableTypeKind::Struct,
            "/user/types.nepl".to_string(),
            name.to_string(),
            0,
            1,
        )
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
    fn stable_drop_traversal_forall_value_reprojects_unknown_offsets() {
        let types = TypeCtx::new();
        let function = function_with_param(types.i32());
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
        let value = stable_drop_traversal_forall_value(&types, &op)
            .expect("unknown storage offset is a conservative summary fact");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_drop_traversal_forall_value(&ctx, &value)
            .expect("unknown storage offset should reproject as unknown");

        assert_eq!(reprojected, op);
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
    fn stable_drop_traversal_forall_value_reprojects_nominal_expected_type() {
        let mut types = TypeCtx::new();
        let field = types.i32();
        let nominal = types.register_named_with_stable_identity(
            "Record".to_string(),
            TypeKind::Struct {
                name: "Record".to_string(),
                type_params: Vec::new(),
                fields: vec![field],
                field_names: vec!["value".to_string()],
            },
            nominal_struct_identity("Record"),
        );
        let function = function_with_param(nominal);
        let op = CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage: SummaryPlace {
                parameter_index: 0,
                suffix: Vec::new(),
                ty: nominal,
            },
            initialized_count: CollectionSlotLifecycleSummaryI32Operand::KnownI32 {
                value: 1,
                ty: types.i32(),
            },
            expected_ty: nominal,
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                CollectionSlotInitializedRangeDropTraversalCertificate {
                    element_stride: 4,
                    drop_proof: CollectionSlotInitializedRangeDropTraversalProof::StateOnly,
                },
            ),
        };
        let value = stable_drop_traversal_forall_value(&types, &op)
            .expect("nominal forall drop traversal should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("nominal function boundary should be reprojectable");

        let reprojected = reproject_drop_traversal_forall_value(&ctx, &value)
            .expect("nominal stable value should reproject");

        assert_eq!(reprojected, op);
    }

    #[test]
    fn stable_raw_init_param_facts_reprojects_nominal_field_type_from_signature_tree() {
        let mut types = TypeCtx::new();
        let field = types.i32();
        let nominal = types.register_named_with_stable_identity(
            "Record".to_string(),
            TypeKind::Struct {
                name: "Record".to_string(),
                type_params: Vec::new(),
                fields: vec![field],
                field_names: vec!["value".to_string()],
            },
            nominal_struct_identity("Record"),
        );
        let function = function_with_param(nominal);
        let summary = RawCellInitializationFunctionSummary {
            function: function.name.clone(),
            type_params: function.type_params.clone(),
            return_cells: Vec::new(),
            return_byte_ranges: Vec::new(),
            param_cells: vec![RawCellInitializationParamCell {
                param_index: 0,
                suffix: vec![SummaryProjection::Field {
                    index: 0,
                    offset_bytes: 0,
                }],
                ty: field,
                holds_raw_address: false,
            }],
            param_byte_ranges: Vec::new(),
            param_release_requirements: Vec::new(),
            variant_param_cells: Vec::new(),
            variant_param_byte_ranges: Vec::new(),
            variant_required_param_cells: Vec::new(),
            variant_conditions: Vec::new(),
        };
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("nominal field param facts should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("signature tree should register nominal field type");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("nominal field fact should reproject");

        assert_eq!(reprojected, summary);
    }

    #[test]
    fn stable_raw_init_param_facts_reprojects_instantiated_generic_nominal_field_type() {
        let mut types = TypeCtx::new();
        let definition_generic = types.fresh_var(Some("T".to_string()));
        let nominal = types.register_named_with_stable_identity(
            "Wrapper".to_string(),
            TypeKind::Struct {
                name: "Wrapper".to_string(),
                type_params: vec![definition_generic],
                fields: vec![definition_generic],
                field_names: vec!["value".to_string()],
            },
            nominal_struct_identity("Wrapper"),
        );
        let function_generic = types.fresh_var(Some("T".to_string()));
        let applied = types.apply(nominal, vec![function_generic]);
        let mut function = function_with_params(vec![("storage", applied)], types.unit());
        function.type_params.push(function_generic);
        let summary = RawCellInitializationFunctionSummary {
            function: function.name.clone(),
            type_params: function.type_params.clone(),
            return_cells: Vec::new(),
            return_byte_ranges: Vec::new(),
            param_cells: vec![RawCellInitializationParamCell {
                param_index: 0,
                suffix: vec![SummaryProjection::Field {
                    index: 0,
                    offset_bytes: 0,
                }],
                ty: function_generic,
                holds_raw_address: false,
            }],
            param_byte_ranges: Vec::new(),
            param_release_requirements: Vec::new(),
            variant_param_cells: Vec::new(),
            variant_param_byte_ranges: Vec::new(),
            variant_required_param_cells: Vec::new(),
            variant_conditions: Vec::new(),
        };
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("instantiated generic nominal field param facts should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[function_generic])
            .expect("definition generic should not shadow the instantiated boundary generic");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("instantiated generic nominal field fact should reproject");

        assert_eq!(reprojected, summary);
    }

    #[test]
    fn stable_type_reprojection_accepts_duplicate_nominal_signature_keys() {
        let mut types = TypeCtx::new();
        let identity = nominal_struct_identity("StableRecord");
        let first = types.register_named_with_stable_identity(
            "FirstRecord".to_string(),
            TypeKind::Struct {
                name: "FirstRecord".to_string(),
                type_params: Vec::new(),
                fields: vec![types.i32()],
                field_names: vec!["value".to_string()],
            },
            identity.clone(),
        );
        let second = types.register_named_with_stable_identity(
            "SecondRecord".to_string(),
            TypeKind::Struct {
                name: "SecondRecord".to_string(),
                type_params: Vec::new(),
                fields: vec![types.i32()],
                field_names: vec!["value".to_string()],
            },
            identity,
        );
        let function =
            function_with_params(vec![("first", first), ("second", second)], types.unit());

        assert!(ResourceSummaryTypeReprojection::new(&types, &function, &[]).is_some());
        let summary = RawCellInitializationFunctionSummary {
            function: function.name.clone(),
            type_params: function.type_params.clone(),
            return_cells: Vec::new(),
            return_byte_ranges: Vec::new(),
            param_cells: vec![RawCellInitializationParamCell {
                param_index: 1,
                suffix: Vec::new(),
                ty: second,
                holds_raw_address: false,
            }],
            param_byte_ranges: Vec::new(),
            param_release_requirements: Vec::new(),
            variant_param_cells: Vec::new(),
            variant_param_byte_ranges: Vec::new(),
            variant_required_param_cells: Vec::new(),
            variant_conditions: Vec::new(),
        };
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("duplicate nominal signature fact should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("duplicate nominal stable keys should be accepted as signature aliases");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("duplicate nominal fact should replay from current signature");

        assert_eq!(reprojected, summary);
    }

    #[test]
    fn stable_type_reprojection_rejects_duplicate_open_generic_signature_keys() {
        let mut types = TypeCtx::new();
        let first = types.fresh_var(Some("T".to_string()));
        let second = types.fresh_var(Some("T".to_string()));
        let function =
            function_with_params(vec![("first", first), ("second", second)], types.unit());

        assert!(ResourceSummaryTypeReprojection::new(&types, &function, &[]).is_none());
    }

    #[test]
    fn stable_type_reprojection_rejects_nested_open_generic_duplicates_through_boundary() {
        let mut types = TypeCtx::new();
        let first_generic = types.fresh_var(Some("T".to_string()));
        let second_generic = types.fresh_var(Some("T".to_string()));
        let first_box = types.box_ty(first_generic);
        let second_box = types.box_ty(second_generic);
        let function = function_with_params(
            vec![("first", first_box), ("second", second_box)],
            types.unit(),
        );
        let boundary = super::super::super::owner_summary_type_params::owner_summary_type_params(
            &types, &function,
        );

        assert_eq!(boundary.len(), 2);
        assert!(ResourceSummaryTypeReprojection::new(&types, &function, &boundary).is_none());
    }

    #[test]
    fn stable_raw_init_param_facts_reprojects_duplicate_structural_signature_key() {
        let mut types = TypeCtx::new();
        let item = types.i32();
        let first = types.tuple(vec![item]);
        let second = types.tuple(vec![item]);
        let function =
            function_with_params(vec![("first", first), ("second", second)], types.unit());
        let summary = RawCellInitializationFunctionSummary {
            function: function.name.clone(),
            type_params: function.type_params.clone(),
            return_cells: Vec::new(),
            return_byte_ranges: Vec::new(),
            param_cells: vec![RawCellInitializationParamCell {
                param_index: 1,
                suffix: Vec::new(),
                ty: second,
                holds_raw_address: false,
            }],
            param_byte_ranges: Vec::new(),
            param_release_requirements: Vec::new(),
            variant_param_cells: Vec::new(),
            variant_param_byte_ranges: Vec::new(),
            variant_required_param_cells: Vec::new(),
            variant_conditions: Vec::new(),
        };
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("duplicate structural signature fact should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("duplicate structural stable keys should be accepted as signature aliases");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("duplicate structural fact should replay from current signature");

        assert_eq!(reprojected, summary);
    }

    #[test]
    fn stable_raw_init_reprojection_reports_param_cell_projection_mismatch() {
        let mut types = TypeCtx::new();
        let field = types.i32();
        let nominal = types.register_named_with_stable_identity(
            "Record".to_string(),
            TypeKind::Struct {
                name: "Record".to_string(),
                type_params: Vec::new(),
                fields: vec![field],
                field_names: vec!["value".to_string()],
            },
            nominal_struct_identity("Record"),
        );
        let function = function_with_param(nominal);
        let mut summary = empty_raw_init_summary(&function);
        summary.param_cells.push(RawCellInitializationParamCell {
            param_index: 0,
            suffix: vec![SummaryProjection::Field {
                index: 0,
                offset_bytes: 0,
            }],
            ty: field,
            holds_raw_address: false,
        });
        let mut entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("valid param cell should convert before corruption");
        entry.param_cells[0].suffix[0] = ResourceSummaryStableProjection::Field {
            index: 0,
            offset_bytes: 4,
        };
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("nominal function boundary should be reprojectable");

        let result = reproject_raw_init_param_facts_leaf_entry_result(&ctx, &function.name, &entry);

        assert!(matches!(
            result,
            Err(ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamCellProjection)
        ));
    }

    #[test]
    fn stable_raw_init_reprojection_uses_signature_type_for_projection_derived_cell() {
        let types = TypeCtx::new();
        let function = function_with_param(types.i32());
        let mut summary = empty_raw_init_summary(&function);
        summary.param_cells.push(RawCellInitializationParamCell {
            param_index: 0,
            suffix: Vec::new(),
            ty: types.i32(),
            holds_raw_address: false,
        });
        let mut entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("valid param cell should convert before corruption");
        entry.param_cells[0].ty = ResourceSummaryStableTypeKey::from_type(&types, types.bool())
            .expect("bool has a stable type key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("projection-derived param cell type should come from the current signature");

        assert_eq!(reprojected.param_cells[0].ty, types.i32());
    }

    #[test]
    fn stable_raw_init_param_cell_reprojects_raw_deref_value_type() {
        let types = TypeCtx::new();
        let function = function_with_param(types.i32());
        let mut summary = empty_raw_init_summary(&function);
        summary.param_cells.push(RawCellInitializationParamCell {
            param_index: 0,
            suffix: vec![
                SummaryProjection::StorageOffset(SummaryOffset::Known(0)),
                SummaryProjection::Deref,
            ],
            ty: types.u8(),
            holds_raw_address: false,
        });
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("raw-deref param cell should convert with its explicit value type");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("raw-deref param cell should use the stable value type");

        assert_eq!(reprojected, summary);
    }

    #[test]
    fn stable_raw_init_param_cell_reprojects_non_signature_nominal_value_type() {
        let mut types = TypeCtx::new();
        let field = types.i32();
        let value_ty = types.register_named_with_stable_identity(
            "Payload".to_string(),
            TypeKind::Struct {
                name: "Payload".to_string(),
                type_params: Vec::new(),
                fields: vec![field],
                field_names: vec!["value".to_string()],
            },
            nominal_struct_identity("Payload"),
        );
        let function = function_with_param(types.i32());
        let mut summary = empty_raw_init_summary(&function);
        summary.param_cells.push(RawCellInitializationParamCell {
            param_index: 0,
            suffix: vec![
                SummaryProjection::StorageOffset(SummaryOffset::Known(0)),
                SummaryProjection::Deref,
            ],
            ty: value_ty,
            holds_raw_address: false,
        });
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("non-signature nominal raw cell type should convert to a stable key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("non-signature nominal raw cell type should be found in current TypeCtx");

        assert_eq!(reprojected, summary);
    }

    #[test]
    fn stable_raw_init_param_cell_rejects_non_boundary_open_generic_value_type() {
        let mut types = TypeCtx::new();
        let value_ty = types.fresh_var(Some("T".to_string()));
        let function = function_with_param(types.i32());
        let mut summary = empty_raw_init_summary(&function);
        summary.param_cells.push(RawCellInitializationParamCell {
            param_index: 0,
            suffix: vec![
                SummaryProjection::StorageOffset(SummaryOffset::Known(0)),
                SummaryProjection::Deref,
            ],
            ty: value_ty,
            holds_raw_address: false,
        });
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("labelled generic raw cell type can be represented as a stable key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let result = reproject_raw_init_param_facts_leaf_entry_result(&ctx, &function.name, &entry);

        assert!(matches!(
            result,
            Err(ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamCellStableType)
        ));
    }

    #[test]
    fn stable_raw_init_param_cell_reprojects_owner_boundary_open_generic_value_type() {
        let mut types = TypeCtx::new();
        let value_ty = types.fresh_var(Some("T".to_string()));
        let function = function_with_param(types.i32());
        let mut summary = empty_raw_init_summary(&function);
        summary.type_params.push(value_ty);
        summary.param_cells.push(RawCellInitializationParamCell {
            param_index: 0,
            suffix: vec![
                SummaryProjection::StorageOffset(SummaryOffset::Known(0)),
                SummaryProjection::Deref,
            ],
            ty: value_ty,
            holds_raw_address: false,
        });
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("labelled generic raw cell type should convert to a stable key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[value_ty])
            .expect("owner summary boundary should make the generic reprojectable");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("owner summary boundary should reproject the raw cell value type");

        assert_eq!(reprojected, summary);
    }

    #[test]
    fn stable_raw_init_param_cell_rebases_projection_derived_open_generic_value_type() {
        let mut types = TypeCtx::new();
        let definition_generic = types.fresh_var(Some("T".to_string()));
        let nominal = types.register_named_with_stable_identity(
            "Wrapper".to_string(),
            TypeKind::Struct {
                name: "Wrapper".to_string(),
                type_params: vec![definition_generic],
                fields: vec![definition_generic],
                field_names: vec!["value".to_string()],
            },
            nominal_struct_identity("Wrapper"),
        );
        let stale_callee_generic = types.fresh_var(Some("T".to_string()));
        let applied = types.apply(nominal, vec![types.i32()]);
        let function = function_with_param(applied);
        let mut summary = empty_raw_init_summary(&function);
        summary.param_cells.push(RawCellInitializationParamCell {
            param_index: 0,
            suffix: vec![SummaryProjection::Field {
                index: 0,
                offset_bytes: 0,
            }],
            ty: stale_callee_generic,
            holds_raw_address: false,
        });
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("projection-derived open generic value type should remain representable");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("applied signature should be reprojectable");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("projection-derived type should be computed from the current signature");

        let mut expected = empty_raw_init_summary(&function);
        expected.param_cells.push(RawCellInitializationParamCell {
            param_index: 0,
            suffix: vec![SummaryProjection::Field {
                index: 0,
                offset_bytes: 0,
            }],
            ty: types.i32(),
            holds_raw_address: false,
        });
        assert_eq!(reprojected, expected);
    }

    #[test]
    fn stable_raw_init_reprojection_reports_param_release_projection_mismatch() {
        let mut types = TypeCtx::new();
        let field = types.i32();
        let nominal = types.register_named_with_stable_identity(
            "Record".to_string(),
            TypeKind::Struct {
                name: "Record".to_string(),
                type_params: Vec::new(),
                fields: vec![field],
                field_names: vec!["value".to_string()],
            },
            nominal_struct_identity("Record"),
        );
        let function = function_with_param(nominal);
        let mut summary = empty_raw_init_summary(&function);
        summary
            .param_release_requirements
            .push(RawCellReleaseParamRequirement {
                param_index: 0,
                suffix: vec![PlaceProjection::Field {
                    index: 0,
                    offset_bytes: 0,
                }],
                ty: field,
                kind: RawCellReleaseRequirementKind::Store,
            });
        let mut entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("valid release requirement should convert before corruption");
        entry.param_release_requirements[0].suffix[0] =
            ResourceSummaryStablePlaceProjection::Field {
                index: 0,
                offset_bytes: 4,
            };
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("nominal function boundary should be reprojectable");

        let result = reproject_raw_init_param_facts_leaf_entry_result(&ctx, &function.name, &entry);

        assert!(matches!(
            result,
            Err(
                ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamReleaseRequirementProjection
            )
        ));
    }

    #[test]
    fn stable_raw_init_reprojection_reports_param_release_type_mismatch() {
        let types = TypeCtx::new();
        let function = function_with_param(types.i32());
        let mut summary = empty_raw_init_summary(&function);
        summary
            .param_release_requirements
            .push(RawCellReleaseParamRequirement {
                param_index: 0,
                suffix: Vec::new(),
                ty: types.i32(),
                kind: RawCellReleaseRequirementKind::Store,
            });
        let mut entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("valid release requirement should convert before corruption");
        entry.param_release_requirements[0].ty =
            ResourceSummaryStableTypeKey::from_type(&types, types.bool())
                .expect("bool has a stable type key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let result = reproject_raw_init_param_facts_leaf_entry_result(&ctx, &function.name, &entry);

        assert!(matches!(
            result,
            Err(
                ResourceSummaryRawInitParamFactsLeafEntryReprojectionReject::ParamReleaseRequirementType
            )
        ));
    }

    #[test]
    fn stable_raw_init_release_requirement_reprojects_scaled_symbolic_storage_offset() {
        let types = TypeCtx::new();
        let function = function_with_params(
            vec![("storage", types.i32()), ("index", types.i32())],
            types.unit(),
        );
        let suffix = vec![PlaceProjection::StorageOffset(
            ResourceOffset::ScaledOffset {
                place: Box::new(Place::local("index".to_string(), types.i32())),
                offset: 4,
                scale: 4,
            },
        )];
        let summary = RawCellInitializationFunctionSummary {
            function: function.name.clone(),
            type_params: function.type_params.clone(),
            return_cells: Vec::new(),
            return_byte_ranges: Vec::new(),
            param_cells: Vec::new(),
            param_byte_ranges: Vec::new(),
            param_release_requirements: vec![RawCellReleaseParamRequirement {
                param_index: 0,
                suffix: suffix.clone(),
                ty: types.i32(),
                kind: RawCellReleaseRequirementKind::Store,
            }],
            variant_param_cells: Vec::new(),
            variant_param_byte_ranges: Vec::new(),
            variant_required_param_cells: Vec::new(),
            variant_conditions: Vec::new(),
        };
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("parameter-relative storage offset should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("parameter-relative release requirement should reproject");

        assert_eq!(reprojected, summary);
    }

    #[test]
    fn stable_raw_init_release_requirement_reprojects_unknown_storage_offset() {
        let types = TypeCtx::new();
        let function = function_with_param(types.i32());
        let summary = RawCellInitializationFunctionSummary {
            function: function.name.clone(),
            type_params: function.type_params.clone(),
            return_cells: Vec::new(),
            return_byte_ranges: Vec::new(),
            param_cells: Vec::new(),
            param_byte_ranges: Vec::new(),
            param_release_requirements: vec![RawCellReleaseParamRequirement {
                param_index: 0,
                suffix: vec![PlaceProjection::StorageOffset(ResourceOffset::Unknown)],
                ty: types.i32(),
                kind: RawCellReleaseRequirementKind::Store,
            }],
            variant_param_cells: Vec::new(),
            variant_param_byte_ranges: Vec::new(),
            variant_required_param_cells: Vec::new(),
            variant_conditions: Vec::new(),
        };
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("unknown storage offset should remain a conservative stable fact");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("unknown storage offset release requirement should reproject");

        assert_eq!(reprojected, summary);
    }

    #[test]
    fn stable_raw_init_release_requirement_degrades_local_offset_to_unknown() {
        let types = TypeCtx::new();
        let function = function_with_param(types.i32());
        let summary = RawCellInitializationFunctionSummary {
            function: function.name.clone(),
            type_params: function.type_params.clone(),
            return_cells: Vec::new(),
            return_byte_ranges: Vec::new(),
            param_cells: Vec::new(),
            param_byte_ranges: Vec::new(),
            param_release_requirements: vec![RawCellReleaseParamRequirement {
                param_index: 0,
                suffix: vec![PlaceProjection::StorageOffset(
                    ResourceOffset::ScaledOffset {
                        place: Box::new(Place::local("local_index".to_string(), types.i32())),
                        offset: 4,
                        scale: 4,
                    },
                )],
                ty: types.i32(),
                kind: RawCellReleaseRequirementKind::Store,
            }],
            variant_param_cells: Vec::new(),
            variant_param_byte_ranges: Vec::new(),
            variant_required_param_cells: Vec::new(),
            variant_conditions: Vec::new(),
        };
        let entry = stable_raw_init_param_facts_leaf_entry(&types, &function, &summary)
            .expect("local offset should degrade instead of blocking the stable entry");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_param_facts_leaf_entry(&ctx, &function.name, &entry)
            .expect("local offset should reproject through conservative unknown offset");

        assert_eq!(
            reprojected.param_release_requirements[0].suffix,
            vec![PlaceProjection::StorageOffset(ResourceOffset::Unknown)]
        );
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
