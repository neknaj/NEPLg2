extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_i32_condition_context::I32ConditionQueryContext;
use super::model::{
    I32ValueCondition, Place, PlaceProjection, ResourceI32RelationOp, ResourceLocal,
};
use super::owner_summary_i32_condition_leaf::I32LeafProjectionCache;
use super::owner_summary_leaf::OwnerLeafPlace;
use super::place_utils::{place_suffix_after_prefix, projected_place_with_concrete_type};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct I32ScalarReturnFacts {
    pub(super) aliases: Vec<I32ScalarReturnAlias>,
    pub(super) offsets: Vec<I32ScalarReturnOffset>,
    pub(super) relations: Vec<I32ScalarReturnRelation>,
    pub(super) constants: Vec<I32ScalarReturnConstant>,
    pub(super) return_conditions: Vec<I32ScalarReturnCondition>,
    pub(super) parameter_conditions: Vec<I32ScalarParameterCondition>,
}

impl I32ScalarReturnFacts {
    pub(super) fn is_empty(&self) -> bool {
        self.aliases.is_empty()
            && self.offsets.is_empty()
            && self.relations.is_empty()
            && self.constants.is_empty()
            && self.return_conditions.is_empty()
            && self.parameter_conditions.is_empty()
    }

    pub(super) fn len(&self) -> usize {
        self.aliases.len()
            + self.offsets.len()
            + self.relations.len()
            + self.constants.len()
            + self.return_conditions.len()
            + self.parameter_conditions.len()
    }

    pub(super) fn extend(&mut self, other: I32ScalarReturnFacts) {
        for alias in other.aliases {
            push_unique_i32_scalar_return_alias(&mut self.aliases, alias);
        }
        for offset in other.offsets {
            push_unique_i32_scalar_return_offset(&mut self.offsets, offset);
        }
        for relation in other.relations {
            push_unique_i32_scalar_return_relation(&mut self.relations, relation);
        }
        for constant in other.constants {
            push_unique_i32_scalar_return_constant(&mut self.constants, constant);
        }
        for condition in other.return_conditions {
            push_unique_i32_scalar_return_condition(&mut self.return_conditions, condition);
        }
        for condition in other.parameter_conditions {
            push_unique_i32_scalar_parameter_condition(&mut self.parameter_conditions, condition);
        }
    }

    /// 要約に含まれる fact を種類ごとに数える。
    ///
    /// cache replay が失敗したとき、単純な合計だけでは alias / condition / constant の
    /// どの再投影規則が残差になっているかを判別できない。この集計は通常の検査結果には
    /// 影響せず、再利用できなかった proof の種類を観測するために使う。
    pub(super) fn fact_counts(&self) -> I32ScalarReturnFactCounts {
        I32ScalarReturnFactCounts {
            aliases: self.aliases.len(),
            offsets: self.offsets.len(),
            relations: self.relations.len(),
            constants: self.constants.len(),
            return_conditions: self.return_conditions.len(),
            parameter_conditions: self.parameter_conditions.len(),
        }
    }
}

/// i32 scalar return summary に保存された fact の種類別件数。
///
/// `I32ScalarReturnFacts::len` は replay 量の合計を表す。一方でこの構造体は、
/// どの種類の fact が stable value へ再投影できなかったかを cache 統計に分解して
/// 出すための内訳である。
pub(super) struct I32ScalarReturnFactCounts {
    pub(super) aliases: usize,
    pub(super) offsets: usize,
    pub(super) relations: usize,
    pub(super) constants: usize,
    pub(super) return_conditions: usize,
    pub(super) parameter_conditions: usize,
}

impl I32ScalarReturnFactCounts {
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    pub(super) fn total(&self) -> usize {
        self.aliases
            + self.offsets
            + self.relations
            + self.constants
            + self.return_conditions
            + self.parameter_conditions
    }
}

