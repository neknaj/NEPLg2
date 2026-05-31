extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, storage_size_bytes};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::super::collection_slot_lifecycle::CollectionSlotLifecycleOp;
use super::super::collection_slot_lifecycle_model::{
    CollectionSlotLifecycleEvent, CollectionSlotState,
};
use super::super::collection_slot_state_table::CollectionSlotStateEntry;
use super::super::collection_slot_summary_model::{
    CollectionSlotInitializedRangeDropTraversalCertificate,
    CollectionSlotInitializedRangeDropTraversalProof,
    CollectionSlotLifecycleSummaryDropTraversalCoverage, CollectionSlotLifecycleSummaryI32Operand,
    CollectionSlotLifecycleSummaryOp,
};
use super::super::i32_scalar_return_facts::{
    I32ScalarParameterCondition, I32ScalarReturnAlias, I32ScalarReturnCondition,
    I32ScalarReturnConstant, I32ScalarReturnFacts, I32ScalarReturnOffset, I32ScalarReturnRelation,
};
use super::super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellInitializationParamCell,
    RawCellInitializationReturnCell,
};
use super::super::initialized_summary_byte_range_model::{
    RawCellInitializationParamByteRange, RawCellInitializationParamCount,
    RawCellInitializationReturnByteRange, RawCellInitializationReturnCount,
    RawCellInitializationVariantParamByteRange,
};
use super::super::initialized_summary_condition::RawCellValueCondition;
use super::super::initialized_summary_release_model::{
    RawCellReleaseParamRequirement, RawCellReleaseRequirementKind,
};
use super::super::initialized_summary_variant_model::{
    RawCellInitializationVariantCondition, RawCellInitializationVariantParamCell,
    RawCellInitializationVariantParamRequirement,
};
use super::super::model::{
    CellState, CellStateEntry, I32ValueCondition, Place, PlaceProjection, PlaceRoot,
    ResourceCallTarget, ResourceConditionFact, ResourceExprKind, ResourceFunction,
    ResourceI32RelationOp, ResourceId, ResourceLocal, ResourceMatchArm, ResourceOffset, ResourceOp,
    ResourceTerminator, StorageId,
};
use super::super::place_utils::projection_result_type;
use super::super::report::{ResourceCheckDeferred, ResourceFunctionCheck};
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
    ResourceSymbolic {
        place: Box<ResourceSummaryStableResourcePlace>,
    },
    ScaledSymbolic {
        place: Box<ResourceSummaryStablePlace>,
        scale: usize,
    },
    ResourceScaledSymbolic {
        place: Box<ResourceSummaryStableResourcePlace>,
        scale: usize,
    },
    Offset {
        place: Box<ResourceSummaryStablePlace>,
        offset: i64,
    },
    ResourceOffset {
        place: Box<ResourceSummaryStableResourcePlace>,
        offset: i64,
    },
    ScaledOffset {
        place: Box<ResourceSummaryStablePlace>,
        offset: i64,
        scale: usize,
    },
    ResourceScaledOffset {
        place: Box<ResourceSummaryStableResourcePlace>,
        offset: i64,
        scale: usize,
    },
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceSummaryStableOffsetPlace {
    Parameter(Box<ResourceSummaryStablePlace>),
    Resource(Box<ResourceSummaryStableResourcePlace>),
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

/// i32 scalar return facts の complete entry。
///
/// `I32ScalarReturnFacts` は return projection、parameter projection、scalar `TypeId` を
/// 含むため、そのまま session cache に保存できない。この entry は projection と type を
/// stable key に変換し、現在の関数 signature へ同じ facts を完全に戻せる場合だけ replay
/// する。部分保存を許すと call 境界の scalar/condition propagation が欠けるため、
/// aliases/offsets/relations/constants/conditions の全 surface を同じ entry に保持する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResourceSummaryStableI32ScalarReturnFactsEntry {
    aliases: Vec<ResourceSummaryStableI32ScalarReturnAlias>,
    offsets: Vec<ResourceSummaryStableI32ScalarReturnOffset>,
    relations: Vec<ResourceSummaryStableI32ScalarReturnRelation>,
    constants: Vec<ResourceSummaryStableI32ScalarReturnConstant>,
    return_conditions: Vec<ResourceSummaryStableI32ScalarReturnCondition>,
    parameter_conditions: Vec<ResourceSummaryStableI32ScalarParameterCondition>,
}

impl ResourceSummaryStableI32ScalarReturnFactsEntry {
    pub(super) fn len(&self) -> usize {
        self.aliases.len()
            + self.offsets.len()
            + self.relations.len()
            + self.constants.len()
            + self.return_conditions.len()
            + self.parameter_conditions.len()
    }
}

