#![allow(dead_code)]

extern crate alloc;

use alloc::collections::BTreeMap;

use crate::ast::Effect;
use crate::effects::PrivateEffectRegion;
use crate::function_identity::FunctionValueIdentity;
use crate::types::{TypeCtx, TypeId};

use super::super::collection_slot_lifecycle_model::{
    CollectionSlotLifecycleEvent, CollectionSlotReplacement,
};
use super::super::model::{
    AggregateKind, BorrowKind, EffectOp, I32ValueCondition, Place, PlaceProjection, PlaceRoot,
    RawAddressAliasKind, RawAddressViewKind, RawBodyKind, ResourceBlockId, ResourceCallTarget,
    ResourceConditionFact, ResourceExprKind, ResourceFunction, ResourceFunctionValueKind,
    ResourceI32RelationOp, ResourceId, ResourceLocal, ResourceMatchArm, ResourceMatchBindMode,
    ResourceMatchPattern, ResourceOffset, ResourceOp, ResourceTerminator, StorageId, StorageOrigin,
};
use super::key::normalize_definition_span_mangle;
use super::stable_hash::ResourceSummaryStableHasher;
use super::stable_type_key::ResourceSummaryStableTypeKey;

// この module は store/hit 実装の直前に、Resource IR body の安定した識別子を固定する
// staging module である。cache map に接続するまではテストからだけ参照されるため、
// module 全体の未使用 warning はここで局所的に抑止する。

/// Resource summary value cache key に入れる Resource IR body hash。
///
/// `Span` は診断位置であり、Resource summary proof の意味を変えないため hash しない。
/// 一方で `TypeId` や一時値・storage・block の数値 ID は compile session 内の割当で
/// あり、そのまま保存境界へ出すと別 session の同じ body を別物として扱ったり、逆に
/// unsafe な対応付けを招いたりする。この関数は `TypeId` を stable type key に変換し、
/// temporary/block は body 内の ordinal に正規化する。raw wasm / LLVM body の本文は
/// `ResourceFunction` には残らないため、raw body/source hash を key に追加するまでは
/// body hash を作らず cache 候補から外す。
///
/// `StorageId` は owner/checker state 側の割当に由来するため、数値を直接 hash しない。
/// 関数本文の traversal で最初に現れた順へ正規化し、同じ本文内の storage root の同一性
/// だけを hash に残す。storage の所有元や寿命上の意味は `StorageOrigin` /
/// collection lifecycle op 側で別途 hash する。
///
/// 関数値は backend symbol だけではなく、型検査後の function type、effect、具体化済み
/// type args を合わせて hash する。`DefId` は compile session 内の補助 identity であり、
/// 長寿命 cache key へ直接保存しない。
///
/// raw body の本文文字列は `ResourceFunction` には残らないが、source text と raw body
/// capability use-site は `ResourceSummaryValueCacheContext` の source capability policy hash
/// に含まれる。そのため body hash では backend kind だけを固定し、cache key の caller が
/// source policy hash と組み合わせることを契約にする。
pub(super) fn resource_function_body_hash(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> Option<u64> {
    let mut ctx = ResourceFunctionBodyHashContext::new(types, function)?;
    let mut hash = ResourceSummaryStableHasher::new("neplg2-resource-function-body-v4");

    hash_type_list(&mut hash, &ctx, &function.type_params)?;
    hash_resource_locals(&mut hash, &mut ctx, &function.params)?;
    hash_type(&mut hash, &ctx, function.result)?;
    hash_effect(&mut hash, function.effect);
    hash_block_id(&mut hash, &ctx, function.entry_block)?;
    hash.write_usize(function.blocks.len());
    for block in &function.blocks {
        hash_block_id(&mut hash, &ctx, block.id)?;
        hash_ops(&mut hash, &mut ctx, &block.ops)?;
        hash_terminator(&mut hash, &mut ctx, &block.terminator)?;
    }

    Some(hash.finish())
}

struct ResourceFunctionBodyHashContext<'a> {
    types: &'a TypeCtx,
    block_ordinals: BTreeMap<ResourceBlockId, usize>,
    temporary_ordinals: BTreeMap<ResourceId, usize>,
    storage_ordinals: BTreeMap<StorageId, usize>,
}

impl<'a> ResourceFunctionBodyHashContext<'a> {
    fn new(types: &'a TypeCtx, function: &ResourceFunction) -> Option<Self> {
        let mut block_ordinals = BTreeMap::new();
        for (index, block) in function.blocks.iter().enumerate() {
            if block_ordinals.insert(block.id, index).is_some() {
                return None;
            }
        }
        if !block_ordinals.contains_key(&function.entry_block) {
            return None;
        }
        Some(Self {
            types,
            block_ordinals,
            temporary_ordinals: BTreeMap::new(),
            storage_ordinals: BTreeMap::new(),
        })
    }

    fn block_ordinal(&self, id: ResourceBlockId) -> Option<usize> {
        self.block_ordinals.get(&id).copied()
    }

    fn temporary_ordinal(&mut self, id: ResourceId) -> usize {
        if let Some(ordinal) = self.temporary_ordinals.get(&id) {
            return *ordinal;
        }
        let ordinal = self.temporary_ordinals.len();
        self.temporary_ordinals.insert(id, ordinal);
        ordinal
    }