#[derive(Default)]
pub(super) struct I32ScalarReturnFactCollection {
    pub(super) facts: I32ScalarReturnFacts,
    pub(super) possible_return_projections: Vec<Vec<PlaceProjection>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScalarReturnAlias {
    pub(super) return_projection: Vec<PlaceProjection>,
    pub(super) parameter_index: usize,
    pub(super) parameter_projection: Vec<PlaceProjection>,
    pub(super) scalar_ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScalarReturnOffset {
    pub(super) return_projection: Vec<PlaceProjection>,
    pub(super) parameter_index: usize,
    pub(super) parameter_projection: Vec<PlaceProjection>,
    pub(super) scalar_ty: TypeId,
    pub(super) offset: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScalarReturnRelation {
    pub(super) left_return_projection: Vec<PlaceProjection>,
    pub(super) op: ResourceI32RelationOp,
    pub(super) right_return_projection: Vec<PlaceProjection>,
    pub(super) scalar_ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScalarReturnConstant {
    pub(super) return_projection: Vec<PlaceProjection>,
    pub(super) scalar_ty: TypeId,
    pub(super) value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScalarReturnCondition {
    pub(super) return_projection: Vec<PlaceProjection>,
    pub(super) scalar_ty: TypeId,
    pub(super) condition: I32ValueCondition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScalarParameterCondition {
    pub(super) parameter_index: usize,
    pub(super) parameter_projection: Vec<PlaceProjection>,
    pub(super) scalar_ty: TypeId,
    pub(super) condition: I32ValueCondition,
}

const I32_SCALAR_SUMMARY_CONDITIONS: [I32ValueCondition; 6] = [
    I32ValueCondition::EqZero,
    I32ValueCondition::NeZero,
    I32ValueCondition::Positive,
    I32ValueCondition::NonPositive,
    I32ValueCondition::Negative,
    I32ValueCondition::NonNegative,
];

pub(super) fn collect_i32_scalar_return_facts_for_value_suffix(
    params: &[ResourceLocal],
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    target_suffix: &[PlaceProjection],
) -> I32ScalarReturnFacts {
    let mut cache = I32LeafProjectionCache::default();
    collect_i32_scalar_return_facts_for_value_suffix_cached(
        params,
        types,
        raw_aliases,
        value,
        target_suffix,
        &mut cache,
    )
}

pub(super) fn collect_i32_scalar_return_facts_for_value_suffix_cached(
    params: &[ResourceLocal],
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    target_suffix: &[PlaceProjection],
    leaf_cache: &mut I32LeafProjectionCache,
) -> I32ScalarReturnFacts {
    collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter(
        params,
        types,
        raw_aliases,
        value,
        target_suffix,
        leaf_cache,
        |_| true,
    )
}

pub(super) fn collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter(
    params: &[ResourceLocal],
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    target_suffix: &[PlaceProjection],
    leaf_cache: &mut I32LeafProjectionCache,
    projection_is_possible: impl Fn(&[PlaceProjection]) -> bool,
) -> I32ScalarReturnFacts {
    collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter_inner(
        params,
        types,
        raw_aliases,
        value,
        target_suffix,
        leaf_cache,
        projection_is_possible,
        true,
    )
    .facts
}

pub(super) fn collect_i32_scalar_return_fact_collection_cached_with_projection_filter(
    params: &[ResourceLocal],
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    target_suffix: &[PlaceProjection],
    leaf_cache: &mut I32LeafProjectionCache,
    projection_is_possible: impl Fn(&[PlaceProjection]) -> bool,
) -> I32ScalarReturnFactCollection {
    collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter_inner(
        params,
        types,
        raw_aliases,
        value,
        target_suffix,
        leaf_cache,
        projection_is_possible,
        true,
    )
}

#[cfg(test)]
pub(super) fn collect_i32_scalar_return_facts_for_value_suffix_cached_without_parameter_conditions(
    params: &[ResourceLocal],
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    target_suffix: &[PlaceProjection],
    leaf_cache: &mut I32LeafProjectionCache,
    projection_is_possible: impl Fn(&[PlaceProjection]) -> bool,
) -> I32ScalarReturnFacts {
    collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter_inner(
        params,
        types,
        raw_aliases,
        value,
        target_suffix,
        leaf_cache,
        projection_is_possible,
        false,
    )
    .facts
}

pub(super) fn collect_i32_scalar_return_fact_collection_cached_without_parameter_conditions(
    params: &[ResourceLocal],
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    target_suffix: &[PlaceProjection],
    leaf_cache: &mut I32LeafProjectionCache,
    projection_is_possible: impl Fn(&[PlaceProjection]) -> bool,
) -> I32ScalarReturnFactCollection {
    collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter_inner(
        params,
        types,
        raw_aliases,
        value,
        target_suffix,
        leaf_cache,
        projection_is_possible,
        false,
    )
}

fn collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter_inner(
    params: &[ResourceLocal],
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    target_suffix: &[PlaceProjection],
    leaf_cache: &mut I32LeafProjectionCache,
    projection_is_possible: impl Fn(&[PlaceProjection]) -> bool,
    collect_parameter_conditions_for_path: bool,
) -> I32ScalarReturnFactCollection {
    let mut facts = I32ScalarReturnFacts::default();
    let mut condition_context = I32ConditionQueryContext::default();
    let mut possible_return_projections = Vec::new();
    let mut leaves = Vec::new();
    for leaf in leaf_cache.leaf_places_for_conditions(types, value) {
        let mut return_projection = target_suffix.to_vec();
        return_projection.extend_from_slice(&leaf.suffix);
        if !projection_is_possible(&return_projection) {
            continue;
        }
        push_unique_i32_scalar_projection(&mut possible_return_projections, return_projection);
        leaves.push(leaf);
    }
    for leaf in &leaves {
        let mut return_projection = target_suffix.to_vec();
        return_projection.extend_from_slice(&leaf.suffix);
        collect_i32_scalar_return_leaf_facts(
            params,
            raw_aliases,
            &leaf.place,
            &return_projection,
            &mut condition_context,
            &mut facts,
        );
    }
    collect_i32_scalar_return_leaf_relations(
        raw_aliases,
        &leaves,
        target_suffix,
        &mut condition_context,
        &mut facts,
    );
    if collect_parameter_conditions_for_path {
        collect_i32_scalar_parameter_conditions(
            params,
            types,
            raw_aliases,
            leaf_cache,
            &mut condition_context,
            &mut facts,
        );
    }
    I32ScalarReturnFactCollection {
        facts,
        possible_return_projections,
    }
}

pub(super) fn translate_i32_scalar_return_facts_for_call(
    params: &[ResourceLocal],
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    target_suffix: &[PlaceProjection],
    callee_facts: &I32ScalarReturnFacts,
) -> I32ScalarReturnFacts {
    let mut facts = I32ScalarReturnFacts::default();
    let mut condition_context = I32ConditionQueryContext::default();
    for alias in &callee_facts.aliases {
        let Some(arg) = args.get(alias.parameter_index) else {
            continue;
        };
        let source = projected_place_with_concrete_type(
            types,
            arg,
            &alias.parameter_projection,
            alias.scalar_ty,
        );
        let return_projection = compose_return_projection(target_suffix, &alias.return_projection);
        collect_i32_scalar_return_leaf_facts(
            params,
            raw_aliases,
            &source,
            &return_projection,
            &mut condition_context,
            &mut facts,
        );
    }
    for offset in &callee_facts.offsets {
        let Some(arg) = args.get(offset.parameter_index) else {
            continue;
        };
        let source = projected_place_with_concrete_type(
            types,
            arg,
            &offset.parameter_projection,
            offset.scalar_ty,
        );
        let return_projection = compose_return_projection(target_suffix, &offset.return_projection);
        collect_i32_scalar_return_offset_facts(
            params,
            raw_aliases,
            &source,
            &return_projection,
            offset.scalar_ty,
            offset.offset,
            &mut condition_context,
            &mut facts,
        );
    }
    for condition in &callee_facts.parameter_conditions {
        let Some(arg) = args.get(condition.parameter_index) else {
            continue;
        };
        let source = projected_place_with_concrete_type(
            types,
            arg,
            &condition.parameter_projection,
            condition.scalar_ty,
        );
        collect_i32_scalar_parameter_condition_fact(
            params,
            raw_aliases,
            &source,
            condition.scalar_ty,
            condition.condition,
            &mut condition_context,
            &mut facts,
        );
    }
    for constant in &callee_facts.constants {
        push_unique_i32_scalar_return_constant(
            &mut facts.constants,
            I32ScalarReturnConstant {
                return_projection: compose_return_projection(
                    target_suffix,
                    &constant.return_projection,
                ),
                scalar_ty: constant.scalar_ty,
                value: constant.value,
            },
        );
    }
    for condition in &callee_facts.return_conditions {
        push_unique_i32_scalar_return_condition(
            &mut facts.return_conditions,
            I32ScalarReturnCondition {
                return_projection: compose_return_projection(
                    target_suffix,
                    &condition.return_projection,
                ),
                scalar_ty: condition.scalar_ty,
                condition: condition.condition,
            },
        );
    }
    for relation in &callee_facts.relations {
        push_unique_i32_scalar_return_relation(
            &mut facts.relations,
            I32ScalarReturnRelation {
                left_return_projection: compose_return_projection(
                    target_suffix,
                    &relation.left_return_projection,
                ),
                op: relation.op,
                right_return_projection: compose_return_projection(
                    target_suffix,
                    &relation.right_return_projection,
                ),
                scalar_ty: relation.scalar_ty,
            },
        );
    }
    facts
}

pub(super) fn apply_i32_scalar_return_facts(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    args: &[Place],
    facts: &I32ScalarReturnFacts,
    types: &TypeCtx,
) -> bool {
    let mut applied = false;
    for (alias, arg) in facts
        .aliases
        .iter()
        .filter_map(|alias| args.get(alias.parameter_index).map(|arg| (alias, arg)))
    {
        let source = projected_place_with_concrete_type(
            types,
            arg,
            &alias.parameter_projection,
            alias.scalar_ty,
        );
        let target = projected_place_with_concrete_type(
            types,
            output,
            &alias.return_projection,
            alias.scalar_ty,
        );
        raw_aliases.copy_scalar_facts_if_tracked(&source, &target);
        applied = true;
    }
    for (offset, arg) in facts
        .offsets
        .iter()
        .filter_map(|offset| args.get(offset.parameter_index).map(|arg| (offset, arg)))
    {
        let source = projected_place_with_concrete_type(
            types,
            arg,
            &offset.parameter_projection,
            offset.scalar_ty,
        );
        let target = projected_place_with_concrete_type(
            types,
            output,
            &offset.return_projection,
            offset.scalar_ty,
        );
        raw_aliases.add_i32_offset(&source, &target, offset.offset);
        applied = true;
    }
    for relation in &facts.relations {
        let left = projected_place_with_concrete_type(
            types,
            output,
            &relation.left_return_projection,
            relation.scalar_ty,
        );
        let right = projected_place_with_concrete_type(
            types,
            output,
            &relation.right_return_projection,
            relation.scalar_ty,
        );
        raw_aliases.add_i32_relation(&left, relation.op, &right);
        applied = true;
    }
    for constant in &facts.constants {
        let target = projected_place_with_concrete_type(
            types,
            output,
            &constant.return_projection,
            constant.scalar_ty,
        );
        raw_aliases.set_i32_value(&target, constant.value);
        applied = true;
    }
    for condition in &facts.return_conditions {
        let target = projected_place_with_concrete_type(
            types,
            output,
            &condition.return_projection,
            condition.scalar_ty,
        );
        raw_aliases.add_i32_condition(&target, condition.condition);
        applied = true;
    }
    for (condition, arg) in facts.parameter_conditions.iter().filter_map(|condition| {
        args.get(condition.parameter_index)
            .map(|arg| (condition, arg))
    }) {
        let source = projected_place_with_concrete_type(
            types,
            arg,
            &condition.parameter_projection,
            condition.scalar_ty,
        );
        raw_aliases.add_i32_condition(&source, condition.condition);
        applied = true;
    }
    applied
}

fn collect_i32_scalar_return_leaf_relations(
    raw_aliases: &RawCellAddressAliases,
    leaves: &[OwnerLeafPlace],
    target_suffix: &[PlaceProjection],
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    // leaf 間の等価性照会は同じ raw alias graph に対する純粋な問い合わせである。
    // return value に多数の i32 leaf がある場合でも、alias/offset 到達性の memo を
    // relation 収集全体で共有し、同じ探索を leaf pair ごとに繰り返さない。
    for (left_index, left) in leaves.iter().enumerate() {
        for right in leaves.iter().skip(left_index + 1) {
            if left.place.ty != right.place.ty
                || !i32_scalar_leaf_places_are_known_equal(
                    raw_aliases,
                    &left.place,
                    &right.place,
                    condition_context,
                )
            {
                continue;
            }
            let left_return_projection = compose_return_projection(target_suffix, &left.suffix);
            let right_return_projection = compose_return_projection(target_suffix, &right.suffix);
            if left_return_projection == right_return_projection {
                continue;
            }
            push_unique_i32_scalar_return_relation(
                &mut facts.relations,
                I32ScalarReturnRelation {
                    left_return_projection,
                    op: ResourceI32RelationOp::Eq,
                    right_return_projection,
                    scalar_ty: left.place.ty,
                },
            );
        }
    }
}

fn push_unique_i32_scalar_projection(
    projections: &mut Vec<Vec<PlaceProjection>>,
    projection: Vec<PlaceProjection>,
) {
    if !projections
        .iter()
        .any(|existing| existing.as_slice() == projection.as_slice())
    {
        projections.push(projection);
    }
}

fn i32_scalar_leaf_places_are_known_equal(
    raw_aliases: &RawCellAddressAliases,
    left: &Place,
    right: &Place,
    condition_context: &mut I32ConditionQueryContext,
) -> bool {
    if left == right {
        return true;
    }
    let left_aliases = raw_aliases.scalar_aliases_for_value_with_context(left, condition_context);
    let right_aliases = raw_aliases.scalar_aliases_for_value_with_context(right, condition_context);
    left_aliases.iter().any(|left_alias| {
        right_aliases
            .iter()
            .any(|right_alias| right_alias == left_alias)
    }) || i32_scalar_leaf_relation_query_may_succeed(raw_aliases, &left_aliases, &right_aliases)
        && raw_aliases.i32_relation_truth_with_context(
            left,
            ResourceI32RelationOp::Eq,
            right,
            condition_context,
        ) == Some(true)
}

fn i32_scalar_leaf_relation_query_may_succeed(
    raw_aliases: &RawCellAddressAliases,
    left_aliases: &[Place],
    right_aliases: &[Place],
) -> bool {
    // `i32_relation_truth_with_context` は direct value、parameter condition、明示 relation、
    // offset graph、scale fact のどれからでも等価性を証明できる汎用問い合わせである。
    // return summary は戻り値 i32 leaf の全組にこの問い合わせを行うため、どちらの leaf も
    // scalar proof source に触れていない組だけは、到達不能な relation 探索へ入る前に外す。
    raw_aliases
        .i32_facts
        .has_condition_sources_for_aliases(left_aliases)
        || raw_aliases
            .i32_facts
            .has_condition_sources_for_aliases(right_aliases)
        || raw_aliases
            .i32_relations
            .has_relation_touching_aliases(left_aliases)
        || raw_aliases
            .i32_relations
            .has_relation_touching_aliases(right_aliases)
        || raw_aliases.i32_offsets.has_offset_for_aliases(left_aliases)
            && raw_aliases.i32_offsets.has_offset_for_aliases(right_aliases)
        || raw_aliases
            .i32_scales
            .has_scaled_source_for_aliases(left_aliases)
        || raw_aliases
            .i32_scales
            .has_scaled_source_for_aliases(right_aliases)
}

fn collect_i32_scalar_return_leaf_facts(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    leaf: &Place,
    return_projection: &[PlaceProjection],
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    if let Some(value) = raw_aliases.i32_value_with_context(leaf, condition_context) {
        push_unique_i32_scalar_return_constant(
            &mut facts.constants,
            I32ScalarReturnConstant {
                return_projection: return_projection.to_vec(),
                scalar_ty: leaf.ty,
                value,
            },
        );
    }
    collect_i32_scalar_return_conditions(
        raw_aliases,
        leaf,
        return_projection,
        condition_context,
        facts,
    );
    let leaf_aliases = raw_aliases.scalar_aliases_for_value_with_context(leaf, condition_context);
    for scalar_alias in &leaf_aliases {
        for (parameter_index, param) in params.iter().enumerate() {
            let Some(parameter_projection) = place_suffix_after_prefix(&scalar_alias, &param.place)
            else {
                continue;
            };
            push_unique_i32_scalar_return_alias(
                &mut facts.aliases,
                I32ScalarReturnAlias {
                    return_projection: return_projection.to_vec(),
                    parameter_index,
                    parameter_projection,
                    scalar_ty: leaf.ty,
                },
            );
        }
    }
    collect_i32_scalar_return_offset_facts_with_source_aliases(
        params,
        raw_aliases,
        leaf,
        &leaf_aliases,
        return_projection,
        leaf.ty,
        0,
        condition_context,
        facts,
    );
}

fn collect_i32_scalar_return_conditions(
    raw_aliases: &RawCellAddressAliases,
    source: &Place,
    return_projection: &[PlaceProjection],
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    if !raw_aliases.can_prove_i32_value_condition_for_value_with_context(source, condition_context)
    {
        return;
    }
    for condition in I32_SCALAR_SUMMARY_CONDITIONS {
        if raw_aliases.i32_condition_is_known_true_with_context(
            source,
            condition,
            condition_context,
        ) {
            push_unique_i32_scalar_return_condition(
                &mut facts.return_conditions,
                I32ScalarReturnCondition {
                    return_projection: return_projection.to_vec(),
                    scalar_ty: source.ty,
                    condition,
                },
            );
        }
    }
}

fn collect_i32_scalar_return_offset_facts(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    source: &Place,
    return_projection: &[PlaceProjection],
    scalar_ty: TypeId,
    additional_offset: i64,
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    let source_aliases =
        raw_aliases.scalar_aliases_for_value_with_context(source, condition_context);
    collect_i32_scalar_return_offset_facts_with_source_aliases(
        params,
        raw_aliases,
        source,
        &source_aliases,
        return_projection,
        scalar_ty,
        additional_offset,
        condition_context,
        facts,
    );
}

fn collect_i32_scalar_return_offset_facts_with_source_aliases(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    source: &Place,
    source_aliases: &[Place],
    return_projection: &[PlaceProjection],
    scalar_ty: TypeId,
    additional_offset: i64,
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    for scalar_alias in source_aliases {
        for (parameter_index, param) in params.iter().enumerate() {
            let Some(parameter_projection) = place_suffix_after_prefix(&scalar_alias, &param.place)
            else {
                continue;
            };
            push_unique_i32_scalar_return_offset(
                &mut facts.offsets,
                I32ScalarReturnOffset {
                    return_projection: return_projection.to_vec(),
                    parameter_index,
                    parameter_projection,
                    scalar_ty,
                    offset: additional_offset,
                },
            );
        }
    }
    for (base, source_offset) in raw_aliases.i32_offset_sources_for_target_aliases_with_context(
        source,
        source_aliases,
        condition_context,
    ) {
        let Some(offset) = source_offset.checked_add(additional_offset) else {
            continue;
        };
        let base_aliases =
            raw_aliases.scalar_aliases_for_value_with_context(&base, condition_context);
        for scalar_alias in base_aliases {
            for (parameter_index, param) in params.iter().enumerate() {
                let Some(parameter_projection) =
                    place_suffix_after_prefix(&scalar_alias, &param.place)
                else {
                    continue;
                };
                push_unique_i32_scalar_return_offset(
                    &mut facts.offsets,
                    I32ScalarReturnOffset {
                        return_projection: return_projection.to_vec(),
                        parameter_index,
                        parameter_projection,
                        scalar_ty,
                        offset,
                    },
                );
            }
        }
    }
}

fn collect_i32_scalar_parameter_conditions(
    params: &[ResourceLocal],
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    leaf_cache: &mut I32LeafProjectionCache,
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    if !raw_aliases.can_prove_i32_value_condition() {
        return;
    }
    for (parameter_index, param) in params.iter().enumerate() {
        for leaf in leaf_cache.leaf_places_for_conditions(types, &param.place) {
            if !raw_aliases.can_prove_i32_value_condition_for_value_with_context(
                &leaf.place,
                condition_context,
            ) {
                continue;
            }
            for condition in I32_SCALAR_SUMMARY_CONDITIONS {
                if raw_aliases.i32_condition_is_known_true_with_context(
                    &leaf.place,
                    condition,
                    condition_context,
                ) {
                    push_unique_i32_scalar_parameter_condition(
                        &mut facts.parameter_conditions,
                        I32ScalarParameterCondition {
                            parameter_index,
                            parameter_projection: leaf.suffix.clone(),
                            scalar_ty: leaf.place.ty,
                            condition,
                        },
                    );
                }
            }
        }
    }
}

fn collect_i32_scalar_parameter_condition_fact(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    source: &Place,
    scalar_ty: TypeId,
    condition: I32ValueCondition,
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    let source_aliases =
        raw_aliases.scalar_aliases_for_value_with_context(source, condition_context);
    for scalar_alias in source_aliases {
        for (parameter_index, param) in params.iter().enumerate() {
            let Some(parameter_projection) = place_suffix_after_prefix(&scalar_alias, &param.place)
            else {
                continue;
            };
            push_unique_i32_scalar_parameter_condition(
                &mut facts.parameter_conditions,
                I32ScalarParameterCondition {
                    parameter_index,
                    parameter_projection,
                    scalar_ty,
                    condition,
                },
            );
        }
    }
}

fn compose_return_projection(
    prefix: &[PlaceProjection],
    suffix: &[PlaceProjection],
) -> Vec<PlaceProjection> {
    let mut out = prefix.to_vec();
    out.extend_from_slice(suffix);
    out
}

fn push_unique_i32_scalar_return_alias(
    aliases: &mut Vec<I32ScalarReturnAlias>,
    alias: I32ScalarReturnAlias,
) {
    if aliases.iter().any(|existing| existing == &alias) {
        return;
    }
    aliases.push(alias);
}

fn push_unique_i32_scalar_return_offset(
    offsets: &mut Vec<I32ScalarReturnOffset>,
    offset: I32ScalarReturnOffset,
) {
    if offsets.iter().any(|existing| existing == &offset) {
        return;
    }
    offsets.push(offset);
}

fn push_unique_i32_scalar_return_relation(
    relations: &mut Vec<I32ScalarReturnRelation>,
    relation: I32ScalarReturnRelation,
) {
    if relations.iter().any(|existing| existing == &relation) {
        return;
    }
    relations.push(relation);
}

fn push_unique_i32_scalar_return_constant(
    constants: &mut Vec<I32ScalarReturnConstant>,
    constant: I32ScalarReturnConstant,
) {
    if constants.iter().any(|existing| existing == &constant) {
        return;
    }
    constants.push(constant);
}

fn push_unique_i32_scalar_return_condition(
    conditions: &mut Vec<I32ScalarReturnCondition>,
    condition: I32ScalarReturnCondition,
) {
    if conditions.iter().any(|existing| existing == &condition) {
        return;
    }
    conditions.push(condition);
}
#[cfg(test)]
#[path = "i32_scalar_return_facts_tests.rs"]
mod tests;

fn push_unique_i32_scalar_parameter_condition(
    conditions: &mut Vec<I32ScalarParameterCondition>,
    condition: I32ScalarParameterCondition,
) {
    if conditions.iter().any(|existing| existing == &condition) {
        return;
    }
    conditions.push(condition);
}