/// final initialized function check の diagnostic-free stable entry。
///
/// 初期 MVP では `ResourceCheckDiagnostic` と `auto_drop_points` を cache しない。どちらも
/// source span を現在の source map へ戻す別設計が必要なため、該当関数は no-store に倒す。
/// この entry は final cell / collection slot state と deferred counter だけを保存し、
/// 現在 compile の `TypeCtx` へ `TypeId` を再投影できる場合だけ check 実行を skip する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResourceSummaryStableInitializedFunctionCheckEntry {
    final_cells: Vec<ResourceSummaryStableCellStateEntry>,
    final_collection_slots: Vec<ResourceSummaryStableCollectionSlotStateEntry>,
    deferred: ResourceCheckDeferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum ResourceSummaryStableInitializedFunctionCheckEntryReject {
    AutoDropPoints,
    Place,
    Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum ResourceSummaryInitializedFunctionCheckEntryReprojectionReject {
    Place,
    PlaceType,
    CellStateType,
    CollectionSlotStateType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableCellStateEntry {
    place: ResourceSummaryStableResourcePlace,
    state: ResourceSummaryStableCellState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceSummaryStableCellState {
    Uninit,
    Initialized(ResourceSummaryStableTypeKey),
    Moved,
    Dropped,
    MaybeMoved,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableCollectionSlotStateEntry {
    slot: ResourceSummaryStableResourcePlace,
    state: ResourceSummaryStableCollectionSlotState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceSummaryStableCollectionSlotState {
    Uninitialized,
    Initialized(ResourceSummaryStableTypeKey),
    MaybeInitialized(Option<ResourceSummaryStableTypeKey>),
    Moved(ResourceSummaryStableTypeKey),
    Dropped(ResourceSummaryStableTypeKey),
    Released,
    MaybeReleased,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableResourcePlace {
    root: ResourceSummaryStableResourcePlaceRoot,
    projections: Vec<ResourceSummaryStablePlaceProjection>,
    ty: ResourceSummaryStableTypeKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceSummaryStableResourcePlaceRoot {
    Local(String),
    Temporary(usize),
    I32Constant(i32),
    Return,
    Storage(usize),
}

/// function-local resource id を stable entry 用 ordinal へ正規化する map。
///
/// `ResourceId` と `StorageId` は lowering session 内の割当値なので、stable cache value へ
/// 直接保存しない。Resource IR body hash と同じ考え方で、関数本文を決定的順序で走査し、
/// 最初に現れた順の ordinal だけを entry に保存する。replay 側では現在の同じ body から
/// ordinal を実 id へ戻すため、同じ本文でも arena/id 割当が変わる場合に stale id を
/// 直接使わずに済む。
#[derive(Debug, Default)]
struct ResourceFunctionPlaceOrdinalMap {
    temporary_ordinals: BTreeMap<ResourceId, usize>,
    temporaries: Vec<ResourceId>,
    storage_ordinals: BTreeMap<StorageId, usize>,
    storages: Vec<StorageId>,
    type_boundary: Vec<TypeId>,
}

impl ResourceFunctionPlaceOrdinalMap {
    fn new(function: &ResourceFunction) -> Self {
        let mut out = Self::default();
        for param in &function.params {
            out.record_place(&param.place);
        }
        for block in &function.blocks {
            out.record_ops(&block.ops);
            out.record_terminator(&block.terminator);
        }
        out
    }

    fn temporary_ordinal(&self, id: ResourceId) -> Option<usize> {
        self.temporary_ordinals.get(&id).copied()
    }

    fn temporary_id(&self, ordinal: usize) -> Option<ResourceId> {
        self.temporaries.get(ordinal).copied()
    }

    fn storage_ordinal(&self, id: StorageId) -> Option<usize> {
        self.storage_ordinals.get(&id).copied()
    }

    fn storage_id(&self, ordinal: usize) -> Option<StorageId> {
        self.storages.get(ordinal).copied()
    }

    fn type_boundary(&self) -> &[TypeId] {
        &self.type_boundary
    }

    fn record_type(&mut self, ty: TypeId) {
        if self.type_boundary.contains(&ty) {
            return;
        }
        self.type_boundary.push(ty);
    }

    fn record_temporary(&mut self, id: ResourceId) {
        if self.temporary_ordinals.contains_key(&id) {
            return;
        }
        let ordinal = self.temporaries.len();
        self.temporary_ordinals.insert(id, ordinal);
        self.temporaries.push(id);
    }

    fn record_storage(&mut self, id: StorageId) {
        if self.storage_ordinals.contains_key(&id) {
            return;
        }
        let ordinal = self.storages.len();
        self.storage_ordinals.insert(id, ordinal);
        self.storages.push(id);
    }

    fn record_place(&mut self, place: &Place) {
        self.record_type(place.ty);
        match place.root {
            PlaceRoot::Temporary(id) => self.record_temporary(id),
            PlaceRoot::Storage(id) => self.record_storage(id),
            PlaceRoot::Local(_)
            | PlaceRoot::I32Constant(_)
            | PlaceRoot::Return
            | PlaceRoot::Unknown => {}
        }
        for projection in &place.projections {
            self.record_projection(projection);
        }
    }

    fn record_projection(&mut self, projection: &PlaceProjection) {
        if let PlaceProjection::StorageOffset(offset) = projection {
            self.record_offset(offset);
        }
    }

    fn record_offset(&mut self, offset: &ResourceOffset) {
        match offset {
            ResourceOffset::Symbolic { place }
            | ResourceOffset::ScaledSymbolic { place, .. }
            | ResourceOffset::Offset { place, .. }
            | ResourceOffset::ScaledOffset { place, .. } => self.record_place(place),
            ResourceOffset::Known(_) | ResourceOffset::Unknown => {}
        }
    }

    fn record_condition_fact(&mut self, fact: &ResourceConditionFact) {
        match fact {
            ResourceConditionFact::EqZero { place }
            | ResourceConditionFact::NeZero { place }
            | ResourceConditionFact::Positive { place }
            | ResourceConditionFact::NonPositive { place }
            | ResourceConditionFact::Negative { place }
            | ResourceConditionFact::NonNegative { place } => self.record_place(place),
            ResourceConditionFact::I32Relation { left, right, .. } => {
                self.record_place(left);
                self.record_place(right);
            }
            ResourceConditionFact::Any(facts) | ResourceConditionFact::All(facts) => {
                for fact in facts {
                    self.record_condition_fact(fact);
                }
            }
        }
    }

    fn record_optional_condition_fact(&mut self, fact: &Option<ResourceConditionFact>) {
        if let Some(fact) = fact {
            self.record_condition_fact(fact);
        }
    }

    fn record_ops(&mut self, ops: &[ResourceOp]) {
        for op in ops {
            self.record_op(op);
        }
    }

    fn record_op(&mut self, op: &ResourceOp) {
        match op {
            ResourceOp::Expr {
                kind, output, ty, ..
            } => {
                if let ResourceExprKind::LayoutSizeOf(ty) = kind {
                    self.record_type(*ty);
                }
                self.record_place(output);
                self.record_type(*ty);
            }
            ResourceOp::FunctionValue { output, .. } | ResourceOp::RawMemory { output, .. } => {
                self.record_place(output)
            }
            ResourceOp::DeclareLocal {
                place, initializer, ..
            } => {
                self.record_place(place);
                if let Some(initializer) = initializer {
                    self.record_place(initializer);
                }
            }
            ResourceOp::Read { source, output, .. }
            | ResourceOp::Move { source, output, .. }
            | ResourceOp::Borrow { source, output, .. } => {
                self.record_place(source);
                self.record_place(output);
            }
            ResourceOp::Assign { target, value, .. } => {
                self.record_place(target);
                self.record_place(value);
            }
            ResourceOp::Drop { place, .. } => self.record_place(place),
            ResourceOp::EndScope { locals, result, .. } => {
                for local in locals {
                    self.record_place(local);
                }
                if let Some(result) = result {
                    self.record_place(result);
                }
            }
            ResourceOp::CallEffect { .. } => {}
            ResourceOp::Call {
                output,
                target,
                args,
                ..
            } => {
                self.record_place(output);
                self.record_call_target(target);
                for arg in args {
                    self.record_place(arg);
                }
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                params,
                result,
                args,
                ..
            } => {
                self.record_place(output);
                self.record_place(callee);
                for param in params {
                    self.record_type(*param);
                }
                self.record_type(*result);
                for arg in args {
                    self.record_place(arg);
                }
            }
            ResourceOp::RawAddressAlias { source, target, .. }
            | ResourceOp::RawAddressView { source, target, .. } => {
                self.record_place(source);
                self.record_place(target);
            }
            ResourceOp::StorageOrigin { target, .. } => self.record_place(target),
            ResourceOp::CollectionSlotLifecycle { target, event, .. } => {
                self.record_place(target);
                self.record_collection_slot_event(*event);
            }
            ResourceOp::CollectionStorageRelocate {
                old_storage,
                new_storage,
                ..
            } => {
                self.record_place(old_storage);
                self.record_place(new_storage);
            }
            ResourceOp::CollectionSlotDropTraversal {
                storage,
                initialized_count,
                expected_ty,
                ..
            } => {
                self.record_place(storage);
                self.record_place(initialized_count);
                self.record_type(*expected_ty);
            }
            ResourceOp::CollectionSlotTransformRange {
                source_storage,
                source_initialized_count,
                output_storage,
                output_initialized_count,
                expected_ty,
                ..
            } => {
                self.record_place(source_storage);
                self.record_place(source_initialized_count);
                self.record_place(output_storage);
                self.record_place(output_initialized_count);
                self.record_type(*expected_ty);
            }
            ResourceOp::Construct { output, inputs, .. } => {
                self.record_place(output);
                for input in inputs {
                    self.record_place(input);
                }
            }
            ResourceOp::Branch {
                output,
                condition,
                condition_fact,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } => {
                self.record_place(output);
                self.record_place(condition);
                self.record_optional_condition_fact(condition_fact);
                self.record_ops(then_ops);
                self.record_place(then_value);
                self.record_ops(else_ops);
                self.record_place(else_value);
            }
            ResourceOp::Loop {
                condition_ops,
                condition,
                condition_fact,
                body_ops,
                ..
            } => {
                self.record_ops(condition_ops);
                self.record_place(condition);
                self.record_optional_condition_fact(condition_fact);
                self.record_ops(body_ops);
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                ..
            } => {
                self.record_place(output);
                self.record_place(scrutinee);
                for arm in arms {
                    self.record_match_arm(arm);
                }
            }
        }
    }

    fn record_match_arm(&mut self, arm: &ResourceMatchArm) {
        if let Some(bind_local) = &arm.bind_local {
            self.record_place(bind_local);
        }
        self.record_ops(&arm.ops);
        self.record_place(&arm.value);
    }

    fn record_call_target(&mut self, target: &ResourceCallTarget) {
        match target {
            ResourceCallTarget::Builtin { .. } => {}
            ResourceCallTarget::User { type_args, .. } => {
                for type_arg in type_args {
                    self.record_type(*type_arg);
                }
            }
            ResourceCallTarget::Trait {
                application,
                self_ty,
                ..
            } => {
                for arg in &application.args {
                    self.record_type(*arg);
                }
                self.record_type(*self_ty);
            }
        }
    }

    fn record_collection_slot_event(&mut self, event: CollectionSlotLifecycleEvent) {
        match event {
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty }
            | CollectionSlotLifecycleEvent::BorrowRead {
                expected_ty: value_ty,
            }
            | CollectionSlotLifecycleEvent::MoveOut {
                expected_ty: value_ty,
            }
            | CollectionSlotLifecycleEvent::DropInitialized {
                expected_ty: value_ty,
            }
            | CollectionSlotLifecycleEvent::StorageDealloc { value_ty } => {
                self.record_type(value_ty);
            }
            CollectionSlotLifecycleEvent::ReplaceInitialized { old_ty, new_ty, .. } => {
                self.record_type(old_ty);
                self.record_type(new_ty);
            }
        }
    }

    fn record_terminator(&mut self, terminator: &ResourceTerminator) {
        match terminator {
            ResourceTerminator::Return { value, .. } => {
                if let Some(value) = value {
                    self.record_place(value);
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }
}

/// raw initialization summary の complete leaf entry。
///
/// `RawCellInitializationFunctionSummary` に含まれる return facts、parameter byte-range、
/// variant 条件、path-sensitive release を同じ entry にまとめて保存する。部分的に保存した
/// summary は replay 後に raw initialization proof を欠落させるため、この entry は
/// leaf summary surface 全体を再投影できる場合だけ cache value として採用する。
/// `TypeId` は stable type key、projection は layout を検証できる形式へ落として保持する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ResourceSummaryStableRawInitCompleteLeafEntry {
    return_cells: Vec<ResourceSummaryStableRawInitReturnCell>,
    return_byte_ranges: Vec<ResourceSummaryStableRawInitReturnByteRange>,
    param_cells: Vec<ResourceSummaryStableRawInitParamCell>,
    param_byte_ranges: Vec<ResourceSummaryStableRawInitParamByteRange>,
    param_release_requirements: Vec<ResourceSummaryStableRawCellReleaseParamRequirement>,
    variant_param_cells: Vec<ResourceSummaryStableRawInitVariantParamCell>,
    variant_param_byte_ranges: Vec<ResourceSummaryStableRawInitVariantParamByteRange>,
    variant_required_param_cells: Vec<ResourceSummaryStableRawInitVariantParamRequirement>,
    variant_conditions: Vec<ResourceSummaryStableRawInitVariantCondition>,
}

impl ResourceSummaryStableRawInitCompleteLeafEntry {
    pub(super) fn len(&self) -> usize {
        self.return_cells.len()
            + self.return_byte_ranges.len()
            + self.param_cells.len()
            + self.param_byte_ranges.len()
            + self.param_release_requirements.len()
            + self.variant_param_cells.len()
            + self.variant_param_byte_ranges.len()
            + self.variant_required_param_cells.len()
            + self.variant_conditions.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum ResourceSummaryStableRawInitCompleteLeafEntryReject {
    Surface,
    ParamCellProjection,
    ParamCellType,
    ParamReleaseRequirementType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum ResourceSummaryRawInitCompleteLeafEntryReprojectionReject {
    EmptyEntry,
    ParamCellProjection,
    ParamCellStableType,
    ParamCellResultType,
    ParamReleaseRequirementProjection,
    ParamReleaseRequirementType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum ResourceSummaryStableI32ScalarReturnFactsEntryReject {
    ReturnProjection,
    ParameterProjection,
    ScalarType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject {
    ReturnProjection,
    ParameterProjection,
    ScalarType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableI32ScalarReturnAlias {
    return_projection: Vec<ResourceSummaryStablePlaceProjection>,
    parameter_index: usize,
    parameter_projection: Vec<ResourceSummaryStablePlaceProjection>,
    scalar_ty: ResourceSummaryStableTypeKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableI32ScalarReturnOffset {
    return_projection: Vec<ResourceSummaryStablePlaceProjection>,
    parameter_index: usize,
    parameter_projection: Vec<ResourceSummaryStablePlaceProjection>,
    scalar_ty: ResourceSummaryStableTypeKey,
    offset: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableI32ScalarReturnRelation {
    left_return_projection: Vec<ResourceSummaryStablePlaceProjection>,
    op: ResourceI32RelationOp,
    right_return_projection: Vec<ResourceSummaryStablePlaceProjection>,
    scalar_ty: ResourceSummaryStableTypeKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableI32ScalarReturnConstant {
    return_projection: Vec<ResourceSummaryStablePlaceProjection>,
    scalar_ty: ResourceSummaryStableTypeKey,
    value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableI32ScalarReturnCondition {
    return_projection: Vec<ResourceSummaryStablePlaceProjection>,
    scalar_ty: ResourceSummaryStableTypeKey,
    condition: I32ValueCondition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableI32ScalarParameterCondition {
    parameter_index: usize,
    parameter_projection: Vec<ResourceSummaryStablePlaceProjection>,
    scalar_ty: ResourceSummaryStableTypeKey,
    condition: I32ValueCondition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableRawInitParamCell {
    param_index: usize,
    suffix: Vec<ResourceSummaryStableProjection>,
    ty: ResourceSummaryStableTypeKey,
    holds_raw_address: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableRawInitReturnCell {
    suffix: Vec<ResourceSummaryStablePlaceProjection>,
    ty: ResourceSummaryStableTypeKey,
    holds_raw_address: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableRawInitReturnByteRange {
    address_suffix: Vec<ResourceSummaryStablePlaceProjection>,
    address_ty: ResourceSummaryStableTypeKey,
    count: ResourceSummaryStableRawInitReturnCount,
    unit: super::super::cell_state_raw_range::InitializedRawRangeUnit,
    ty: ResourceSummaryStableTypeKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceSummaryStableRawInitReturnCount {
    ReturnValueProjection {
        suffix: Vec<ResourceSummaryStablePlaceProjection>,
        ty: ResourceSummaryStableTypeKey,
    },
    KnownI32 {
        value: i32,
        ty: ResourceSummaryStableTypeKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableRawInitParamByteRange {
    address_param_index: usize,
    address_suffix: Vec<ResourceSummaryStableProjection>,
    address_ty: ResourceSummaryStableTypeKey,
    count: ResourceSummaryStableRawInitParamCount,
    unit: super::super::cell_state_raw_range::InitializedRawRangeUnit,
    ty: ResourceSummaryStableTypeKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ResourceSummaryStableRawInitParamCount {
    ParamProjection {
        param_index: usize,
        suffix: Vec<ResourceSummaryStableProjection>,
        ty: ResourceSummaryStableTypeKey,
    },
    KnownI32 {
        value: i32,
        ty: ResourceSummaryStableTypeKey,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableRawInitVariantParamCell {
    variant: String,
    param_index: usize,
    suffix: Vec<ResourceSummaryStableProjection>,
    ty: ResourceSummaryStableTypeKey,
    holds_raw_address: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableRawInitVariantParamByteRange {
    variant: String,
    address_param_index: usize,
    address_suffix: Vec<ResourceSummaryStableProjection>,
    address_ty: ResourceSummaryStableTypeKey,
    count: ResourceSummaryStableRawInitParamCount,
    unit: super::super::cell_state_raw_range::InitializedRawRangeUnit,
    ty: ResourceSummaryStableTypeKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableRawInitVariantParamRequirement {
    variant: String,
    param_index: usize,
    suffix: Vec<ResourceSummaryStableProjection>,
    ty: ResourceSummaryStableTypeKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResourceSummaryStableRawInitVariantCondition {
    variant: String,
    param_index: usize,
    suffix: Vec<ResourceSummaryStableProjection>,
    ty: ResourceSummaryStableTypeKey,
    condition: RawCellValueCondition,
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

    pub(super) fn new_for_initialized_function_check(
        types: &'a TypeCtx,
        function: &'a ResourceFunction,
        type_params: &[TypeId],
    ) -> Option<Self> {
        let mut out = Self::new(types, function, type_params)?;
        let body_boundary = ResourceFunctionPlaceOrdinalMap::new(function);
        for ty in body_boundary.type_boundary() {
            out.insert_type(*ty)?;
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

pub(super) fn stable_i32_scalar_return_facts_entry(
    types: &TypeCtx,
    function: &ResourceFunction,
    facts: &I32ScalarReturnFacts,
) -> Result<
    ResourceSummaryStableI32ScalarReturnFactsEntry,
    ResourceSummaryStableI32ScalarReturnFactsEntryReject,
> {
    Ok(ResourceSummaryStableI32ScalarReturnFactsEntry {
        aliases: facts
            .aliases
            .iter()
            .map(|fact| stable_i32_scalar_return_alias(types, function, fact))
            .collect::<Result<Vec<_>, _>>()?,
        offsets: facts
            .offsets
            .iter()
            .map(|fact| stable_i32_scalar_return_offset(types, function, fact))
            .collect::<Result<Vec<_>, _>>()?,
        relations: facts
            .relations
            .iter()
            .map(|fact| stable_i32_scalar_return_relation(types, function, fact))
            .collect::<Result<Vec<_>, _>>()?,
        constants: facts
            .constants
            .iter()
            .map(|fact| stable_i32_scalar_return_constant(types, function, fact))
            .collect::<Result<Vec<_>, _>>()?,
        return_conditions: facts
            .return_conditions
            .iter()
            .map(|fact| stable_i32_scalar_return_condition(types, function, fact))
            .collect::<Result<Vec<_>, _>>()?,
        parameter_conditions: facts
            .parameter_conditions
            .iter()
            .map(|fact| stable_i32_scalar_parameter_condition(types, function, fact))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub(super) fn reproject_i32_scalar_return_facts_entry(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    entry: &ResourceSummaryStableI32ScalarReturnFactsEntry,
) -> Option<I32ScalarReturnFacts> {
    reproject_i32_scalar_return_facts_entry_result(ctx, entry).ok()
}

pub(super) fn reproject_i32_scalar_return_facts_entry_result(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    entry: &ResourceSummaryStableI32ScalarReturnFactsEntry,
) -> Result<I32ScalarReturnFacts, ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject> {
    Ok(I32ScalarReturnFacts {
        aliases: entry
            .aliases
            .iter()
            .map(|fact| reproject_i32_scalar_return_alias(ctx, fact))
            .collect::<Result<Vec<_>, _>>()?,
        offsets: entry
            .offsets
            .iter()
            .map(|fact| reproject_i32_scalar_return_offset(ctx, fact))
            .collect::<Result<Vec<_>, _>>()?,
        relations: entry
            .relations
            .iter()
            .map(|fact| reproject_i32_scalar_return_relation(ctx, fact))
            .collect::<Result<Vec<_>, _>>()?,
        constants: entry
            .constants
            .iter()
            .map(|fact| reproject_i32_scalar_return_constant(ctx, fact))
            .collect::<Result<Vec<_>, _>>()?,
        return_conditions: entry
            .return_conditions
            .iter()
            .map(|fact| reproject_i32_scalar_return_condition(ctx, fact))
            .collect::<Result<Vec<_>, _>>()?,
        parameter_conditions: entry
            .parameter_conditions
            .iter()
            .map(|fact| reproject_i32_scalar_parameter_condition(ctx, fact))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

pub(super) fn stable_initialized_function_check_entry(
    types: &TypeCtx,
    function: &ResourceFunction,
    check: &ResourceFunctionCheck,
) -> Result<
    ResourceSummaryStableInitializedFunctionCheckEntry,
    ResourceSummaryStableInitializedFunctionCheckEntryReject,
> {
    if !check.auto_drop_points.is_empty() {
        return Err(ResourceSummaryStableInitializedFunctionCheckEntryReject::AutoDropPoints);
    }
    let place_ordinals = ResourceFunctionPlaceOrdinalMap::new(function);
    Ok(ResourceSummaryStableInitializedFunctionCheckEntry {
        final_cells: check
            .final_cells
            .iter()
            .map(|entry| stable_cell_state_entry(types, function, &place_ordinals, entry))
            .collect::<Result<Vec<_>, _>>()?,
        final_collection_slots: check
            .final_collection_slots
            .iter()
            .map(|entry| {
                stable_collection_slot_state_entry(types, function, &place_ordinals, entry)
            })
            .collect::<Result<Vec<_>, _>>()?,
        deferred: check.deferred,
    })
}

pub(super) fn reproject_initialized_function_check_entry(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    function_name: &str,
    entry: &ResourceSummaryStableInitializedFunctionCheckEntry,
) -> Option<ResourceFunctionCheck> {
    reproject_initialized_function_check_entry_result(ctx, function_name, entry).ok()
}

pub(super) fn reproject_initialized_function_check_entry_result(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    function_name: &str,
    entry: &ResourceSummaryStableInitializedFunctionCheckEntry,
) -> Result<ResourceFunctionCheck, ResourceSummaryInitializedFunctionCheckEntryReprojectionReject> {
    let place_ordinals = ResourceFunctionPlaceOrdinalMap::new(ctx.function);
    Ok(ResourceFunctionCheck {
        name: function_name.to_string(),
        final_cells: entry
            .final_cells
            .iter()
            .map(|entry| reproject_cell_state_entry(ctx, &place_ordinals, entry))
            .collect::<Result<Vec<_>, _>>()?,
        final_collection_slots: entry
            .final_collection_slots
            .iter()
            .map(|entry| reproject_collection_slot_state_entry(ctx, &place_ordinals, entry))
            .collect::<Result<Vec<_>, _>>()?,
        auto_drop_points: Vec::new(),
        deferred: entry.deferred,
    })
}

fn stable_cell_state_entry(
    types: &TypeCtx,
    function: &ResourceFunction,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    entry: &CellStateEntry,
) -> Result<
    ResourceSummaryStableCellStateEntry,
    ResourceSummaryStableInitializedFunctionCheckEntryReject,
> {
    Ok(ResourceSummaryStableCellStateEntry {
        place: stable_resource_place(types, function, place_ordinals, &entry.place)?,
        state: stable_cell_state(types, entry.state.clone())?,
    })
}

fn stable_cell_state(
    types: &TypeCtx,
    state: CellState,
) -> Result<ResourceSummaryStableCellState, ResourceSummaryStableInitializedFunctionCheckEntryReject>
{
    Ok(match state {
        CellState::Uninit => ResourceSummaryStableCellState::Uninit,
        CellState::Initialized(ty) => ResourceSummaryStableCellState::Initialized(
            ResourceSummaryStableTypeKey::from_type(types, ty)
                .ok_or(ResourceSummaryStableInitializedFunctionCheckEntryReject::Type)?,
        ),
        CellState::Moved => ResourceSummaryStableCellState::Moved,
        CellState::Dropped => ResourceSummaryStableCellState::Dropped,
        CellState::MaybeMoved => ResourceSummaryStableCellState::MaybeMoved,
    })
}

fn stable_collection_slot_state_entry(
    types: &TypeCtx,
    function: &ResourceFunction,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    entry: &CollectionSlotStateEntry,
) -> Result<
    ResourceSummaryStableCollectionSlotStateEntry,
    ResourceSummaryStableInitializedFunctionCheckEntryReject,
> {
    Ok(ResourceSummaryStableCollectionSlotStateEntry {
        slot: stable_resource_place(types, function, place_ordinals, &entry.slot)?,
        state: stable_collection_slot_state(types, entry.state)?,
    })
}

fn stable_collection_slot_state(
    types: &TypeCtx,
    state: CollectionSlotState,
) -> Result<
    ResourceSummaryStableCollectionSlotState,
    ResourceSummaryStableInitializedFunctionCheckEntryReject,
> {
    Ok(match state {
        CollectionSlotState::Uninitialized => {
            ResourceSummaryStableCollectionSlotState::Uninitialized
        }
        CollectionSlotState::Initialized(ty) => {
            ResourceSummaryStableCollectionSlotState::Initialized(
                ResourceSummaryStableTypeKey::from_type(types, ty)
                    .ok_or(ResourceSummaryStableInitializedFunctionCheckEntryReject::Type)?,
            )
        }
        CollectionSlotState::MaybeInitialized(ty) => {
            ResourceSummaryStableCollectionSlotState::MaybeInitialized(
                ty.map(|ty| {
                    ResourceSummaryStableTypeKey::from_type(types, ty)
                        .ok_or(ResourceSummaryStableInitializedFunctionCheckEntryReject::Type)
                })
                .transpose()?,
            )
        }
        CollectionSlotState::Moved(ty) => ResourceSummaryStableCollectionSlotState::Moved(
            ResourceSummaryStableTypeKey::from_type(types, ty)
                .ok_or(ResourceSummaryStableInitializedFunctionCheckEntryReject::Type)?,
        ),
        CollectionSlotState::Dropped(ty) => ResourceSummaryStableCollectionSlotState::Dropped(
            ResourceSummaryStableTypeKey::from_type(types, ty)
                .ok_or(ResourceSummaryStableInitializedFunctionCheckEntryReject::Type)?,
        ),
        CollectionSlotState::Released => ResourceSummaryStableCollectionSlotState::Released,
        CollectionSlotState::MaybeReleased => {
            ResourceSummaryStableCollectionSlotState::MaybeReleased
        }
    })
}

fn stable_resource_place(
    types: &TypeCtx,
    function: &ResourceFunction,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    place: &Place,
) -> Result<
    ResourceSummaryStableResourcePlace,
    ResourceSummaryStableInitializedFunctionCheckEntryReject,
> {
    Ok(ResourceSummaryStableResourcePlace {
        root: stable_resource_place_root(place_ordinals, &place.root)?,
        projections: place
            .projections
            .iter()
            .map(|projection| {
                stable_resource_place_projection(types, function, place_ordinals, projection)
            })
            .collect::<Result<Vec<_>, _>>()?,
        ty: ResourceSummaryStableTypeKey::from_type(types, place.ty)
            .ok_or(ResourceSummaryStableInitializedFunctionCheckEntryReject::Type)?,
    })
}

fn stable_resource_place_root(
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    root: &PlaceRoot,
) -> Result<
    ResourceSummaryStableResourcePlaceRoot,
    ResourceSummaryStableInitializedFunctionCheckEntryReject,
> {
    Ok(match root {
        PlaceRoot::Local(name) => ResourceSummaryStableResourcePlaceRoot::Local(name.clone()),
        PlaceRoot::Temporary(id) => ResourceSummaryStableResourcePlaceRoot::Temporary(
            place_ordinals
                .temporary_ordinal(*id)
                .ok_or(ResourceSummaryStableInitializedFunctionCheckEntryReject::Place)?,
        ),
        PlaceRoot::I32Constant(value) => {
            ResourceSummaryStableResourcePlaceRoot::I32Constant(*value)
        }
        PlaceRoot::Return => ResourceSummaryStableResourcePlaceRoot::Return,
        PlaceRoot::Storage(id) => ResourceSummaryStableResourcePlaceRoot::Storage(
            place_ordinals
                .storage_ordinal(*id)
                .ok_or(ResourceSummaryStableInitializedFunctionCheckEntryReject::Place)?,
        ),
        PlaceRoot::Unknown => {
            return Err(ResourceSummaryStableInitializedFunctionCheckEntryReject::Place);
        }
    })
}

fn stable_resource_place_projection(
    types: &TypeCtx,
    function: &ResourceFunction,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    projection: &PlaceProjection,
) -> Result<
    ResourceSummaryStablePlaceProjection,
    ResourceSummaryStableInitializedFunctionCheckEntryReject,
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
            ResourceSummaryStablePlaceProjection::StorageOffset(stable_resource_place_offset(
                types,
                function,
                place_ordinals,
                offset,
            )?)
        }
    })
}

fn stable_resource_place_offset(
    types: &TypeCtx,
    function: &ResourceFunction,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    offset: &ResourceOffset,
) -> Result<ResourceSummaryStableOffset, ResourceSummaryStableInitializedFunctionCheckEntryReject> {
    Ok(match offset {
        ResourceOffset::Known(value) => ResourceSummaryStableOffset::Known(*value),
        ResourceOffset::Symbolic { place } => {
            stable_resource_symbolic_offset(types, function, place_ordinals, place)?
        }
        ResourceOffset::ScaledSymbolic { place, scale } => {
            stable_resource_scaled_symbolic_offset(types, function, place_ordinals, place, *scale)?
        }
        ResourceOffset::Offset { place, offset } => {
            stable_resource_shifted_offset(types, function, place_ordinals, place, *offset)?
        }
        ResourceOffset::ScaledOffset {
            place,
            offset,
            scale,
        } => stable_resource_scaled_shifted_offset(
            types,
            function,
            place_ordinals,
            place,
            *offset,
            *scale,
        )?,
        ResourceOffset::Unknown => ResourceSummaryStableOffset::Unknown,
    })
}

fn stable_resource_symbolic_offset(
    types: &TypeCtx,
    function: &ResourceFunction,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    place: &Place,
) -> Result<ResourceSummaryStableOffset, ResourceSummaryStableInitializedFunctionCheckEntryReject> {
    Ok(
        match stable_resource_offset_place_for_function(types, function, place_ordinals, place)? {
            ResourceSummaryStableOffsetPlace::Parameter(place) => {
                ResourceSummaryStableOffset::Symbolic { place }
            }
            ResourceSummaryStableOffsetPlace::Resource(place) => {
                ResourceSummaryStableOffset::ResourceSymbolic { place }
            }
        },
    )
}

fn stable_resource_scaled_symbolic_offset(
    types: &TypeCtx,
    function: &ResourceFunction,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    place: &Place,
    scale: usize,
) -> Result<ResourceSummaryStableOffset, ResourceSummaryStableInitializedFunctionCheckEntryReject> {
    Ok(
        match stable_resource_offset_place_for_function(types, function, place_ordinals, place)? {
            ResourceSummaryStableOffsetPlace::Parameter(place) => {
                ResourceSummaryStableOffset::ScaledSymbolic { place, scale }
            }
            ResourceSummaryStableOffsetPlace::Resource(place) => {
                ResourceSummaryStableOffset::ResourceScaledSymbolic { place, scale }
            }
        },
    )
}

fn stable_resource_shifted_offset(
    types: &TypeCtx,
    function: &ResourceFunction,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    place: &Place,
    offset: i64,
) -> Result<ResourceSummaryStableOffset, ResourceSummaryStableInitializedFunctionCheckEntryReject> {
    Ok(
        match stable_resource_offset_place_for_function(types, function, place_ordinals, place)? {
            ResourceSummaryStableOffsetPlace::Parameter(place) => {
                ResourceSummaryStableOffset::Offset { place, offset }
            }
            ResourceSummaryStableOffsetPlace::Resource(place) => {
                ResourceSummaryStableOffset::ResourceOffset { place, offset }
            }
        },
    )
}

fn stable_resource_scaled_shifted_offset(
    types: &TypeCtx,
    function: &ResourceFunction,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    place: &Place,
    offset: i64,
    scale: usize,
) -> Result<ResourceSummaryStableOffset, ResourceSummaryStableInitializedFunctionCheckEntryReject> {
    Ok(
        match stable_resource_offset_place_for_function(types, function, place_ordinals, place)? {
            ResourceSummaryStableOffsetPlace::Parameter(place) => {
                ResourceSummaryStableOffset::ScaledOffset {
                    place,
                    offset,
                    scale,
                }
            }
            ResourceSummaryStableOffsetPlace::Resource(place) => {
                ResourceSummaryStableOffset::ResourceScaledOffset {
                    place,
                    offset,
                    scale,
                }
            }
        },
    )
}

fn stable_resource_offset_place_for_function(
    types: &TypeCtx,
    function: &ResourceFunction,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    place: &Place,
) -> Result<
    ResourceSummaryStableOffsetPlace,
    ResourceSummaryStableInitializedFunctionCheckEntryReject,
> {
    if let Some(place) = stable_resource_offset_place(types, &function.params, place) {
        return Ok(ResourceSummaryStableOffsetPlace::Parameter(Box::new(place)));
    }
    stable_resource_place(types, function, place_ordinals, place)
        .map(|place| ResourceSummaryStableOffsetPlace::Resource(Box::new(place)))
}

fn reproject_cell_state_entry(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    entry: &ResourceSummaryStableCellStateEntry,
) -> Result<CellStateEntry, ResourceSummaryInitializedFunctionCheckEntryReprojectionReject> {
    Ok(CellStateEntry {
        place: reproject_resource_place(ctx, place_ordinals, &entry.place)?,
        state: reproject_cell_state(ctx, &entry.state)?,
    })
}

fn reproject_cell_state(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    state: &ResourceSummaryStableCellState,
) -> Result<CellState, ResourceSummaryInitializedFunctionCheckEntryReprojectionReject> {
    Ok(match state {
        ResourceSummaryStableCellState::Uninit => CellState::Uninit,
        ResourceSummaryStableCellState::Initialized(ty) => {
            CellState::Initialized(ctx.reproject_type(ty).ok_or(
                ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::CellStateType,
            )?)
        }
        ResourceSummaryStableCellState::Moved => CellState::Moved,
        ResourceSummaryStableCellState::Dropped => CellState::Dropped,
        ResourceSummaryStableCellState::MaybeMoved => CellState::MaybeMoved,
    })
}

fn reproject_collection_slot_state_entry(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    entry: &ResourceSummaryStableCollectionSlotStateEntry,
) -> Result<CollectionSlotStateEntry, ResourceSummaryInitializedFunctionCheckEntryReprojectionReject>
{
    Ok(CollectionSlotStateEntry {
        slot: reproject_resource_place(ctx, place_ordinals, &entry.slot)?,
        state: reproject_collection_slot_state(ctx, &entry.state)?,
    })
}

fn reproject_collection_slot_state(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    state: &ResourceSummaryStableCollectionSlotState,
) -> Result<CollectionSlotState, ResourceSummaryInitializedFunctionCheckEntryReprojectionReject> {
    Ok(match state {
        ResourceSummaryStableCollectionSlotState::Uninitialized => {
            CollectionSlotState::Uninitialized
        }
        ResourceSummaryStableCollectionSlotState::Initialized(ty) => {
            CollectionSlotState::Initialized(
                ctx.reproject_type(ty)
                    .ok_or(
                        ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::CollectionSlotStateType,
                    )?,
            )
        }
        ResourceSummaryStableCollectionSlotState::MaybeInitialized(ty) => {
            CollectionSlotState::MaybeInitialized(
                ty.as_ref()
                    .map(|ty| {
                        ctx.reproject_type(ty).ok_or(
                            ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::CollectionSlotStateType,
                        )
                    })
                    .transpose()?,
            )
        }
        ResourceSummaryStableCollectionSlotState::Moved(ty) => CollectionSlotState::Moved(
            ctx.reproject_type(ty)
                .ok_or(
                    ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::CollectionSlotStateType,
                )?,
        ),
        ResourceSummaryStableCollectionSlotState::Dropped(ty) => CollectionSlotState::Dropped(
            ctx.reproject_type(ty)
                .ok_or(
                    ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::CollectionSlotStateType,
                )?,
        ),
        ResourceSummaryStableCollectionSlotState::Released => CollectionSlotState::Released,
        ResourceSummaryStableCollectionSlotState::MaybeReleased => {
            CollectionSlotState::MaybeReleased
        }
    })
}

fn reproject_resource_place(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    place: &ResourceSummaryStableResourcePlace,
) -> Result<Place, ResourceSummaryInitializedFunctionCheckEntryReprojectionReject> {
    let root = reproject_resource_place_root(place_ordinals, &place.root)?;
    let ty = ctx
        .reproject_type(&place.ty)
        .ok_or(ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::PlaceType)?;
    if let Some(base_ty) = reproject_resource_place_root_base_type(ctx, &root) {
        let projections = match reproject_place_projection_suffix(ctx, base_ty, &place.projections)
        {
            Some((projections, projected_ty)) => {
                if !place.ty.matches_type(ctx.types, projected_ty) {
                    // final check entry は、body hash と stable place surface が一致する場合にだけ
                    // replay される。projection の layout 自体を検証でき、保存済み place type が
                    // 現在 boundary へ戻せているなら、その保存済み型を final state の proof
                    // surface として採用する。これは Resource IR の final state が持っていた
                    // checked place type を replay するための境界であり、TypeCtx 全体検索で
                    // 類似型を探す緩和ではない。place 型そのものを再投影できない場合は、この
                    // 分岐に到達する前に fail-closed で reject される。
                    return Ok(Place {
                        root,
                        projections,
                        ty,
                    });
                }
                projections
            }
            None => {
                reproject_resource_place_projection_suffix_without_layout(ctx, &place.projections)?
            }
        };
        return Ok(Place {
            root,
            projections,
            ty,
        });
    }

    Ok(Place {
        root,
        projections: place
            .projections
            .iter()
            .map(|projection| {
                reproject_place_projection(ctx, projection)
                    .ok_or(ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::Place)
            })
            .collect::<Result<Vec<_>, _>>()?,
        ty,
    })
}

/// final check 用の stable place projection を layout 再計算なしで戻す。
///
/// final check entry の key には Resource IR body hash が含まれるため、同じ body に
/// 対する replay では projection 列そのものが proof surface の一部である。generic や
/// raw storage view のように現在の `TypeCtx` だけでは field layout を再計算できない場合も、
/// stable entry に保存済みの projection をそのまま戻し、offset 内 place の再投影に失敗
/// した場合だけ安全側で reject する。
fn reproject_resource_place_projection_suffix_without_layout(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    suffix: &[ResourceSummaryStablePlaceProjection],
) -> Result<Vec<PlaceProjection>, ResourceSummaryInitializedFunctionCheckEntryReprojectionReject> {
    suffix
        .iter()
        .map(|projection| {
            reproject_place_projection(ctx, projection)
                .ok_or(ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::Place)
        })
        .collect()
}

fn reproject_resource_place_root(
    place_ordinals: &ResourceFunctionPlaceOrdinalMap,
    root: &ResourceSummaryStableResourcePlaceRoot,
) -> Result<PlaceRoot, ResourceSummaryInitializedFunctionCheckEntryReprojectionReject> {
    Ok(match root {
        ResourceSummaryStableResourcePlaceRoot::Local(name) => PlaceRoot::Local(name.clone()),
        ResourceSummaryStableResourcePlaceRoot::Temporary(ordinal) => PlaceRoot::Temporary(
            place_ordinals
                .temporary_id(*ordinal)
                .ok_or(ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::Place)?,
        ),
        ResourceSummaryStableResourcePlaceRoot::I32Constant(value) => {
            PlaceRoot::I32Constant(*value)
        }
        ResourceSummaryStableResourcePlaceRoot::Return => PlaceRoot::Return,
        ResourceSummaryStableResourcePlaceRoot::Storage(ordinal) => PlaceRoot::Storage(
            place_ordinals
                .storage_id(*ordinal)
                .ok_or(ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::Place)?,
        ),
    })
}

fn reproject_resource_place_root_base_type(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    root: &PlaceRoot,
) -> Option<TypeId> {
    match root {
        PlaceRoot::Local(name) => ctx
            .function
            .params
            .iter()
            .find(|param| &param.name == name)
            .map(|param| param.ty),
        PlaceRoot::I32Constant(_) => Some(ctx.types.i32()),
        PlaceRoot::Return => Some(ctx.function.result),
        PlaceRoot::Temporary(_) | PlaceRoot::Storage(_) | PlaceRoot::Unknown => None,
    }
}

pub(super) fn stable_raw_init_complete_leaf_entry(
    types: &TypeCtx,
    function: &ResourceFunction,
    summary: &RawCellInitializationFunctionSummary,
) -> Result<
    ResourceSummaryStableRawInitCompleteLeafEntry,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    if raw_init_summary_fact_count(summary) == 0 {
        return Err(ResourceSummaryStableRawInitCompleteLeafEntryReject::Surface);
    }
    let return_cells = summary
        .return_cells
        .iter()
        .map(|cell| stable_raw_init_return_cell(types, &function.params, cell))
        .collect::<Result<Vec<_>, _>>()?;
    let return_byte_ranges = summary
        .return_byte_ranges
        .iter()
        .map(|range| stable_raw_init_return_byte_range(types, &function.params, range))
        .collect::<Result<Vec<_>, _>>()?;
    let param_cells = summary
        .param_cells
        .iter()
        .map(|cell| stable_raw_init_param_cell(types, cell))
        .collect::<Result<Vec<_>, _>>()?;
    let param_byte_ranges = summary
        .param_byte_ranges
        .iter()
        .map(|range| stable_raw_init_param_byte_range(types, range))
        .collect::<Result<Vec<_>, _>>()?;
    let param_release_requirements = summary
        .param_release_requirements
        .iter()
        .map(|requirement| {
            stable_raw_cell_release_param_requirement(types, &function.params, requirement)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let variant_param_cells = summary
        .variant_param_cells
        .iter()
        .map(|cell| stable_raw_init_variant_param_cell(types, cell))
        .collect::<Result<Vec<_>, _>>()?;
    let variant_param_byte_ranges = summary
        .variant_param_byte_ranges
        .iter()
        .map(|range| stable_raw_init_variant_param_byte_range(types, range))
        .collect::<Result<Vec<_>, _>>()?;
    let variant_required_param_cells = summary
        .variant_required_param_cells
        .iter()
        .map(|cell| stable_raw_init_variant_param_requirement(types, cell))
        .collect::<Result<Vec<_>, _>>()?;
    let variant_conditions = summary
        .variant_conditions
        .iter()
        .map(|condition| stable_raw_init_variant_condition(types, condition))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ResourceSummaryStableRawInitCompleteLeafEntry {
        return_cells,
        return_byte_ranges,
        param_cells,
        param_byte_ranges,
        param_release_requirements,
        variant_param_cells,
        variant_param_byte_ranges,
        variant_required_param_cells,
        variant_conditions,
    })
}

pub(super) fn reproject_raw_init_complete_leaf_entry(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    function_name: &str,
    entry: &ResourceSummaryStableRawInitCompleteLeafEntry,
) -> Option<RawCellInitializationFunctionSummary> {
    reproject_raw_init_complete_leaf_entry_result(ctx, function_name, entry).ok()
}

/// stable raw-init param facts entry を現在の Resource IR summary に戻す。
///
/// この関数は cache 候補の自己再投影検査でも使うため、`Option` で失敗を潰さず、
/// projection の不一致と型 key の不一致を分けて返す。再投影できない entry は安全側で
/// store/replay しないが、失敗面を分けることで次の canonicalization 対象を測定できる。
pub(super) fn reproject_raw_init_complete_leaf_entry_result(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    function_name: &str,
    entry: &ResourceSummaryStableRawInitCompleteLeafEntry,
) -> Result<
    RawCellInitializationFunctionSummary,
    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
> {
    if entry.len() == 0 {
        return Err(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::EmptyEntry);
    }
    Ok(RawCellInitializationFunctionSummary {
        function: function_name.to_string(),
        type_params: ctx.type_params.clone(),
        return_cells: entry
            .return_cells
            .iter()
            .map(|cell| reproject_raw_init_return_cell(ctx, cell))
            .collect::<Result<Vec<_>, _>>()?,
        return_byte_ranges: entry
            .return_byte_ranges
            .iter()
            .map(|range| reproject_raw_init_return_byte_range(ctx, range))
            .collect::<Result<Vec<_>, _>>()?,
        param_cells: entry
            .param_cells
            .iter()
            .map(|cell| reproject_raw_init_param_cell(ctx, cell))
            .collect::<Result<Vec<_>, _>>()?,
        param_byte_ranges: entry
            .param_byte_ranges
            .iter()
            .map(|range| reproject_raw_init_param_byte_range(ctx, range))
            .collect::<Result<Vec<_>, _>>()?,
        param_release_requirements: entry
            .param_release_requirements
            .iter()
            .map(|requirement| reproject_raw_cell_release_param_requirement(ctx, requirement))
            .collect::<Result<Vec<_>, _>>()?,
        variant_param_cells: entry
            .variant_param_cells
            .iter()
            .map(|cell| reproject_raw_init_variant_param_cell(ctx, cell))
            .collect::<Result<Vec<_>, _>>()?,
        variant_param_byte_ranges: entry
            .variant_param_byte_ranges
            .iter()
            .map(|range| reproject_raw_init_variant_param_byte_range(ctx, range))
            .collect::<Result<Vec<_>, _>>()?,
        variant_required_param_cells: entry
            .variant_required_param_cells
            .iter()
            .map(|cell| reproject_raw_init_variant_param_requirement(ctx, cell))
            .collect::<Result<Vec<_>, _>>()?,
        variant_conditions: entry
            .variant_conditions
            .iter()
            .map(|condition| reproject_raw_init_variant_condition(ctx, condition))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn raw_init_summary_fact_count(summary: &RawCellInitializationFunctionSummary) -> usize {
    summary.return_cells.len()
        + summary.return_byte_ranges.len()
        + summary.param_cells.len()
        + summary.param_byte_ranges.len()
        + summary.param_release_requirements.len()
        + summary.variant_param_cells.len()
        + summary.variant_param_byte_ranges.len()
        + summary.variant_required_param_cells.len()
        + summary.variant_conditions.len()
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

fn stable_i32_scalar_return_alias(
    types: &TypeCtx,
    function: &ResourceFunction,
    fact: &I32ScalarReturnAlias,
) -> Result<
    ResourceSummaryStableI32ScalarReturnAlias,
    ResourceSummaryStableI32ScalarReturnFactsEntryReject,
> {
    Ok(ResourceSummaryStableI32ScalarReturnAlias {
        return_projection: stable_i32_scalar_return_projection(
            types,
            function,
            &fact.return_projection,
        )?,
        parameter_index: fact.parameter_index,
        parameter_projection: stable_i32_scalar_parameter_projection(
            types,
            function,
            fact.parameter_index,
            &fact.parameter_projection,
        )?,
        scalar_ty: stable_i32_scalar_type(types, fact.scalar_ty)?,
    })
}

fn stable_i32_scalar_return_offset(
    types: &TypeCtx,
    function: &ResourceFunction,
    fact: &I32ScalarReturnOffset,
) -> Result<
    ResourceSummaryStableI32ScalarReturnOffset,
    ResourceSummaryStableI32ScalarReturnFactsEntryReject,
> {
    Ok(ResourceSummaryStableI32ScalarReturnOffset {
        return_projection: stable_i32_scalar_return_projection(
            types,
            function,
            &fact.return_projection,
        )?,
        parameter_index: fact.parameter_index,
        parameter_projection: stable_i32_scalar_parameter_projection(
            types,
            function,
            fact.parameter_index,
            &fact.parameter_projection,
        )?,
        scalar_ty: stable_i32_scalar_type(types, fact.scalar_ty)?,
        offset: fact.offset,
    })
}

fn stable_i32_scalar_return_relation(
    types: &TypeCtx,
    function: &ResourceFunction,
    fact: &I32ScalarReturnRelation,
) -> Result<
    ResourceSummaryStableI32ScalarReturnRelation,
    ResourceSummaryStableI32ScalarReturnFactsEntryReject,
> {
    Ok(ResourceSummaryStableI32ScalarReturnRelation {
        left_return_projection: stable_i32_scalar_return_projection(
            types,
            function,
            &fact.left_return_projection,
        )?,
        op: fact.op,
        right_return_projection: stable_i32_scalar_return_projection(
            types,
            function,
            &fact.right_return_projection,
        )?,
        scalar_ty: stable_i32_scalar_type(types, fact.scalar_ty)?,
    })
}

fn stable_i32_scalar_return_constant(
    types: &TypeCtx,
    function: &ResourceFunction,
    fact: &I32ScalarReturnConstant,
) -> Result<
    ResourceSummaryStableI32ScalarReturnConstant,
    ResourceSummaryStableI32ScalarReturnFactsEntryReject,
> {
    Ok(ResourceSummaryStableI32ScalarReturnConstant {
        return_projection: stable_i32_scalar_return_projection(
            types,
            function,
            &fact.return_projection,
        )?,
        scalar_ty: stable_i32_scalar_type(types, fact.scalar_ty)?,
        value: fact.value,
    })
}

fn stable_i32_scalar_return_condition(
    types: &TypeCtx,
    function: &ResourceFunction,
    fact: &I32ScalarReturnCondition,
) -> Result<
    ResourceSummaryStableI32ScalarReturnCondition,
    ResourceSummaryStableI32ScalarReturnFactsEntryReject,
> {
    Ok(ResourceSummaryStableI32ScalarReturnCondition {
        return_projection: stable_i32_scalar_return_projection(
            types,
            function,
            &fact.return_projection,
        )?,
        scalar_ty: stable_i32_scalar_type(types, fact.scalar_ty)?,
        condition: fact.condition,
    })
}

fn stable_i32_scalar_parameter_condition(
    types: &TypeCtx,
    function: &ResourceFunction,
    fact: &I32ScalarParameterCondition,
) -> Result<
    ResourceSummaryStableI32ScalarParameterCondition,
    ResourceSummaryStableI32ScalarReturnFactsEntryReject,
> {
    Ok(ResourceSummaryStableI32ScalarParameterCondition {
        parameter_index: fact.parameter_index,
        parameter_projection: stable_i32_scalar_parameter_projection(
            types,
            function,
            fact.parameter_index,
            &fact.parameter_projection,
        )?,
        scalar_ty: stable_i32_scalar_type(types, fact.scalar_ty)?,
        condition: fact.condition,
    })
}

fn stable_i32_scalar_return_projection(
    types: &TypeCtx,
    function: &ResourceFunction,
    projection: &[PlaceProjection],
) -> Result<
    Vec<ResourceSummaryStablePlaceProjection>,
    ResourceSummaryStableI32ScalarReturnFactsEntryReject,
> {
    stable_i32_scalar_projection(types, function, projection)
        .map_err(|_| ResourceSummaryStableI32ScalarReturnFactsEntryReject::ReturnProjection)
}

fn stable_i32_scalar_parameter_projection(
    types: &TypeCtx,
    function: &ResourceFunction,
    parameter_index: usize,
    projection: &[PlaceProjection],
) -> Result<
    Vec<ResourceSummaryStablePlaceProjection>,
    ResourceSummaryStableI32ScalarReturnFactsEntryReject,
> {
    if function.params.get(parameter_index).is_none() {
        return Err(ResourceSummaryStableI32ScalarReturnFactsEntryReject::ParameterProjection);
    }
    stable_i32_scalar_projection(types, function, projection)
        .map_err(|_| ResourceSummaryStableI32ScalarReturnFactsEntryReject::ParameterProjection)
}

fn stable_i32_scalar_projection(
    types: &TypeCtx,
    function: &ResourceFunction,
    projection: &[PlaceProjection],
) -> Result<
    Vec<ResourceSummaryStablePlaceProjection>,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    projection
        .iter()
        .map(|projection| stable_place_projection(types, &function.params, projection))
        .collect()
}

fn stable_i32_scalar_type(
    types: &TypeCtx,
    ty: TypeId,
) -> Result<ResourceSummaryStableTypeKey, ResourceSummaryStableI32ScalarReturnFactsEntryReject> {
    ResourceSummaryStableTypeKey::from_type(types, ty)
        .ok_or(ResourceSummaryStableI32ScalarReturnFactsEntryReject::ScalarType)
}

fn stable_raw_init_param_cell(
    types: &TypeCtx,
    cell: &RawCellInitializationParamCell,
) -> Result<
    ResourceSummaryStableRawInitParamCell,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    let suffix = cell
        .suffix
        .iter()
        .map(|projection| {
            stable_summary_projection(types, projection)
                .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellProjection)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let ty = ResourceSummaryStableTypeKey::from_type(types, cell.ty)
        .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
    Ok(ResourceSummaryStableRawInitParamCell {
        param_index: cell.param_index,
        suffix,
        ty,
        holds_raw_address: cell.holds_raw_address,
    })
}

fn stable_raw_init_return_cell(
    types: &TypeCtx,
    params: &[ResourceLocal],
    cell: &RawCellInitializationReturnCell,
) -> Result<
    ResourceSummaryStableRawInitReturnCell,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    let suffix = cell
        .suffix
        .iter()
        .map(|projection| stable_place_projection(types, params, projection))
        .collect::<Result<Vec<_>, _>>()?;
    let ty = ResourceSummaryStableTypeKey::from_type(types, cell.ty)
        .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
    Ok(ResourceSummaryStableRawInitReturnCell {
        suffix,
        ty,
        holds_raw_address: cell.holds_raw_address,
    })
}

fn stable_raw_init_return_byte_range(
    types: &TypeCtx,
    params: &[ResourceLocal],
    range: &RawCellInitializationReturnByteRange,
) -> Result<
    ResourceSummaryStableRawInitReturnByteRange,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    let address_suffix = range
        .address_suffix
        .iter()
        .map(|projection| stable_place_projection(types, params, projection))
        .collect::<Result<Vec<_>, _>>()?;
    let address_ty = ResourceSummaryStableTypeKey::from_type(types, range.address_ty)
        .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
    let count = stable_raw_init_return_count(types, params, &range.count)?;
    let ty = ResourceSummaryStableTypeKey::from_type(types, range.ty)
        .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
    Ok(ResourceSummaryStableRawInitReturnByteRange {
        address_suffix,
        address_ty,
        count,
        unit: range.unit,
        ty,
    })
}

fn stable_raw_init_return_count(
    types: &TypeCtx,
    params: &[ResourceLocal],
    count: &RawCellInitializationReturnCount,
) -> Result<
    ResourceSummaryStableRawInitReturnCount,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    Ok(match count {
        RawCellInitializationReturnCount::ReturnValueProjection { suffix, ty } => {
            let suffix = suffix
                .iter()
                .map(|projection| stable_place_projection(types, params, projection))
                .collect::<Result<Vec<_>, _>>()?;
            let ty = ResourceSummaryStableTypeKey::from_type(types, *ty)
                .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
            ResourceSummaryStableRawInitReturnCount::ReturnValueProjection { suffix, ty }
        }
        RawCellInitializationReturnCount::KnownI32 { value, ty } => {
            let ty = ResourceSummaryStableTypeKey::from_type(types, *ty)
                .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
            ResourceSummaryStableRawInitReturnCount::KnownI32 { value: *value, ty }
        }
    })
}

fn stable_raw_init_param_byte_range(
    types: &TypeCtx,
    range: &RawCellInitializationParamByteRange,
) -> Result<
    ResourceSummaryStableRawInitParamByteRange,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    let address_suffix = range
        .address_suffix
        .iter()
        .map(|projection| {
            stable_summary_projection(types, projection)
                .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellProjection)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let address_ty = ResourceSummaryStableTypeKey::from_type(types, range.address_ty)
        .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
    let count = stable_raw_init_param_count(types, &range.count)?;
    let ty = ResourceSummaryStableTypeKey::from_type(types, range.ty)
        .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
    Ok(ResourceSummaryStableRawInitParamByteRange {
        address_param_index: range.address_param_index,
        address_suffix,
        address_ty,
        count,
        unit: range.unit,
        ty,
    })
}

fn stable_raw_init_param_count(
    types: &TypeCtx,
    count: &RawCellInitializationParamCount,
) -> Result<
    ResourceSummaryStableRawInitParamCount,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    Ok(match count {
        RawCellInitializationParamCount::ParamProjection {
            param_index,
            suffix,
            ty,
        } => {
            let suffix = suffix
                .iter()
                .map(|projection| {
                    stable_summary_projection(types, projection).ok_or(
                        ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellProjection,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let ty = ResourceSummaryStableTypeKey::from_type(types, *ty)
                .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
            ResourceSummaryStableRawInitParamCount::ParamProjection {
                param_index: *param_index,
                suffix,
                ty,
            }
        }
        RawCellInitializationParamCount::KnownI32 { value, ty } => {
            let ty = ResourceSummaryStableTypeKey::from_type(types, *ty)
                .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
            ResourceSummaryStableRawInitParamCount::KnownI32 { value: *value, ty }
        }
    })
}

fn stable_raw_init_variant_param_cell(
    types: &TypeCtx,
    cell: &RawCellInitializationVariantParamCell,
) -> Result<
    ResourceSummaryStableRawInitVariantParamCell,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    let base = stable_raw_init_param_cell(
        types,
        &RawCellInitializationParamCell {
            param_index: cell.param_index,
            suffix: cell.suffix.clone(),
            ty: cell.ty,
            holds_raw_address: cell.holds_raw_address,
        },
    )?;
    Ok(ResourceSummaryStableRawInitVariantParamCell {
        variant: cell.variant.clone(),
        param_index: base.param_index,
        suffix: base.suffix,
        ty: base.ty,
        holds_raw_address: base.holds_raw_address,
    })
}

fn stable_raw_init_variant_param_byte_range(
    types: &TypeCtx,
    range: &RawCellInitializationVariantParamByteRange,
) -> Result<
    ResourceSummaryStableRawInitVariantParamByteRange,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    let base = stable_raw_init_param_byte_range(
        types,
        &RawCellInitializationParamByteRange {
            address_param_index: range.address_param_index,
            address_suffix: range.address_suffix.clone(),
            address_ty: range.address_ty,
            count: range.count.clone(),
            unit: range.unit,
            ty: range.ty,
        },
    )?;
    Ok(ResourceSummaryStableRawInitVariantParamByteRange {
        variant: range.variant.clone(),
        address_param_index: base.address_param_index,
        address_suffix: base.address_suffix,
        address_ty: base.address_ty,
        count: base.count,
        unit: base.unit,
        ty: base.ty,
    })
}

fn stable_raw_init_variant_param_requirement(
    types: &TypeCtx,
    cell: &RawCellInitializationVariantParamRequirement,
) -> Result<
    ResourceSummaryStableRawInitVariantParamRequirement,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    let suffix = cell
        .suffix
        .iter()
        .map(|projection| {
            stable_summary_projection(types, projection)
                .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellProjection)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let ty = ResourceSummaryStableTypeKey::from_type(types, cell.ty)
        .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
    Ok(ResourceSummaryStableRawInitVariantParamRequirement {
        variant: cell.variant.clone(),
        param_index: cell.param_index,
        suffix,
        ty,
    })
}

fn stable_raw_init_variant_condition(
    types: &TypeCtx,
    condition: &RawCellInitializationVariantCondition,
) -> Result<
    ResourceSummaryStableRawInitVariantCondition,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    let suffix = condition
        .suffix
        .iter()
        .map(|projection| {
            stable_summary_projection(types, projection)
                .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellProjection)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let ty = ResourceSummaryStableTypeKey::from_type(types, condition.ty)
        .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType)?;
    Ok(ResourceSummaryStableRawInitVariantCondition {
        variant: condition.variant.clone(),
        param_index: condition.param_index,
        suffix,
        ty,
        condition: condition.condition,
    })
}

fn stable_raw_cell_release_param_requirement(
    types: &TypeCtx,
    params: &[ResourceLocal],
    requirement: &RawCellReleaseParamRequirement,
) -> Result<
    ResourceSummaryStableRawCellReleaseParamRequirement,
    ResourceSummaryStableRawInitCompleteLeafEntryReject,
> {
    let suffix = requirement
        .suffix
        .iter()
        .map(|projection| stable_place_projection(types, params, projection))
        .collect::<Result<Vec<_>, _>>()?;
    let ty = ResourceSummaryStableTypeKey::from_type(types, requirement.ty)
        .ok_or(ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamReleaseRequirementType)?;
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
) -> Result<ResourceSummaryStablePlaceProjection, ResourceSummaryStableRawInitCompleteLeafEntryReject>
{
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

fn reproject_i32_scalar_return_alias(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    fact: &ResourceSummaryStableI32ScalarReturnAlias,
) -> Result<I32ScalarReturnAlias, ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject> {
    Ok(I32ScalarReturnAlias {
        return_projection: reproject_i32_scalar_return_projection(
            ctx,
            &fact.return_projection,
            &fact.scalar_ty,
        )?,
        parameter_index: fact.parameter_index,
        parameter_projection: reproject_i32_scalar_parameter_projection(
            ctx,
            fact.parameter_index,
            &fact.parameter_projection,
            &fact.scalar_ty,
        )?,
        scalar_ty: reproject_i32_scalar_type(ctx, &fact.scalar_ty)?,
    })
}

fn reproject_i32_scalar_return_offset(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    fact: &ResourceSummaryStableI32ScalarReturnOffset,
) -> Result<I32ScalarReturnOffset, ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject> {
    Ok(I32ScalarReturnOffset {
        return_projection: reproject_i32_scalar_return_projection(
            ctx,
            &fact.return_projection,
            &fact.scalar_ty,
        )?,
        parameter_index: fact.parameter_index,
        parameter_projection: reproject_i32_scalar_parameter_projection(
            ctx,
            fact.parameter_index,
            &fact.parameter_projection,
            &fact.scalar_ty,
        )?,
        scalar_ty: reproject_i32_scalar_type(ctx, &fact.scalar_ty)?,
        offset: fact.offset,
    })
}

fn reproject_i32_scalar_return_relation(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    fact: &ResourceSummaryStableI32ScalarReturnRelation,
) -> Result<I32ScalarReturnRelation, ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject> {
    Ok(I32ScalarReturnRelation {
        left_return_projection: reproject_i32_scalar_return_projection(
            ctx,
            &fact.left_return_projection,
            &fact.scalar_ty,
        )?,
        op: fact.op,
        right_return_projection: reproject_i32_scalar_return_projection(
            ctx,
            &fact.right_return_projection,
            &fact.scalar_ty,
        )?,
        scalar_ty: reproject_i32_scalar_type(ctx, &fact.scalar_ty)?,
    })
}

fn reproject_i32_scalar_return_constant(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    fact: &ResourceSummaryStableI32ScalarReturnConstant,
) -> Result<I32ScalarReturnConstant, ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject> {
    Ok(I32ScalarReturnConstant {
        return_projection: reproject_i32_scalar_return_projection(
            ctx,
            &fact.return_projection,
            &fact.scalar_ty,
        )?,
        scalar_ty: reproject_i32_scalar_type(ctx, &fact.scalar_ty)?,
        value: fact.value,
    })
}

fn reproject_i32_scalar_return_condition(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    fact: &ResourceSummaryStableI32ScalarReturnCondition,
) -> Result<I32ScalarReturnCondition, ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject> {
    Ok(I32ScalarReturnCondition {
        return_projection: reproject_i32_scalar_return_projection(
            ctx,
            &fact.return_projection,
            &fact.scalar_ty,
        )?,
        scalar_ty: reproject_i32_scalar_type(ctx, &fact.scalar_ty)?,
        condition: fact.condition,
    })
}

fn reproject_i32_scalar_parameter_condition(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    fact: &ResourceSummaryStableI32ScalarParameterCondition,
) -> Result<I32ScalarParameterCondition, ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject>
{
    Ok(I32ScalarParameterCondition {
        parameter_index: fact.parameter_index,
        parameter_projection: reproject_i32_scalar_parameter_projection(
            ctx,
            fact.parameter_index,
            &fact.parameter_projection,
            &fact.scalar_ty,
        )?,
        scalar_ty: reproject_i32_scalar_type(ctx, &fact.scalar_ty)?,
        condition: fact.condition,
    })
}

fn reproject_i32_scalar_return_projection(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    suffix: &[ResourceSummaryStablePlaceProjection],
    scalar_ty: &ResourceSummaryStableTypeKey,
) -> Result<Vec<PlaceProjection>, ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject> {
    let (suffix, ty) = reproject_place_projection_suffix(ctx, ctx.function.result, suffix)
        .ok_or(ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ReturnProjection)?;
    if !scalar_ty.matches_type(ctx.types, ty) {
        return Err(ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ScalarType);
    }
    Ok(suffix)
}

fn reproject_i32_scalar_parameter_projection(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    parameter_index: usize,
    suffix: &[ResourceSummaryStablePlaceProjection],
    scalar_ty: &ResourceSummaryStableTypeKey,
) -> Result<Vec<PlaceProjection>, ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject> {
    let base_ty = ctx
        .function
        .params
        .get(parameter_index)
        .ok_or(ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ParameterProjection)?
        .ty;
    let (suffix, ty) = reproject_place_projection_suffix(ctx, base_ty, suffix)
        .ok_or(ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ParameterProjection)?;
    if !scalar_ty.matches_type(ctx.types, ty) {
        return Err(ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ScalarType);
    }
    Ok(suffix)
}

fn reproject_i32_scalar_type(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    ty: &ResourceSummaryStableTypeKey,
) -> Result<TypeId, ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject> {
    ctx.reproject_type(ty)
        .ok_or(ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ScalarType)
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
        ResourceSummaryStableOffset::ResourceSymbolic { .. } => return None,
        ResourceSummaryStableOffset::ScaledSymbolic { place, scale } => {
            SummaryOffset::ScaledSymbolic {
                place: Box::new(reproject_summary_place(ctx, place)?),
                scale: *scale,
            }
        }
        ResourceSummaryStableOffset::ResourceScaledSymbolic { .. } => return None,
        ResourceSummaryStableOffset::Offset { place, offset } => SummaryOffset::Offset {
            place: Box::new(reproject_summary_place(ctx, place)?),
            offset: *offset,
        },
        ResourceSummaryStableOffset::ResourceOffset { .. } => return None,
        ResourceSummaryStableOffset::ScaledOffset {
            place,
            offset,
            scale,
        } => SummaryOffset::ScaledOffset {
            place: Box::new(reproject_summary_place(ctx, place)?),
            offset: *offset,
            scale: *scale,
        },
        ResourceSummaryStableOffset::ResourceScaledOffset { .. } => return None,
        ResourceSummaryStableOffset::Unknown => SummaryOffset::Unknown,
    })
}

fn reproject_raw_init_param_cell(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    cell: &ResourceSummaryStableRawInitParamCell,
) -> Result<RawCellInitializationParamCell, ResourceSummaryRawInitCompleteLeafEntryReprojectionReject>
{
    let base = ctx
        .function
        .params
        .get(cell.param_index)
        .ok_or(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection)?
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
    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
> {
    let stable_key = stable_ty;
    let mut out = Vec::new();
    let mut current_ty = base_ty;
    let mut used_stored_cell_ty = false;
    for stable_projection in suffix {
        let projection = reproject_summary_projection(ctx, stable_projection).ok_or(
            ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection,
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
        return Err(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellResultType);
    }
    Ok((out, current_ty))
}

fn reproject_raw_init_param_cell_projection_result_type(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    current_ty: TypeId,
    stable_cell_ty: &ResourceSummaryStableTypeKey,
    projection: &SummaryProjection,
    used_stored_cell_ty: &mut bool,
) -> Result<TypeId, ResourceSummaryRawInitCompleteLeafEntryReprojectionReject> {
    if matches!(projection, SummaryProjection::Deref) {
        if let Some(ty) = summary_projection_result_type(ctx.types, current_ty, projection) {
            return Ok(ty);
        }
        // raw-init の param cell は raw address から見た cell view を表せる。
        // その `Deref` は通常の参照型 dereference ではないため、ここでは保存済み
        // cell 型だけを復元先として採用し、field/tuple など通常projectionの検証は
        // 引き続き `summary_projection_result_type` に任せる。
        let stable_cell_ty = ctx.reproject_type(stable_cell_ty).ok_or(
            ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellStableType,
        )?;
        *used_stored_cell_ty = true;
        return Ok(stable_cell_ty);
    }
    validate_projection_layout(ctx.types, current_ty, projection)
        .ok_or(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection)?;
    if let Some(ty) = summary_projection_result_type(ctx.types, current_ty, projection) {
        return Ok(ty);
    }
    Err(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection)
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

fn reproject_raw_init_return_cell(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    cell: &ResourceSummaryStableRawInitReturnCell,
) -> Result<
    RawCellInitializationReturnCell,
    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
> {
    let (suffix, ty) = reproject_stable_place_projection_suffix_with_expected_type(
        ctx,
        ctx.function.result,
        &cell.ty,
        &cell.suffix,
    )?;
    Ok(RawCellInitializationReturnCell {
        suffix,
        ty,
        holds_raw_address: cell.holds_raw_address,
    })
}

fn reproject_raw_init_return_byte_range(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    range: &ResourceSummaryStableRawInitReturnByteRange,
) -> Result<
    RawCellInitializationReturnByteRange,
    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
> {
    let (address_suffix, address_ty) = reproject_stable_place_projection_suffix_with_expected_type(
        ctx,
        ctx.function.result,
        &range.address_ty,
        &range.address_suffix,
    )?;
    let count = reproject_raw_init_return_count(ctx, &range.count)?;
    let ty = reproject_raw_init_stable_type(ctx, &range.ty)?;
    Ok(RawCellInitializationReturnByteRange {
        address_suffix,
        address_ty,
        count,
        unit: range.unit,
        ty,
    })
}

fn reproject_raw_init_return_count(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    count: &ResourceSummaryStableRawInitReturnCount,
) -> Result<
    RawCellInitializationReturnCount,
    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
> {
    Ok(match count {
        ResourceSummaryStableRawInitReturnCount::ReturnValueProjection { suffix, ty } => {
            let (suffix, ty) = reproject_stable_place_projection_suffix_with_expected_type(
                ctx,
                ctx.function.result,
                ty,
                suffix,
            )?;
            RawCellInitializationReturnCount::ReturnValueProjection { suffix, ty }
        }
        ResourceSummaryStableRawInitReturnCount::KnownI32 { value, ty } => {
            RawCellInitializationReturnCount::KnownI32 {
                value: *value,
                ty: reproject_raw_init_stable_type(ctx, ty)?,
            }
        }
    })
}

fn reproject_raw_init_param_byte_range(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    range: &ResourceSummaryStableRawInitParamByteRange,
) -> Result<
    RawCellInitializationParamByteRange,
    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
> {
    let base = ctx
        .function
        .params
        .get(range.address_param_index)
        .ok_or(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection)?
        .ty;
    let (address_suffix, address_ty) = reproject_raw_init_param_cell_summary_suffix(
        ctx,
        base,
        &range.address_ty,
        &range.address_suffix,
    )?;
    let count = reproject_raw_init_param_count(ctx, &range.count)?;
    let ty = reproject_raw_init_stable_type(ctx, &range.ty)?;
    Ok(RawCellInitializationParamByteRange {
        address_param_index: range.address_param_index,
        address_suffix,
        address_ty,
        count,
        unit: range.unit,
        ty,
    })
}

fn reproject_raw_init_param_count(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    count: &ResourceSummaryStableRawInitParamCount,
) -> Result<
    RawCellInitializationParamCount,
    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
> {
    Ok(match count {
        ResourceSummaryStableRawInitParamCount::ParamProjection {
            param_index,
            suffix,
            ty,
        } => {
            let base = ctx
                .function
                .params
                .get(*param_index)
                .ok_or(
                    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection,
                )?
                .ty;
            let (suffix, ty) = reproject_raw_init_param_cell_summary_suffix(ctx, base, ty, suffix)?;
            RawCellInitializationParamCount::ParamProjection {
                param_index: *param_index,
                suffix,
                ty,
            }
        }
        ResourceSummaryStableRawInitParamCount::KnownI32 { value, ty } => {
            RawCellInitializationParamCount::KnownI32 {
                value: *value,
                ty: reproject_raw_init_stable_type(ctx, ty)?,
            }
        }
    })
}

fn reproject_raw_init_variant_param_cell(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    cell: &ResourceSummaryStableRawInitVariantParamCell,
) -> Result<
    RawCellInitializationVariantParamCell,
    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
> {
    let base = reproject_raw_init_param_cell(
        ctx,
        &ResourceSummaryStableRawInitParamCell {
            param_index: cell.param_index,
            suffix: cell.suffix.clone(),
            ty: cell.ty.clone(),
            holds_raw_address: cell.holds_raw_address,
        },
    )?;
    Ok(RawCellInitializationVariantParamCell {
        variant: cell.variant.clone(),
        param_index: base.param_index,
        suffix: base.suffix,
        ty: base.ty,
        holds_raw_address: base.holds_raw_address,
    })
}

fn reproject_raw_init_variant_param_byte_range(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    range: &ResourceSummaryStableRawInitVariantParamByteRange,
) -> Result<
    RawCellInitializationVariantParamByteRange,
    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
> {
    let base = reproject_raw_init_param_byte_range(
        ctx,
        &ResourceSummaryStableRawInitParamByteRange {
            address_param_index: range.address_param_index,
            address_suffix: range.address_suffix.clone(),
            address_ty: range.address_ty.clone(),
            count: range.count.clone(),
            unit: range.unit,
            ty: range.ty.clone(),
        },
    )?;
    Ok(RawCellInitializationVariantParamByteRange {
        variant: range.variant.clone(),
        address_param_index: base.address_param_index,
        address_suffix: base.address_suffix,
        address_ty: base.address_ty,
        count: base.count,
        unit: base.unit,
        ty: base.ty,
    })
}

fn reproject_raw_init_variant_param_requirement(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    cell: &ResourceSummaryStableRawInitVariantParamRequirement,
) -> Result<
    RawCellInitializationVariantParamRequirement,
    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
> {
    let base = ctx
        .function
        .params
        .get(cell.param_index)
        .ok_or(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection)?
        .ty;
    let (suffix, ty) =
        reproject_raw_init_param_cell_summary_suffix(ctx, base, &cell.ty, &cell.suffix)?;
    Ok(RawCellInitializationVariantParamRequirement {
        variant: cell.variant.clone(),
        param_index: cell.param_index,
        suffix,
        ty,
    })
}

fn reproject_raw_init_variant_condition(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    condition: &ResourceSummaryStableRawInitVariantCondition,
) -> Result<
    RawCellInitializationVariantCondition,
    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
> {
    let base = ctx
        .function
        .params
        .get(condition.param_index)
        .ok_or(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection)?
        .ty;
    let (suffix, ty) =
        reproject_raw_init_param_cell_summary_suffix(ctx, base, &condition.ty, &condition.suffix)?;
    Ok(RawCellInitializationVariantCondition {
        variant: condition.variant.clone(),
        param_index: condition.param_index,
        suffix,
        ty,
        condition: condition.condition,
    })
}

fn reproject_stable_place_projection_suffix_with_expected_type(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    base_ty: TypeId,
    expected_ty: &ResourceSummaryStableTypeKey,
    suffix: &[ResourceSummaryStablePlaceProjection],
) -> Result<(Vec<PlaceProjection>, TypeId), ResourceSummaryRawInitCompleteLeafEntryReprojectionReject>
{
    let mut out = Vec::new();
    let mut current_ty = base_ty;
    let mut used_stored_cell_ty = false;
    for (index, stable_projection) in suffix.iter().enumerate() {
        let projection = reproject_place_projection(ctx, stable_projection).ok_or(
            ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection,
        )?;
        let result_ty = match projection_result_type(ctx.types, current_ty, &projection) {
            Some(ty)
                if validate_place_projection_layout(ctx.types, current_ty, &projection)
                    .is_some() =>
            {
                ty
            }
            _ if matches!(projection, PlaceProjection::Deref) && index + 1 == suffix.len() => {
                // raw address 由来の `Deref` は通常の reference 型 dereference ではない。
                // suffix の最後でだけ保存済みの最終型を proof boundary として使い、
                // 途中 projection の型を推測して後続 field 検証を弱めない。
                used_stored_cell_ty = true;
                reproject_raw_init_stable_type(ctx, expected_ty)?
            }
            _ => {
                return Err(
                    ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection,
                );
            }
        };
        current_ty = result_ty;
        out.push(projection);
    }
    // return / byte-range 側の place projection も、通常の layout 規則で型が決まる
    // 場合は現在 compile の function signature と suffix が replay authority になる。
    // 保存済み型 key は raw address `Deref` のように typed projection だけでは値型を
    // 得られない場合にだけ proof boundary として照合する。
    if used_stored_cell_ty && !expected_ty.matches_type(ctx.types, current_ty) {
        return Err(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellResultType);
    }
    Ok((out, current_ty))
}

fn reproject_raw_init_stable_type(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    ty: &ResourceSummaryStableTypeKey,
) -> Result<TypeId, ResourceSummaryRawInitCompleteLeafEntryReprojectionReject> {
    ctx.reproject_type(ty)
        .ok_or(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellStableType)
}

fn reproject_raw_cell_release_param_requirement(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    requirement: &ResourceSummaryStableRawCellReleaseParamRequirement,
) -> Result<RawCellReleaseParamRequirement, ResourceSummaryRawInitCompleteLeafEntryReprojectionReject>
{
    let base = ctx
        .function
        .params
        .get(requirement.param_index)
        .ok_or(
            ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamReleaseRequirementProjection,
        )?
        .ty;
    let (suffix, ty) = reproject_place_projection_suffix(ctx, base, &requirement.suffix).ok_or(
        ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamReleaseRequirementProjection,
    )?;
    if !requirement.ty.matches_type(ctx.types, ty) {
        return Err(
            ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamReleaseRequirementType,
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
        ResourceSummaryStableOffset::ResourceSymbolic { place } => ResourceOffset::Symbolic {
            place: Box::new(reproject_stable_resource_place_to_place(ctx, place)?),
        },
        ResourceSummaryStableOffset::ScaledSymbolic { place, scale } => {
            ResourceOffset::ScaledSymbolic {
                place: Box::new(reproject_stable_place_to_place(ctx, place)?),
                scale: *scale,
            }
        }
        ResourceSummaryStableOffset::ResourceScaledSymbolic { place, scale } => {
            ResourceOffset::ScaledSymbolic {
                place: Box::new(reproject_stable_resource_place_to_place(ctx, place)?),
                scale: *scale,
            }
        }
        ResourceSummaryStableOffset::Offset { place, offset } => ResourceOffset::Offset {
            place: Box::new(reproject_stable_place_to_place(ctx, place)?),
            offset: *offset,
        },
        ResourceSummaryStableOffset::ResourceOffset { place, offset } => ResourceOffset::Offset {
            place: Box::new(reproject_stable_resource_place_to_place(ctx, place)?),
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
        ResourceSummaryStableOffset::ResourceScaledOffset {
            place,
            offset,
            scale,
        } => ResourceOffset::ScaledOffset {
            place: Box::new(reproject_stable_resource_place_to_place(ctx, place)?),
            offset: *offset,
            scale: *scale,
        },
        ResourceSummaryStableOffset::Unknown => ResourceOffset::Unknown,
    })
}

fn reproject_stable_resource_place_to_place(
    ctx: &ResourceSummaryTypeReprojection<'_>,
    place: &ResourceSummaryStableResourcePlace,
) -> Option<Place> {
    let place_ordinals = ResourceFunctionPlaceOrdinalMap::new(ctx.function);
    reproject_resource_place(ctx, &place_ordinals, place).ok()
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
        RawCellInitializationReturnCell,
    };
    use super::super::super::initialized_summary_byte_range_model::{
        RawCellInitializationReturnByteRange, RawCellInitializationReturnCount,
    };
    use super::super::super::initialized_summary_release_model::{
        RawCellReleaseParamRequirement, RawCellReleaseRequirementKind,
    };
    use super::super::super::model::{
        Place, PlaceProjection, PlaceRoot, ResourceBlock, ResourceBlockId, ResourceExprKind,
        ResourceFunction, ResourceId, ResourceLocal, ResourceOffset, ResourceOp,
        ResourceTerminator, StorageId, StorageOrigin,
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
    fn stable_initialized_check_reprojects_body_local_generic_state_type() {
        let mut types = TypeCtx::new();
        let value_ty = types.fresh_var(Some("T".to_string()));
        let output = Place::temporary(ResourceId(0), value_ty);
        let function = ResourceFunction {
            name: "generic_body_local".to_string(),
            origin_name: "generic_body_local".to_string(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: types.unit(),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::Literal,
                    output: output.clone(),
                    ty: value_ty,
                    span: Span::dummy(),
                }],
                terminator: ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let check = ResourceFunctionCheck {
            name: function.name.clone(),
            final_cells: vec![CellStateEntry {
                place: output.clone(),
                state: CellState::Initialized(value_ty),
            }],
            final_collection_slots: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
        };
        let entry = stable_initialized_function_check_entry(&types, &function, &check)
            .expect("body-local generic final state should be representable");
        let signature_ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("signature-only context itself should build");
        assert!(matches!(
            reproject_initialized_function_check_entry_result(
                &signature_ctx,
                &function.name,
                &entry
            ),
            Err(ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::PlaceType)
        ));
        let body_ctx = ResourceSummaryTypeReprojection::new_for_initialized_function_check(
            &types,
            &function,
            &[],
        )
        .expect("final check context should include function body types");

        let reprojected =
            reproject_initialized_function_check_entry(&body_ctx, &function.name, &entry)
                .expect("body-local generic final state should reproject");

        assert_eq!(reprojected.final_cells, check.final_cells);
    }

    #[test]
    fn stable_initialized_check_reprojects_temporary_storage_offset_place() {
        let types = TypeCtx::new();
        let index = Place::temporary(ResourceId(0), types.i32());
        let storage = Place {
            root: PlaceRoot::Storage(StorageId(0)),
            projections: Vec::new(),
            ty: types.i32(),
        };
        let slot = storage.clone().with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::ScaledOffset {
                place: Box::new(index.clone()),
                offset: 4,
                scale: 4,
            }),
            types.i32(),
        );
        let function = ResourceFunction {
            name: "temporary_storage_offset".to_string(),
            origin_name: "temporary_storage_offset".to_string(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: types.unit(),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::Expr {
                        kind: ResourceExprKind::LiteralI32(0),
                        output: index,
                        ty: types.i32(),
                        span: Span::dummy(),
                    },
                    ResourceOp::StorageOrigin {
                        target: storage,
                        origin: StorageOrigin::Internal,
                        span: Span::dummy(),
                    },
                ],
                terminator: ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let check = ResourceFunctionCheck {
            name: function.name.clone(),
            final_cells: Vec::new(),
            final_collection_slots: vec![CollectionSlotStateEntry {
                slot: slot.clone(),
                state: CollectionSlotState::Initialized(types.i32()),
            }],
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
        };

        let entry = stable_initialized_function_check_entry(&types, &function, &check)
            .expect("function-local offset place should be representable in final check entry");
        let ctx = ResourceSummaryTypeReprojection::new_for_initialized_function_check(
            &types,
            &function,
            &[],
        )
        .expect("primitive final check context should build");
        let reprojected = reproject_initialized_function_check_entry(&ctx, &function.name, &entry)
            .expect("function-local offset place should reproject");

        assert_eq!(
            reprojected.final_collection_slots,
            check.final_collection_slots
        );
    }

    #[test]
    fn stable_initialized_check_reprojects_layout_opaque_generic_projection() {
        let mut types = TypeCtx::new();
        let generic = types.fresh_var(Some("T".to_string()));
        let param = Place::local("value".to_string(), generic);
        let projected = param.clone().with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            generic,
        );
        let function = ResourceFunction {
            name: "layout_opaque_projection".to_string(),
            origin_name: "layout_opaque_projection".to_string(),
            type_params: vec![generic],
            params: vec![ResourceLocal {
                name: "value".to_string(),
                ty: generic,
                mutable: false,
                place: param,
            }],
            result: types.unit(),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: Vec::new(),
                terminator: ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let check = ResourceFunctionCheck {
            name: function.name.clone(),
            final_cells: vec![CellStateEntry {
                place: projected,
                state: CellState::Initialized(generic),
            }],
            final_collection_slots: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
        };
        let entry = stable_initialized_function_check_entry(&types, &function, &check)
            .expect("layout-opaque projection should still have a stable final check surface");
        let ctx = ResourceSummaryTypeReprojection::new_for_initialized_function_check(
            &types,
            &function,
            &[generic],
        )
        .expect("generic final check context should build");

        let reprojected = reproject_initialized_function_check_entry(&ctx, &function.name, &entry)
            .expect("layout-opaque stable projection should reproject from the body hash");

        assert_eq!(reprojected.final_cells, check.final_cells);
    }

    #[test]
    fn stable_initialized_check_prefers_stored_open_generic_projection_type() {
        let mut types = TypeCtx::new();
        let definition_generic = types.fresh_var(Some("DefinitionT".to_string()));
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
        let param = Place::local("value".to_string(), nominal);
        let projected = param.clone().with_projection(
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            function_generic,
        );
        let function = ResourceFunction {
            name: "stored_open_generic_projection_type".to_string(),
            origin_name: "stored_open_generic_projection_type".to_string(),
            type_params: vec![function_generic],
            params: vec![ResourceLocal {
                name: "value".to_string(),
                ty: nominal,
                mutable: false,
                place: param,
            }],
            result: types.unit(),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: Vec::new(),
                terminator: ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        };
        let check = ResourceFunctionCheck {
            name: function.name.clone(),
            final_cells: vec![CellStateEntry {
                place: projected,
                state: CellState::Initialized(function_generic),
            }],
            final_collection_slots: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
        };
        let entry = stable_initialized_function_check_entry(&types, &function, &check)
            .expect("open generic place type should have a stable final check surface");
        let ctx = ResourceSummaryTypeReprojection::new_for_initialized_function_check(
            &types,
            &function,
            &[function_generic],
        )
        .expect("function and definition generic boundary should build");

        let reprojected = reproject_initialized_function_check_entry(&ctx, &function.name, &entry)
            .expect("stored open generic place type should be replay authority");

        assert_eq!(reprojected.final_cells, check.final_cells);
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
    fn stable_raw_init_complete_leaf_reprojects_nominal_field_type_from_signature_tree() {
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("nominal field param facts should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("signature tree should register nominal field type");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
            .expect("nominal field fact should reproject");

        assert_eq!(reprojected, summary);
    }

    #[test]
    fn stable_raw_init_complete_leaf_reprojects_instantiated_generic_nominal_field_type() {
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("instantiated generic nominal field param facts should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[function_generic])
            .expect("definition generic should not shadow the instantiated boundary generic");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("duplicate nominal signature fact should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("duplicate nominal stable keys should be accepted as signature aliases");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
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
    fn stable_raw_init_complete_leaf_reprojects_duplicate_structural_signature_key() {
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("duplicate structural signature fact should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("duplicate structural stable keys should be accepted as signature aliases");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
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
        let mut entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("valid param cell should convert before corruption");
        entry.param_cells[0].suffix[0] = ResourceSummaryStableProjection::Field {
            index: 0,
            offset_bytes: 4,
        };
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("nominal function boundary should be reprojectable");

        let result = reproject_raw_init_complete_leaf_entry_result(&ctx, &function.name, &entry);

        assert!(matches!(
            result,
            Err(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection)
        ));
    }

    #[test]
    fn stable_raw_init_reprojection_reports_return_byte_range_projection_mismatch() {
        let mut types = TypeCtx::new();
        let field = types.i32();
        let nominal = types.register_named_with_stable_identity(
            "ReturnRecord".to_string(),
            TypeKind::Struct {
                name: "ReturnRecord".to_string(),
                type_params: Vec::new(),
                fields: vec![field],
                field_names: vec!["value".to_string()],
            },
            nominal_struct_identity("ReturnRecord"),
        );
        let function = function_with_params(Vec::new(), nominal);
        let mut summary = empty_raw_init_summary(&function);
        summary
            .return_byte_ranges
            .push(RawCellInitializationReturnByteRange {
                address_suffix: vec![PlaceProjection::Field {
                    index: 0,
                    offset_bytes: 0,
                }],
                address_ty: field,
                count: RawCellInitializationReturnCount::KnownI32 {
                    value: 4,
                    ty: types.i32(),
                },
                unit: super::super::super::cell_state_raw_range::InitializedRawRangeUnit::Bytes,
                ty: types.u8(),
            });
        let mut entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("valid return byte-range fact should convert before corruption");
        entry.return_byte_ranges[0].address_suffix[0] =
            ResourceSummaryStablePlaceProjection::Field {
                index: 0,
                offset_bytes: 4,
            };
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("return type boundary should be reprojectable");

        let result = reproject_raw_init_complete_leaf_entry_result(&ctx, &function.name, &entry);

        assert!(matches!(
            result,
            Err(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection)
        ));
    }

    #[test]
    fn stable_raw_init_return_cell_rejects_non_final_raw_deref_fallback() {
        let types = TypeCtx::new();
        let function = function_with_params(Vec::new(), types.i32());
        let mut summary = empty_raw_init_summary(&function);
        summary.return_cells.push(RawCellInitializationReturnCell {
            suffix: vec![
                PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
                PlaceProjection::Deref,
                PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
            ],
            ty: types.u8(),
            holds_raw_address: false,
        });
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("raw return-cell surface should convert before reprojection validation");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive return boundary should be reprojectable");

        let result = reproject_raw_init_complete_leaf_entry_result(&ctx, &function.name, &entry);

        assert!(matches!(
            result,
            Err(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection)
        ));
    }

    #[test]
    fn stable_raw_init_return_cell_reprojects_final_raw_deref_value_type() {
        let types = TypeCtx::new();
        let function = function_with_params(Vec::new(), types.i32());
        let mut summary = empty_raw_init_summary(&function);
        summary.return_cells.push(RawCellInitializationReturnCell {
            suffix: vec![
                PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
                PlaceProjection::Deref,
            ],
            ty: types.u8(),
            holds_raw_address: false,
        });
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("final raw-deref return cell should convert with its explicit value type");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive return boundary should be reprojectable");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
            .expect("final raw-deref return cell should use the stable value type");

        assert_eq!(reprojected, summary);
    }

    #[test]
    fn stable_raw_init_return_cell_rejects_non_boundary_open_generic_raw_deref_type() {
        let mut types = TypeCtx::new();
        let value_ty = types.fresh_var(Some("T".to_string()));
        let function = function_with_params(Vec::new(), types.i32());
        let mut summary = empty_raw_init_summary(&function);
        summary.return_cells.push(RawCellInitializationReturnCell {
            suffix: vec![
                PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
                PlaceProjection::Deref,
            ],
            ty: value_ty,
            holds_raw_address: false,
        });
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("labelled generic raw return-cell type can be represented as a stable key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive return boundary should be reprojectable");

        let result = reproject_raw_init_complete_leaf_entry_result(&ctx, &function.name, &entry);

        assert!(matches!(
            result,
            Err(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellStableType)
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
        let mut entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("valid param cell should convert before corruption");
        entry.param_cells[0].ty = ResourceSummaryStableTypeKey::from_type(&types, types.bool())
            .expect("bool has a stable type key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
            .expect("projection-derived param cell type should come from the current signature");

        assert_eq!(reprojected.param_cells[0].ty, types.i32());
    }

    #[test]
    fn stable_raw_init_return_cell_uses_signature_type_for_projection_derived_cell() {
        let mut types = TypeCtx::new();
        let field = types.i32();
        let nominal = types.register_named_with_stable_identity(
            "ReturnRecord".to_string(),
            TypeKind::Struct {
                name: "ReturnRecord".to_string(),
                type_params: Vec::new(),
                fields: vec![field],
                field_names: vec!["value".to_string()],
            },
            nominal_struct_identity("ReturnRecord"),
        );
        let function = function_with_params(Vec::new(), nominal);
        let mut summary = empty_raw_init_summary(&function);
        summary.return_cells.push(RawCellInitializationReturnCell {
            suffix: vec![PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            }],
            ty: field,
            holds_raw_address: false,
        });
        let mut entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("projection-derived return cell should convert before corruption");
        entry.return_cells[0].ty = ResourceSummaryStableTypeKey::from_type(&types, types.bool())
            .expect("bool has a stable type key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("return signature boundary should be reprojectable");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
            .expect("projection-derived return cell type should come from the current signature");

        assert_eq!(reprojected.return_cells[0].ty, field);
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("raw-deref param cell should convert with its explicit value type");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("non-signature nominal raw cell type should convert to a stable key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("labelled generic raw cell type can be represented as a stable key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let result = reproject_raw_init_complete_leaf_entry_result(&ctx, &function.name, &entry);

        assert!(matches!(
            result,
            Err(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellStableType)
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("labelled generic raw cell type should convert to a stable key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[value_ty])
            .expect("owner summary boundary should make the generic reprojectable");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("projection-derived open generic value type should remain representable");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("applied signature should be reprojectable");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
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
        let mut entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("valid release requirement should convert before corruption");
        entry.param_release_requirements[0].suffix[0] =
            ResourceSummaryStablePlaceProjection::Field {
                index: 0,
                offset_bytes: 4,
            };
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("nominal function boundary should be reprojectable");

        let result = reproject_raw_init_complete_leaf_entry_result(&ctx, &function.name, &entry);

        assert!(matches!(
            result,
            Err(
                ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamReleaseRequirementProjection
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
        let mut entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("valid release requirement should convert before corruption");
        entry.param_release_requirements[0].ty =
            ResourceSummaryStableTypeKey::from_type(&types, types.bool())
                .expect("bool has a stable type key");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let result = reproject_raw_init_complete_leaf_entry_result(&ctx, &function.name, &entry);

        assert!(matches!(
            result,
            Err(
                ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamReleaseRequirementType
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("parameter-relative storage offset should convert");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("unknown storage offset should remain a conservative stable fact");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
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
        let entry = stable_raw_init_complete_leaf_entry(&types, &function, &summary)
            .expect("local offset should degrade instead of blocking the stable entry");
        let ctx = ResourceSummaryTypeReprojection::new(&types, &function, &[])
            .expect("primitive function boundary should be reprojectable");

        let reprojected = reproject_raw_init_complete_leaf_entry(&ctx, &function.name, &entry)
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