    fn storage_ordinal(&mut self, id: StorageId) -> usize {
        if let Some(ordinal) = self.storage_ordinals.get(&id) {
            return *ordinal;
        }
        let ordinal = self.storage_ordinals.len();
        self.storage_ordinals.insert(id, ordinal);
        ordinal
    }
}

fn hash_type(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &ResourceFunctionBodyHashContext<'_>,
    ty: TypeId,
) -> Option<()> {
    let key = ResourceSummaryStableTypeKey::from_type(ctx.types, ty)?;
    hash.write_str(key.as_str());
    Some(())
}

fn hash_type_list(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &ResourceFunctionBodyHashContext<'_>,
    items: &[TypeId],
) -> Option<()> {
    hash.write_usize(items.len());
    for item in items {
        hash_type(hash, ctx, *item)?;
    }
    Some(())
}

fn hash_resource_locals(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    locals: &[ResourceLocal],
) -> Option<()> {
    hash.write_usize(locals.len());
    for local in locals {
        hash.write_str(&local.name);
        hash_type(hash, ctx, local.ty)?;
        hash.write_bool(local.mutable);
        hash_place(hash, ctx, &local.place)?;
    }
    Some(())
}

fn hash_block_id(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &ResourceFunctionBodyHashContext<'_>,
    id: ResourceBlockId,
) -> Option<()> {
    hash.write_usize(ctx.block_ordinal(id)?);
    Some(())
}

fn hash_ops(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    ops: &[ResourceOp],
) -> Option<()> {
    hash.write_usize(ops.len());
    for op in ops {
        hash_op(hash, ctx, op)?;
    }
    Some(())
}

fn hash_op(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    op: &ResourceOp,
) -> Option<()> {
    match op {
        ResourceOp::Expr {
            kind, output, ty, ..
        } => {
            hash.write_str("expr");
            hash_expr_kind(hash, ctx, *kind)?;
            hash_place(hash, ctx, output)?;
            hash_type(hash, ctx, *ty)?;
        }
        ResourceOp::DeclareLocal {
            place,
            source_name,
            mutable,
            initializer,
            ..
        } => {
            hash.write_str("declare_local");
            hash_place(hash, ctx, place)?;
            hash.write_str(source_name);
            hash.write_bool(*mutable);
            hash_optional_place(hash, ctx, initializer)?;
        }
        ResourceOp::Read { source, output, .. } => {
            hash.write_str("read");
            hash_place(hash, ctx, source)?;
            hash_place(hash, ctx, output)?;
        }
        ResourceOp::Assign { target, value, .. } => {
            hash.write_str("assign");
            hash_place(hash, ctx, target)?;
            hash_place(hash, ctx, value)?;
        }
        ResourceOp::Borrow {
            source,
            output,
            kind,
            synthetic,
            ..
        } => {
            hash.write_str("borrow");
            hash_place(hash, ctx, source)?;
            hash_place(hash, ctx, output)?;
            hash_borrow_kind(hash, *kind);
            hash.write_bool(*synthetic);
        }
        ResourceOp::Move { source, output, .. } => {
            hash.write_str("move");
            hash_place(hash, ctx, source)?;
            hash_place(hash, ctx, output)?;
        }
        ResourceOp::Drop { place, .. } => {
            hash.write_str("drop");
            hash_place(hash, ctx, place)?;
        }
        ResourceOp::EndScope { locals, result, .. } => {
            hash.write_str("end_scope");
            hash_places(hash, ctx, locals)?;
            hash_optional_place(hash, ctx, result)?;
        }
        ResourceOp::CallEffect { effect, .. } => {
            hash.write_str("call_effect");
            hash_effect_op(hash, effect);
        }
        ResourceOp::FunctionValue {
            output,
            identity,
            value_kind,
            effect,
            ..
        } => {
            hash.write_str("function_value");
            hash_function_value_kind(hash, *value_kind);
            hash_place(hash, ctx, output)?;
            hash_function_value_identity(hash, ctx, identity)?;
            hash_effect_op(hash, effect);
        }
        ResourceOp::Call {
            output,
            target,
            args,
            effect,
            ..
        } => {
            hash.write_str("call");
            hash_place(hash, ctx, output)?;
            hash_call_target(hash, ctx, target)?;
            hash_places(hash, ctx, args)?;
            hash_effect_op(hash, effect);
        }
        ResourceOp::IndirectCall {
            output,
            callee,
            params,
            result,
            args,
            effect,
            ..
        } => {
            hash.write_str("indirect_call");
            hash_place(hash, ctx, output)?;
            hash_place(hash, ctx, callee)?;
            hash_type_list(hash, ctx, params)?;
            hash_type(hash, ctx, *result)?;
            hash_places(hash, ctx, args)?;
            hash_effect_op(hash, effect);
        }
        ResourceOp::RawMemory {
            operation,
            output,
            args,
            ..
        } => {
            hash.write_str("raw_memory");
            hash.write_str(operation.as_str());
            hash_place(hash, ctx, output)?;
            hash_places(hash, ctx, args)?;
        }
        ResourceOp::RawAddressAlias {
            source,
            target,
            kind,
            ..
        } => {
            hash.write_str("raw_address_alias");
            hash_place(hash, ctx, source)?;
            hash_place(hash, ctx, target)?;
            hash_raw_address_alias_kind(hash, *kind);
        }
        ResourceOp::RawAddressView {
            source,
            target,
            kind,
            ..
        } => {
            hash.write_str("raw_address_view");
            hash_place(hash, ctx, source)?;
            hash_place(hash, ctx, target)?;
            hash_raw_address_view_kind(hash, *kind);
        }
        ResourceOp::StorageOrigin { target, origin, .. } => {
            hash.write_str("storage_origin");
            hash_place(hash, ctx, target)?;
            hash_storage_origin(hash, *origin);
        }
        ResourceOp::CollectionSlotLifecycle { target, event, .. } => {
            hash.write_str("collection_slot_lifecycle");
            hash_place(hash, ctx, target)?;
            hash_collection_slot_lifecycle_event(hash, ctx, *event)?;
        }
        ResourceOp::CollectionStorageRelocate {
            old_storage,
            new_storage,
            ..
        } => {
            hash.write_str("collection_storage_relocate");
            hash_place(hash, ctx, old_storage)?;
            hash_place(hash, ctx, new_storage)?;
        }
        ResourceOp::CollectionSlotDropTraversal {
            storage,
            initialized_count,
            expected_ty,
            ..
        } => {
            hash.write_str("collection_slot_drop_traversal");
            hash_place(hash, ctx, storage)?;
            hash_place(hash, ctx, initialized_count)?;
            hash_type(hash, ctx, *expected_ty)?;
        }
        ResourceOp::CollectionSlotTransformRange {
            source_storage,
            source_initialized_count,
            output_storage,
            output_initialized_count,
            expected_ty,
            ..
        } => {
            hash.write_str("collection_slot_transform_range");
            hash_place(hash, ctx, source_storage)?;
            hash_place(hash, ctx, source_initialized_count)?;
            hash_place(hash, ctx, output_storage)?;
            hash_place(hash, ctx, output_initialized_count)?;
            hash_type(hash, ctx, *expected_ty)?;
        }
        ResourceOp::Construct {
            output,
            kind,
            inputs,
            ..
        } => {
            hash.write_str("construct");
            hash_place(hash, ctx, output)?;
            hash_aggregate_kind(hash, kind);
            hash_places(hash, ctx, inputs)?;
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
            hash.write_str("branch");
            hash_place(hash, ctx, output)?;
            hash_place(hash, ctx, condition)?;
            hash_optional_condition_fact(hash, ctx, condition_fact)?;
            hash_ops(hash, ctx, then_ops)?;
            hash_place(hash, ctx, then_value)?;
            hash_ops(hash, ctx, else_ops)?;
            hash_place(hash, ctx, else_value)?;
        }
        ResourceOp::Loop {
            condition_ops,
            condition,
            condition_fact,
            body_ops,
            ..
        } => {
            hash.write_str("loop");
            hash_ops(hash, ctx, condition_ops)?;
            hash_place(hash, ctx, condition)?;
            hash_optional_condition_fact(hash, ctx, condition_fact)?;
            hash_ops(hash, ctx, body_ops)?;
        }
        ResourceOp::Match {
            output,
            scrutinee,
            scrutinee_is_borrow_target,
            arms,
            ..
        } => {
            hash.write_str("match");
            hash_place(hash, ctx, output)?;
            hash_place(hash, ctx, scrutinee)?;
            hash.write_bool(*scrutinee_is_borrow_target);
            hash_match_arms(hash, ctx, arms)?;
        }
    }
    Some(())
}

fn hash_function_value_identity(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &ResourceFunctionBodyHashContext<'_>,
    identity: &FunctionValueIdentity,
) -> Option<()> {
    hash_resource_function_symbol(hash, identity.symbol());
    hash_type(hash, ctx, identity.function_ty)?;
    hash_effect(hash, identity.effect);
    hash_type_list(hash, ctx, &identity.type_args)?;
    Some(())
}

fn hash_function_value_kind(
    hash: &mut ResourceSummaryStableHasher,
    value_kind: ResourceFunctionValueKind,
) {
    match value_kind {
        ResourceFunctionValueKind::Plain => hash.write_str("plain"),
        ResourceFunctionValueKind::Memoized => hash.write_str("memoized"),
    }
}

fn hash_terminator(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    terminator: &ResourceTerminator,
) -> Option<()> {
    match terminator {
        ResourceTerminator::Return { value, .. } => {
            hash.write_str("return");
            hash_optional_place(hash, ctx, value)?;
        }
        ResourceTerminator::Unreachable { .. } => {
            hash.write_str("unreachable");
        }
        ResourceTerminator::RawBody { kind, .. } => {
            hash.write_str("raw_body");
            hash_raw_body_kind(hash, *kind);
        }
    }
    Some(())
}

fn hash_expr_kind(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &ResourceFunctionBodyHashContext<'_>,
    kind: ResourceExprKind,
) -> Option<()> {
    match kind {
        ResourceExprKind::Literal => hash.write_str("literal"),
        ResourceExprKind::LiteralI32(value) => {
            hash.write_str("literal_i32");
            hash.write_i32(value);
        }
        ResourceExprKind::LayoutSizeOf(ty) => {
            hash.write_str("layout_size_of");
            hash_type(hash, ctx, ty)?;
        }
        ResourceExprKind::LocalRead => hash.write_str("local_read"),
        ResourceExprKind::FunctionValue => hash.write_str("function_value"),
        ResourceExprKind::Call => hash.write_str("call"),
        ResourceExprKind::IndirectCall => hash.write_str("indirect_call"),
        ResourceExprKind::Branch => hash.write_str("branch"),
        ResourceExprKind::Loop => hash.write_str("loop"),
        ResourceExprKind::Match => hash.write_str("match"),
        ResourceExprKind::Construct => hash.write_str("construct"),
        ResourceExprKind::Block => hash.write_str("block"),
        ResourceExprKind::Let => hash.write_str("let"),
        ResourceExprKind::Set => hash.write_str("set"),
        ResourceExprKind::Intrinsic => hash.write_str("intrinsic"),
        ResourceExprKind::Borrow => hash.write_str("borrow"),
        ResourceExprKind::Deref => hash.write_str("deref"),
        ResourceExprKind::Drop => hash.write_str("drop"),
    }
    Some(())
}

fn hash_places(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    places: &[Place],
) -> Option<()> {
    hash.write_usize(places.len());
    for place in places {
        hash_place(hash, ctx, place)?;
    }
    Some(())
}

fn hash_optional_place(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    place: &Option<Place>,
) -> Option<()> {
    match place {
        Some(place) => {
            hash.write_str("some");
            hash_place(hash, ctx, place)?;
        }
        None => hash.write_str("none"),
    }
    Some(())
}

fn hash_place(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    place: &Place,
) -> Option<()> {
    hash_place_root(hash, ctx, &place.root)?;
    hash.write_usize(place.projections.len());
    for projection in &place.projections {
        hash_place_projection(hash, ctx, projection)?;
    }
    hash_type(hash, ctx, place.ty)?;
    Some(())
}

fn hash_place_root(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    root: &PlaceRoot,
) -> Option<()> {
    match root {
        PlaceRoot::Local(name) => {
            hash.write_str("local");
            hash.write_str(name);
        }
        PlaceRoot::Temporary(id) => {
            hash.write_str("temporary");
            hash.write_usize(ctx.temporary_ordinal(*id));
        }
        PlaceRoot::I32Constant(value) => {
            hash.write_str("i32_constant");
            hash.write_i32(*value);
        }
        PlaceRoot::Return => hash.write_str("return_place"),
        PlaceRoot::Storage(id) => {
            hash.write_str("storage");
            hash.write_usize(ctx.storage_ordinal(*id));
        }
        PlaceRoot::Unknown => hash.write_str("unknown"),
    }
    Some(())
}

fn hash_place_projection(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    projection: &PlaceProjection,
) -> Option<()> {
    match projection {
        PlaceProjection::Field {
            index,
            offset_bytes,
        } => {
            hash.write_str("field");
            hash.write_usize(*index);
            hash.write_usize(*offset_bytes);
        }
        PlaceProjection::TupleField {
            index,
            offset_bytes,
        } => {
            hash.write_str("tuple_field");
            hash.write_usize(*index);
            hash.write_usize(*offset_bytes);
        }
        PlaceProjection::EnumPayload { variant } => {
            hash.write_str("enum_payload");
            hash.write_str(variant);
        }
        PlaceProjection::Deref => hash.write_str("deref"),
        PlaceProjection::StorageOffset(offset) => {
            hash.write_str("storage_offset");
            hash_resource_offset(hash, ctx, offset)?;
        }
    }
    Some(())
}

fn hash_resource_offset(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    offset: &ResourceOffset,
) -> Option<()> {
    match offset {
        ResourceOffset::Known(value) => {
            hash.write_str("known");
            hash.write_usize(*value);
        }
        ResourceOffset::Symbolic { place } => {
            hash.write_str("symbolic");
            hash_place(hash, ctx, place)?;
        }
        ResourceOffset::ScaledSymbolic { place, scale } => {
            hash.write_str("scaled_symbolic");
            hash_place(hash, ctx, place)?;
            hash.write_usize(*scale);
        }
        ResourceOffset::Offset { place, offset } => {
            hash.write_str("offset");
            hash_place(hash, ctx, place)?;
            hash.write_i64(*offset);
        }
        ResourceOffset::ScaledOffset {
            place,
            offset,
            scale,
        } => {
            hash.write_str("scaled_offset");
            hash_place(hash, ctx, place)?;
            hash.write_i64(*offset);
            hash.write_usize(*scale);
        }
        ResourceOffset::Unknown => hash.write_str("unknown"),
    }
    Some(())
}

fn hash_call_target(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &ResourceFunctionBodyHashContext<'_>,
    target: &ResourceCallTarget,
) -> Option<()> {
    match target {
        ResourceCallTarget::Builtin { name } => {
            hash.write_str("builtin");
            hash.write_str(name);
        }
        ResourceCallTarget::User { name, type_args } => {
            hash.write_str("user");
            hash_resource_function_symbol(hash, name);
            hash_type_list(hash, ctx, type_args)?;
        }
        ResourceCallTarget::Trait {
            application,
            method,
            self_ty,
        } => {
            hash.write_str("trait");
            hash.write_str(application.trait_id.as_str());
            hash_type_list(hash, ctx, &application.args)?;
            hash.write_str(method.as_str());
            hash_type(hash, ctx, *self_ty)?;
        }
    }
    Some(())
}

fn hash_aggregate_kind(hash: &mut ResourceSummaryStableHasher, kind: &AggregateKind) {
    match kind {
        AggregateKind::Enum { name, variant } => {
            hash.write_str("enum");
            hash.write_str(name);
            hash.write_str(variant);
        }
        AggregateKind::Struct {
            name,
            field_offsets,
        } => {
            hash.write_str("struct");
            hash.write_str(name);
            hash_usize_list(hash, field_offsets);
        }
        AggregateKind::Tuple { field_offsets } => {
            hash.write_str("tuple");
            hash_usize_list(hash, field_offsets);
        }
    }
}

fn hash_match_arms(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    arms: &[ResourceMatchArm],
) -> Option<()> {
    hash.write_usize(arms.len());
    for arm in arms {
        hash_match_pattern(hash, &arm.pattern);
        hash_optional_place(hash, ctx, &arm.bind_local)?;
        hash_optional_str(hash, arm.bind_source_name.as_deref());
        hash_optional_match_bind_mode(hash, arm.bind_mode);
        hash_ops(hash, ctx, &arm.ops)?;
        hash_place(hash, ctx, &arm.value)?;
    }
    Some(())
}

fn hash_match_pattern(hash: &mut ResourceSummaryStableHasher, pattern: &ResourceMatchPattern) {
    match pattern {
        ResourceMatchPattern::Variant(variant) => {
            hash.write_str("variant");
            hash.write_str(variant);
        }
        ResourceMatchPattern::IntLiteral(value) => {
            hash.write_str("int_literal");
            hash.write_i32(*value);
        }
        ResourceMatchPattern::BoolLiteral(value) => {
            hash.write_str("bool_literal");
            hash.write_bool(*value);
        }
        ResourceMatchPattern::Wildcard => hash.write_str("wildcard"),
    }
}

fn hash_optional_match_bind_mode(
    hash: &mut ResourceSummaryStableHasher,
    mode: Option<ResourceMatchBindMode>,
) {
    match mode {
        Some(ResourceMatchBindMode::Owned) => hash.write_str("owned"),
        Some(ResourceMatchBindMode::Borrowed { is_mut }) => {
            hash.write_str("borrowed");
            hash.write_bool(is_mut);
        }
        None => hash.write_str("none"),
    }
}

fn hash_optional_condition_fact(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    fact: &Option<ResourceConditionFact>,
) -> Option<()> {
    match fact {
        Some(fact) => {
            hash.write_str("some");
            hash_condition_fact(hash, ctx, fact)?;
        }
        None => hash.write_str("none"),
    }
    Some(())
}

fn hash_condition_fact(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    fact: &ResourceConditionFact,
) -> Option<()> {
    match fact {
        ResourceConditionFact::EqZero { place } => {
            hash.write_str("eq_zero");
            hash_place(hash, ctx, place)?;
        }
        ResourceConditionFact::NeZero { place } => {
            hash.write_str("ne_zero");
            hash_place(hash, ctx, place)?;
        }
        ResourceConditionFact::Positive { place } => {
            hash.write_str("positive");
            hash_place(hash, ctx, place)?;
        }
        ResourceConditionFact::NonPositive { place } => {
            hash.write_str("non_positive");
            hash_place(hash, ctx, place)?;
        }
        ResourceConditionFact::Negative { place } => {
            hash.write_str("negative");
            hash_place(hash, ctx, place)?;
        }
        ResourceConditionFact::NonNegative { place } => {
            hash.write_str("non_negative");
            hash_place(hash, ctx, place)?;
        }
        ResourceConditionFact::I32Relation { left, op, right } => {
            hash.write_str("i32_relation");
            hash_place(hash, ctx, left)?;
            hash_i32_relation_op(hash, *op);
            hash_place(hash, ctx, right)?;
        }
        ResourceConditionFact::Any(facts) => {
            hash.write_str("any");
            hash_condition_facts(hash, ctx, facts)?;
        }
        ResourceConditionFact::All(facts) => {
            hash.write_str("all");
            hash_condition_facts(hash, ctx, facts)?;
        }
    }
    Some(())
}

fn hash_condition_facts(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &mut ResourceFunctionBodyHashContext<'_>,
    facts: &[ResourceConditionFact],
) -> Option<()> {
    hash.write_usize(facts.len());
    for fact in facts {
        hash_condition_fact(hash, ctx, fact)?;
    }
    Some(())
}

fn hash_collection_slot_lifecycle_event(
    hash: &mut ResourceSummaryStableHasher,
    ctx: &ResourceFunctionBodyHashContext<'_>,
    event: CollectionSlotLifecycleEvent,
) -> Option<()> {
    match event {
        CollectionSlotLifecycleEvent::InitializeEmpty { value_ty } => {
            hash.write_str("initialize_empty");
            hash_type(hash, ctx, value_ty)?;
        }
        CollectionSlotLifecycleEvent::BorrowRead { expected_ty } => {
            hash.write_str("borrow_read");
            hash_type(hash, ctx, expected_ty)?;
        }
        CollectionSlotLifecycleEvent::MoveOut { expected_ty } => {
            hash.write_str("move_out");
            hash_type(hash, ctx, expected_ty)?;
        }
        CollectionSlotLifecycleEvent::ReplaceInitialized {
            old_ty,
            new_ty,
            old_owner,
        } => {
            hash.write_str("replace_initialized");
            hash_type(hash, ctx, old_ty)?;
            hash_type(hash, ctx, new_ty)?;
            hash_collection_slot_replacement(hash, old_owner);
        }
        CollectionSlotLifecycleEvent::DropInitialized { expected_ty } => {
            hash.write_str("drop_initialized");
            hash_type(hash, ctx, expected_ty)?;
        }
        CollectionSlotLifecycleEvent::StorageDealloc { value_ty } => {
            hash.write_str("storage_dealloc");
            hash_type(hash, ctx, value_ty)?;
        }
    }
    Some(())
}

fn hash_collection_slot_replacement(
    hash: &mut ResourceSummaryStableHasher,
    replacement: CollectionSlotReplacement,
) {
    match replacement {
        CollectionSlotReplacement::ReturnOldOwner => hash.write_str("return_old_owner"),
        CollectionSlotReplacement::DropOldOwner => hash.write_str("drop_old_owner"),
    }
}

fn hash_effect_op(hash: &mut ResourceSummaryStableHasher, effect: &EffectOp) {
    match effect {
        EffectOp::Pure => hash.write_str("pure"),
        EffectOp::UserCall { name, effect } => {
            hash.write_str("user_call");
            hash_resource_function_symbol(hash, name);
            hash_effect(hash, *effect);
        }
        EffectOp::IndirectCall { effect } => {
            hash.write_str("indirect_call");
            hash_effect(hash, *effect);
        }
        EffectOp::InternalAlloc { operation } => {
            hash.write_str("internal_alloc");
            hash.write_str(operation.as_str());
        }
        EffectOp::UnsafeMemory { operation } => {
            hash.write_str("unsafe_memory");
            hash.write_str(operation.as_str());
        }
        EffectOp::PrivateState { operation, region } => {
            hash.write_str("private_state");
            hash.write_str(operation.as_str());
            hash_private_effect_region(hash, *region);
        }
        EffectOp::PrivateCache { operation, region } => {
            hash.write_str("private_cache");
            hash.write_str(operation.as_str());
            hash_private_effect_region(hash, *region);
        }
        EffectOp::ExternalIo { operation } => {
            hash.write_str("external_io");
            hash.write_str(operation.as_str());
        }
        EffectOp::Nondet { operation } => {
            hash.write_str("nondet");
            hash.write_str(operation.as_str());
        }
        EffectOp::Unknown { reason } => {
            hash.write_str("unknown");
            hash.write_str(reason.as_str());
        }
    }
}

fn hash_private_effect_region(
    hash: &mut ResourceSummaryStableHasher,
    region: PrivateEffectRegion,
) {
    hash.write_str(region.as_str());
    if let Some(id) = region.numeric_id() {
        hash.write_u64(u64::from(id));
    }
}

fn hash_effect(hash: &mut ResourceSummaryStableHasher, effect: Effect) {
    match effect {
        Effect::Pure => hash.write_str("pure"),
        Effect::Impure => hash.write_str("impure"),
    }
}

fn hash_raw_body_kind(hash: &mut ResourceSummaryStableHasher, kind: RawBodyKind) {
    match kind {
        RawBodyKind::Wasm => hash.write_str("wasm"),
        RawBodyKind::LlvmIr => hash.write_str("llvmir"),
    }
}

fn hash_borrow_kind(hash: &mut ResourceSummaryStableHasher, kind: BorrowKind) {
    match kind {
        BorrowKind::Shared => hash.write_str("shared"),
        BorrowKind::Unique => hash.write_str("unique"),
    }
}

fn hash_raw_address_alias_kind(hash: &mut ResourceSummaryStableHasher, kind: RawAddressAliasKind) {
    match kind {
        RawAddressAliasKind::Transparent => hash.write_str("transparent"),
        RawAddressAliasKind::InternalHelper => hash.write_str("internal_helper"),
        RawAddressAliasKind::OwnerTokenConstruct => hash.write_str("owner_token_construct"),
    }
}

fn hash_raw_address_view_kind(hash: &mut ResourceSummaryStableHasher, kind: RawAddressViewKind) {
    match kind {
        RawAddressViewKind::Offset => hash.write_str("offset"),
        RawAddressViewKind::MemPtrOffset => hash.write_str("mem_ptr_offset"),
        RawAddressViewKind::NonOwningProjection => hash.write_str("non_owning_projection"),
        RawAddressViewKind::InternalHelper => hash.write_str("internal_helper"),
    }
}

fn hash_storage_origin(hash: &mut ResourceSummaryStableHasher, origin: StorageOrigin) {
    match origin {
        StorageOrigin::Owned => hash.write_str("owned"),
        StorageOrigin::Unmanaged => hash.write_str("unmanaged"),
        StorageOrigin::Internal => hash.write_str("internal"),
    }
}

fn hash_i32_relation_op(hash: &mut ResourceSummaryStableHasher, op: ResourceI32RelationOp) {
    match op {
        ResourceI32RelationOp::Eq => hash.write_str("eq"),
        ResourceI32RelationOp::Ne => hash.write_str("ne"),
        ResourceI32RelationOp::Lt => hash.write_str("lt"),
        ResourceI32RelationOp::Le => hash.write_str("le"),
        ResourceI32RelationOp::Gt => hash.write_str("gt"),
        ResourceI32RelationOp::Ge => hash.write_str("ge"),
    }
}

fn hash_i32_value_condition(hash: &mut ResourceSummaryStableHasher, condition: I32ValueCondition) {
    match condition {
        I32ValueCondition::EqZero => hash.write_str("eq_zero"),
        I32ValueCondition::NeZero => hash.write_str("ne_zero"),
        I32ValueCondition::Positive => hash.write_str("positive"),
        I32ValueCondition::NonPositive => hash.write_str("non_positive"),
        I32ValueCondition::Negative => hash.write_str("negative"),
        I32ValueCondition::NonNegative => hash.write_str("non_negative"),
    }
}

fn hash_optional_str(hash: &mut ResourceSummaryStableHasher, value: Option<&str>) {
    match value {
        Some(value) => {
            hash.write_str("some");
            hash.write_str(value);
        }
        None => hash.write_str("none"),
    }
}

fn hash_resource_function_symbol(hash: &mut ResourceSummaryStableHasher, symbol: &str) {
    let normalized = normalize_definition_span_mangle(symbol);
    hash.write_str(&normalized);
}

fn hash_usize_list(hash: &mut ResourceSummaryStableHasher, values: &[usize]) {
    hash.write_usize(values.len());
    for value in values {
        hash.write_usize(*value);
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::effects::{PrivateCacheOp, PrivateEffectRegion, PrivateEffectRegionId};
    use crate::span::{FileId, Span};

    use super::super::super::model::StorageId;
    use super::*;

    fn simple_function(
        types: &TypeCtx,
        temp_id: usize,
        literal: i32,
        span_base: u32,
    ) -> ResourceFunction {
        let ty = types.i32();
        let output = Place::temporary(ResourceId(temp_id), ty);
        ResourceFunction {
            name: "example".into(),
            origin_name: "example".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![super::super::super::model::ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::LiteralI32(literal),
                    output: output.clone(),
                    ty,
                    span: Span::new(FileId(1), span_base, span_base + 1),
                }],
                terminator: ResourceTerminator::Return {
                    value: Some(output),
                    span: Span::new(FileId(1), span_base + 1, span_base + 2),
                },
                span: Span::new(FileId(1), span_base, span_base + 2),
            }],
            span: Span::new(FileId(1), span_base, span_base + 2),
        }
    }

    fn function_value_body(
        output_ty: TypeId,
        identity: FunctionValueIdentity,
        value_kind: ResourceFunctionValueKind,
    ) -> ResourceFunction {
        let output = Place::temporary(ResourceId(0), output_ty);
        ResourceFunction {
            name: "function_value_holder".into(),
            origin_name: "function_value_holder".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: output_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![super::super::super::model::ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::FunctionValue {
                    output: output.clone(),
                    name: "legacy_function_name".into(),
                    identity,
                    value_kind,
                    effect: EffectOp::Pure,
                    span: Span::dummy(),
                }],
                terminator: ResourceTerminator::Return {
                    value: Some(output),
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }
    }

    fn call_body(types: &TypeCtx, target_name: &str) -> ResourceFunction {
        let ty = types.i32();
        let output = Place::temporary(ResourceId(0), ty);
        ResourceFunction {
            name: "caller".into(),
            origin_name: "caller".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![super::super::super::model::ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::Call {
                    output: output.clone(),
                    target: ResourceCallTarget::User {
                        name: target_name.into(),
                        type_args: Vec::new(),
                    },
                    args: Vec::new(),
                    effect: EffectOp::UserCall {
                        name: target_name.into(),
                        effect: Effect::Pure,
                    },
                    span: Span::dummy(),
                }],
                terminator: ResourceTerminator::Return {
                    value: Some(output),
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }
    }

    fn call_effect_body(types: &TypeCtx, effect: EffectOp) -> ResourceFunction {
        ResourceFunction {
            name: "effect_holder".into(),
            origin_name: "effect_holder".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: types.i32(),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![super::super::super::model::ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::CallEffect {
                    effect,
                    span: Span::dummy(),
                }],
                terminator: ResourceTerminator::Return {
                    value: None,
                    span: Span::dummy(),
                },
                span: Span::dummy(),
            }],
            span: Span::dummy(),
        }
    }

    #[test]
    fn resource_function_body_hash_ignores_spans() {
        let types = TypeCtx::new();
        let first = simple_function(&types, 0, 1, 0);
        let second = simple_function(&types, 0, 1, 100);

        assert_eq!(
            resource_function_body_hash(&types, &first),
            resource_function_body_hash(&types, &second)
        );
    }

    #[test]
    fn resource_function_body_hash_tracks_body_operations() {
        let types = TypeCtx::new();
        let first = simple_function(&types, 0, 1, 0);
        let second = simple_function(&types, 0, 2, 0);

        assert_ne!(
            resource_function_body_hash(&types, &first),
            resource_function_body_hash(&types, &second)
        );
    }

    #[test]
    fn resource_function_body_stable_hash_uses_body_hash_authority() {
        let types = TypeCtx::new();
        let function = simple_function(&types, 0, 1, 0);

        assert_eq!(
            super::super::resource_function_body_stable_hash(&types, &function),
            resource_function_body_hash(&types, &function)
        );
    }

    #[test]
    fn resource_function_body_hash_normalizes_function_local_ids() {
        let types = TypeCtx::new();
        let first = simple_function(&types, 7, 1, 0);
        let second = simple_function(&types, 99, 1, 0);

        assert_eq!(
            resource_function_body_hash(&types, &first),
            resource_function_body_hash(&types, &second)
        );
    }

    /// Resource summary proof の再利用では、operation と Resource IR body が同じでも
    /// sealed private cache region が違えば別 proof として扱う必要がある。
    #[test]
    fn resource_function_body_hash_tracks_private_cache_region_identity() {
        let types = TypeCtx::new();
        let first = call_effect_body(
            &types,
            EffectOp::PrivateCache {
                operation: PrivateCacheOp::Lookup,
                region: PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(1)),
            },
        );
        let second = call_effect_body(
            &types,
            EffectOp::PrivateCache {
                operation: PrivateCacheOp::Lookup,
                region: PrivateEffectRegion::SealedCompilerPrivateCache(PrivateEffectRegionId(2)),
            },
        );

        assert_ne!(
            resource_function_body_hash(&types, &first),
            resource_function_body_hash(&types, &second)
        );
    }

    #[test]
    fn resource_function_body_hash_tracks_function_value_typed_identity() {
        let mut types = TypeCtx::new();
        let function_ty = types.function(vec![], vec![types.i32()], types.i32(), Effect::Pure);
        let first = function_value_body(
            function_ty,
            FunctionValueIdentity::new(
                "same_backend_symbol".into(),
                None,
                function_ty,
                Effect::Pure,
                vec![types.i32()],
            ),
            ResourceFunctionValueKind::Plain,
        );
        let second = function_value_body(
            function_ty,
            FunctionValueIdentity::new(
                "same_backend_symbol".into(),
                None,
                function_ty,
                Effect::Pure,
                vec![types.bool()],
            ),
            ResourceFunctionValueKind::Plain,
        );

        assert_ne!(
            resource_function_body_hash(&types, &first),
            resource_function_body_hash(&types, &second)
        );
    }

    #[test]
    fn resource_function_body_hash_tracks_memoized_function_value_kind() {
        let mut types = TypeCtx::new();
        let function_ty = types.function(vec![], vec![types.i32()], types.i32(), Effect::Pure);
        let identity = FunctionValueIdentity::new(
            "same_backend_symbol".into(),
            None,
            function_ty,
            Effect::Pure,
            vec![types.i32()],
        );
        let plain = function_value_body(
            function_ty,
            identity.clone(),
            ResourceFunctionValueKind::Plain,
        );
        let memoized =
            function_value_body(function_ty, identity, ResourceFunctionValueKind::Memoized);

        assert_ne!(
            resource_function_body_hash(&types, &plain),
            resource_function_body_hash(&types, &memoized)
        );
    }

    #[test]
    fn resource_function_body_hash_normalizes_called_function_def_span_mangle() {
        let types = TypeCtx::new();
        let first = call_body(&types, "callee__def7_10_20__i32__i32__pure");
        let second = call_body(&types, "callee__def7_12_22__i32__i32__pure");

        assert_eq!(
            resource_function_body_hash(&types, &first),
            resource_function_body_hash(&types, &second)
        );
    }

    #[test]
    fn resource_function_body_hash_normalizes_function_value_def_span_mangle() {
        let mut types = TypeCtx::new();
        let function_ty = types.function(vec![], vec![types.i32()], types.i32(), Effect::Pure);
        let first = function_value_body(
            function_ty,
            FunctionValueIdentity::new(
                "callee__def7_10_20__i32__i32__pure".into(),
                None,
                function_ty,
                Effect::Pure,
                vec![types.i32()],
            ),
            ResourceFunctionValueKind::Plain,
        );
        let second = function_value_body(
            function_ty,
            FunctionValueIdentity::new(
                "callee__def7_12_22__i32__i32__pure".into(),
                None,
                function_ty,
                Effect::Pure,
                vec![types.i32()],
            ),
            ResourceFunctionValueKind::Plain,
        );

        assert_eq!(
            resource_function_body_hash(&types, &first),
            resource_function_body_hash(&types, &second)
        );
    }

    #[test]
    fn resource_function_body_hash_normalizes_storage_roots() {
        let types = TypeCtx::new();
        let ty = types.i32();
        let storage_function = |storage: StorageId| {
            let output = Place {
                root: PlaceRoot::Storage(storage),
                projections: Vec::new(),
                ty,
            };
            ResourceFunction {
                name: "storage".into(),
                origin_name: "storage".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                result: ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![super::super::super::model::ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: Vec::new(),
                    terminator: ResourceTerminator::Return {
                        value: Some(output),
                        span: Span::dummy(),
                    },
                    span: Span::dummy(),
                }],
                span: Span::dummy(),
            }
        };

        assert_eq!(
            resource_function_body_hash(&types, &storage_function(StorageId(0))),
            resource_function_body_hash(&types, &storage_function(StorageId(99)))
        );
    }

    #[test]
    fn resource_function_body_hash_accepts_raw_body_kind_with_external_source_policy() {
        let types = TypeCtx::new();
        let raw_function = |kind| ResourceFunction {
            name: "raw".into(),
            origin_name: "raw".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: types.i32(),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![super::super::super::model::ResourceBlock {
                id: ResourceBlockId(0),
                ops: Vec::new(),
                terminator: ResourceTerminator::RawBody {
                    kind,
                    span: Span::new(FileId(1), 10, 20),
                },
                span: Span::new(FileId(1), 0, 20),
            }],
            span: Span::new(FileId(1), 0, 20),
        };

        let wasm = raw_function(super::super::super::model::RawBodyKind::Wasm);
        let llvm = raw_function(super::super::super::model::RawBodyKind::LlvmIr);

        assert!(resource_function_body_hash(&types, &wasm).is_some());
        assert_ne!(
            resource_function_body_hash(&types, &wasm),
            resource_function_body_hash(&types, &llvm)
        );
    }

    #[test]
    fn resource_function_body_hash_rejects_unstable_type_variables() {
        let mut types = TypeCtx::new();
        let ty = types.fresh_var(None);
        let function = ResourceFunction {
            name: "unstable".into(),
            origin_name: "unstable".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![super::super::super::model::ResourceBlock {
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

        assert!(resource_function_body_hash(&types, &function).is_none());
    }
}
