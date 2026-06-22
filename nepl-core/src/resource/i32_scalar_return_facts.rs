extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_i32_condition_context::I32ConditionQueryContext;
use super::model::{
    I32ValueCondition, Place, PlaceProjection, PlaceRoot, ResourceI32RelationOp, ResourceLocal,
};
use super::owner_summary_i32_condition_leaf::I32LeafProjectionCache;
use super::owner_summary_leaf::OwnerLeafPlace;
use super::place_utils::{
    place_suffix_after_prefix, projected_place_with_concrete_type, push_unique_place,
};

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct I32ScalarParameterConditionCandidate {
    parameter_index: usize,
    parameter_projection: Vec<PlaceProjection>,
    place: Place,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct I32ScalarReturnLeafFactMode {
    collect_conditions: bool,
    collect_parameter_aliases: bool,
    collect_offsets: bool,
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
    collect_i32_scalar_return_facts_for_value_suffix_cached_with_projection_filter_inner(
        params,
        types,
        raw_aliases,
        value,
        target_suffix,
        leaf_cache,
        |_| true,
        true,
        false,
        None,
    )
    .facts
}

#[cfg(test)]
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
        true,
        None,
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
    timing_function: Option<&str>,
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
        true,
        timing_function,
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
        true,
        None,
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
    timing_function: Option<&str>,
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
        true,
        timing_function,
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
    prune_return_leaves_by_projection_filter: bool,
    timing_function: Option<&str>,
) -> I32ScalarReturnFactCollection {
    #[cfg(any(target_os = "none", target_arch = "wasm32"))]
    let _ = timing_function;
    let mut facts = I32ScalarReturnFacts::default();
    let mut condition_context = I32ConditionQueryContext::default();
    let mut possible_return_projections = Vec::new();
    let mut leaves = Vec::new();
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let leaf_generation_start =
        i32_scalar_return_detail_timing_start(timing_function, "leaf_generation", 0);
    let return_leaves = if prune_return_leaves_by_projection_filter {
        leaf_cache.leaf_places_for_conditions_with_projection_filter(
            types,
            value,
            target_suffix,
            &projection_is_possible,
        )
    } else {
        leaf_cache.leaf_places_for_conditions(types, value)
    };
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    i32_scalar_return_detail_timing_finish(
        timing_function,
        "leaf_generation",
        return_leaves.len(),
        leaf_generation_start,
    );
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let leaf_prepare_start =
        i32_scalar_return_detail_timing_start(timing_function, "leaf_prepare", return_leaves.len());
    for leaf in return_leaves {
        let mut return_projection = target_suffix.to_vec();
        return_projection.extend_from_slice(&leaf.suffix);
        if !prune_return_leaves_by_projection_filter && !projection_is_possible(&return_projection)
        {
            continue;
        }
        push_unique_i32_scalar_projection(&mut possible_return_projections, return_projection);
        leaves.push(leaf);
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    i32_scalar_return_detail_timing_finish(
        timing_function,
        "leaf_prepare",
        leaves.len(),
        leaf_prepare_start,
    );
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let fact_leaf_filter_start =
        i32_scalar_return_detail_timing_start(timing_function, "fact_leaf_filter", leaves.len());
    let condition_candidate_projections =
        i32_scalar_return_condition_candidate_projections(raw_aliases, value);
    let parameter_candidate_projections =
        i32_scalar_return_parameter_linked_candidate_projections(params, raw_aliases, value);
    let offset_candidate_projections =
        i32_scalar_return_offset_candidate_projections(raw_aliases, value);
    let fact_leaf_modes = i32_scalar_return_fact_leaf_modes(
        &leaves,
        &condition_candidate_projections,
        &parameter_candidate_projections,
        &offset_candidate_projections,
    );
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    i32_scalar_return_detail_timing_finish(
        timing_function,
        "fact_leaf_filter",
        fact_leaf_modes.len(),
        fact_leaf_filter_start,
    );
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let leaf_facts_start =
        i32_scalar_return_detail_timing_start(timing_function, "leaf_facts", fact_leaf_modes.len());
    for (leaf_index, mode) in &fact_leaf_modes {
        let leaf = &leaves[*leaf_index];
        let mut return_projection = target_suffix.to_vec();
        return_projection.extend_from_slice(&leaf.suffix);
        collect_i32_scalar_return_leaf_facts_with_mode(
            params,
            raw_aliases,
            &leaf.place,
            &return_projection,
            *mode,
            &mut condition_context,
            &mut facts,
        );
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    i32_scalar_return_detail_timing_finish(
        timing_function,
        "leaf_facts",
        facts.len(),
        leaf_facts_start,
    );
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let relation_leaf_filter_start = i32_scalar_return_detail_timing_start(
        timing_function,
        "relation_leaf_filter",
        leaves.len(),
    );
    let relation_candidate_projections =
        i32_scalar_return_relation_candidate_projections(params, raw_aliases, value);
    let relation_leaf_indices =
        i32_scalar_return_fact_leaf_indices(&leaves, &relation_candidate_projections);
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    i32_scalar_return_detail_timing_finish(
        timing_function,
        "relation_leaf_filter",
        relation_leaf_indices.len(),
        relation_leaf_filter_start,
    );
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let leaf_relations_start = i32_scalar_return_detail_timing_start(
        timing_function,
        "leaf_relations",
        relation_leaf_indices.len(),
    );
    collect_i32_scalar_return_leaf_relations(
        raw_aliases,
        &leaves,
        &relation_leaf_indices,
        target_suffix,
        &mut condition_context,
        &mut facts,
    );
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    i32_scalar_return_detail_timing_finish(
        timing_function,
        "leaf_relations",
        facts.relations.len(),
        leaf_relations_start,
    );
    if collect_parameter_conditions_for_path {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        let parameter_conditions_start = i32_scalar_return_detail_timing_start(
            timing_function,
            "parameter_conditions",
            params.len(),
        );
        collect_i32_scalar_parameter_conditions(
            params,
            raw_aliases,
            &mut condition_context,
            &mut facts,
        );
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        i32_scalar_return_detail_timing_finish(
            timing_function,
            "parameter_conditions",
            facts.parameter_conditions.len(),
            parameter_conditions_start,
        );
    }
    I32ScalarReturnFactCollection {
        facts,
        possible_return_projections,
    }
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
fn i32_scalar_return_detail_timing_start(
    function: Option<&str>,
    stage: &str,
    count: usize,
) -> Option<std::time::Instant> {
    let function = function?;
    if !super::timing::resource_i32_return_timing_enabled()
        || !super::timing::resource_timing_function_matches(function)
    {
        return None;
    }
    std::eprintln!(
        "[resource-i32-return-detail] start function={} stage={} count={}",
        function,
        stage,
        count
    );
    Some(std::time::Instant::now())
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
fn i32_scalar_return_detail_timing_finish(
    function: Option<&str>,
    stage: &str,
    count: usize,
    start: Option<std::time::Instant>,
) {
    let Some(start) = start else {
        return;
    };
    let Some(function) = function else {
        return;
    };
    std::eprintln!(
        "[resource-i32-return-detail] end function={} stage={} count={} elapsed_ms={}",
        function,
        stage,
        count,
        start.elapsed().as_millis()
    );
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
    leaf_indices: &[usize],
    target_suffix: &[PlaceProjection],
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    // leaf 間の等価性照会は同じ raw alias graph に対する純粋な問い合わせである。
    // return value に多数の i32 leaf がある場合でも、alias/offset 到達性の memo を
    // relation 収集全体で共有し、同じ探索を leaf pair ごとに繰り返さない。
    for (left_position, left_index) in leaf_indices.iter().enumerate() {
        let left = &leaves[*left_index];
        for right_index in leaf_indices.iter().skip(left_position + 1) {
            let right = &leaves[*right_index];
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

fn i32_scalar_return_condition_candidate_projections(
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
) -> Vec<Vec<PlaceProjection>> {
    let mut out = Vec::new();
    let non_relation_condition_places =
        i32_scalar_return_non_relation_condition_candidate_places(raw_aliases);
    push_i32_scalar_return_candidate_projections(
        &mut out,
        value,
        non_relation_condition_places.iter(),
    );
    let relation_condition_places = i32_scalar_return_condition_connected_relation_candidate_places(
        raw_aliases,
        non_relation_condition_places,
    );
    push_i32_scalar_return_candidate_projections(&mut out, value, relation_condition_places.iter());
    out
}

fn i32_scalar_return_offset_candidate_projections(
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
) -> Vec<Vec<PlaceProjection>> {
    let mut out = Vec::new();
    let offset_candidate_places = raw_aliases.i32_offsets.condition_candidate_places();
    push_i32_scalar_return_candidate_projections(&mut out, value, offset_candidate_places.iter());
    out
}

fn i32_scalar_return_relation_candidate_projections(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
) -> Vec<Vec<PlaceProjection>> {
    let mut out = Vec::new();
    let mut relation_candidate_places = Vec::new();
    for place in raw_aliases.i32_relations.condition_candidate_places() {
        push_unique_place(&mut relation_candidate_places, &place);
    }
    for place in raw_aliases.i32_offsets.condition_candidate_places() {
        push_unique_place(&mut relation_candidate_places, &place);
    }
    push_i32_scalar_return_candidate_projections(&mut out, value, relation_candidate_places.iter());
    for projection in
        i32_scalar_return_parameter_linked_candidate_projections(params, raw_aliases, value)
    {
        push_unique_i32_scalar_projection(&mut out, projection);
    }
    out
}

fn i32_scalar_return_non_relation_condition_candidate_places(
    raw_aliases: &RawCellAddressAliases,
) -> Vec<Place> {
    let mut out = Vec::new();
    for place in raw_aliases.i32_facts.condition_candidate_places() {
        push_unique_place(&mut out, &place);
    }
    for place in raw_aliases.i32_offsets.condition_candidate_places() {
        push_unique_place(&mut out, &place);
    }
    for place in raw_aliases.i32_scales.condition_candidate_places() {
        push_unique_place(&mut out, &place);
    }
    out
}

fn i32_scalar_return_condition_connected_relation_candidate_places(
    raw_aliases: &RawCellAddressAliases,
    mut condition_sources: Vec<Place>,
) -> Vec<Place> {
    for relation in &raw_aliases.i32_relations.relations {
        if matches!(relation.left.root, PlaceRoot::I32Constant(_)) {
            push_unique_place(&mut condition_sources, &relation.left);
        }
        if matches!(relation.right.root, PlaceRoot::I32Constant(_)) {
            push_unique_place(&mut condition_sources, &relation.right);
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for relation in &raw_aliases.i32_relations.relations {
            let left_connected = condition_sources
                .iter()
                .any(|source| source == &relation.left);
            let right_connected = condition_sources
                .iter()
                .any(|source| source == &relation.right);
            if left_connected && !right_connected {
                push_unique_place(&mut condition_sources, &relation.right);
                changed = true;
            }
            if right_connected && !left_connected {
                push_unique_place(&mut condition_sources, &relation.left);
                changed = true;
            }
        }
    }
    condition_sources
}

fn push_i32_scalar_return_candidate_projections<'a>(
    out: &mut Vec<Vec<PlaceProjection>>,
    value: &Place,
    places: impl IntoIterator<Item = &'a Place>,
) {
    for place in places {
        let Some(projection) = place_suffix_after_prefix(&place, value) else {
            continue;
        };
        push_unique_i32_scalar_projection(out, projection);
    }
}

fn i32_scalar_return_parameter_linked_candidate_projections(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
) -> Vec<Vec<PlaceProjection>> {
    let mut out = Vec::new();
    for param in params {
        for place in raw_aliases
            .scalar_alias_candidate_places_under_prefix_linked_to_prefix(value, &param.place)
        {
            let Some(projection) = place_suffix_after_prefix(&place, value) else {
                continue;
            };
            push_unique_i32_scalar_projection(&mut out, projection);
        }
    }
    out
}

fn i32_scalar_return_fact_leaf_modes(
    leaves: &[OwnerLeafPlace],
    condition_candidate_projections: &[Vec<PlaceProjection>],
    parameter_candidate_projections: &[Vec<PlaceProjection>],
    offset_candidate_projections: &[Vec<PlaceProjection>],
) -> Vec<(usize, I32ScalarReturnLeafFactMode)> {
    let mut out = Vec::new();
    for (index, leaf) in leaves.iter().enumerate() {
        let collect_conditions =
            i32_scalar_projection_matches_any_prefix(condition_candidate_projections, &leaf.suffix);
        let collect_parameter_aliases =
            i32_scalar_projection_matches_any_prefix(parameter_candidate_projections, &leaf.suffix);
        let collect_offsets = collect_parameter_aliases
            || i32_scalar_projection_matches_any_prefix(offset_candidate_projections, &leaf.suffix);
        if collect_conditions || collect_parameter_aliases || collect_offsets {
            out.push((
                index,
                I32ScalarReturnLeafFactMode {
                    collect_conditions,
                    collect_parameter_aliases,
                    collect_offsets,
                },
            ));
        }
    }
    out
}

fn i32_scalar_return_fact_leaf_indices(
    leaves: &[OwnerLeafPlace],
    candidate_projections: &[Vec<PlaceProjection>],
) -> Vec<usize> {
    if candidate_projections.is_empty() {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (index, leaf) in leaves.iter().enumerate() {
        if i32_scalar_projection_matches_any_prefix(candidate_projections, &leaf.suffix) {
            out.push(index);
        }
    }
    out
}

fn i32_scalar_projection_matches_any_prefix(
    candidate_projections: &[Vec<PlaceProjection>],
    projection: &[PlaceProjection],
) -> bool {
    candidate_projections
        .iter()
        .any(|candidate| i32_scalar_projection_prefix_matches(candidate.as_slice(), projection))
}

fn i32_scalar_projection_prefix_matches(
    prefix: &[PlaceProjection],
    projection: &[PlaceProjection],
) -> bool {
    prefix.len() <= projection.len() && projection.starts_with(prefix)
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
            && raw_aliases
                .i32_offsets
                .has_offset_for_aliases(right_aliases)
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
    collect_i32_scalar_return_leaf_facts_with_mode(
        params,
        raw_aliases,
        leaf,
        return_projection,
        I32ScalarReturnLeafFactMode {
            collect_conditions: true,
            collect_parameter_aliases: true,
            collect_offsets: true,
        },
        condition_context,
        facts,
    );
}

fn collect_i32_scalar_return_leaf_facts_with_mode(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    leaf: &Place,
    return_projection: &[PlaceProjection],
    mode: I32ScalarReturnLeafFactMode,
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    let mut leaf_aliases = None;
    let has_parameter_alias = mode.collect_parameter_aliases
        && i32_scalar_leaf_has_parameter_alias(
            i32_scalar_return_leaf_aliases(&mut leaf_aliases, raw_aliases, leaf, condition_context),
            params,
        );
    let condition_query_may_succeed = mode.collect_conditions
        && (matches!(leaf.root, PlaceRoot::I32Constant(_))
            || i32_scalar_leaf_condition_query_may_succeed(
                raw_aliases,
                i32_scalar_return_leaf_aliases(
                    &mut leaf_aliases,
                    raw_aliases,
                    leaf,
                    condition_context,
                ),
            ));
    let offset_query_may_succeed = mode.collect_offsets
        && raw_aliases
            .i32_offsets
            .has_offset_for_aliases(i32_scalar_return_leaf_aliases(
                &mut leaf_aliases,
                raw_aliases,
                leaf,
                condition_context,
            ));

    if !has_parameter_alias && !condition_query_may_succeed && !offset_query_may_succeed {
        return;
    }

    if condition_query_may_succeed {
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
    }
    if has_parameter_alias {
        collect_i32_scalar_return_alias_facts(
            params,
            i32_scalar_return_leaf_aliases(&mut leaf_aliases, raw_aliases, leaf, condition_context),
            return_projection,
            leaf.ty,
            facts,
        );
    }
    if mode.collect_offsets && offset_query_may_succeed {
        collect_i32_scalar_return_offset_facts_with_source_aliases(
            params,
            raw_aliases,
            leaf,
            i32_scalar_return_leaf_aliases(&mut leaf_aliases, raw_aliases, leaf, condition_context),
            return_projection,
            leaf.ty,
            0,
            condition_context,
            facts,
        );
    }
}

fn i32_scalar_return_leaf_aliases<'a>(
    cache: &'a mut Option<Vec<Place>>,
    raw_aliases: &RawCellAddressAliases,
    leaf: &Place,
    condition_context: &mut I32ConditionQueryContext,
) -> &'a [Place] {
    cache
        .get_or_insert_with(|| {
            raw_aliases.scalar_aliases_for_value_with_context(leaf, condition_context)
        })
        .as_slice()
}

fn i32_scalar_leaf_has_parameter_alias(leaf_aliases: &[Place], params: &[ResourceLocal]) -> bool {
    leaf_aliases.iter().any(|scalar_alias| {
        params
            .iter()
            .any(|param| place_suffix_after_prefix(scalar_alias, &param.place).is_some())
    })
}

fn i32_scalar_leaf_condition_query_may_succeed(
    raw_aliases: &RawCellAddressAliases,
    leaf_aliases: &[Place],
) -> bool {
    raw_aliases
        .i32_facts
        .has_condition_sources_for_aliases(leaf_aliases)
        || raw_aliases
            .i32_relations
            .has_relation_touching_aliases(leaf_aliases)
        || raw_aliases.i32_offsets.has_offset_for_aliases(leaf_aliases)
        || raw_aliases
            .i32_scales
            .has_scaled_source_for_aliases(leaf_aliases)
}

fn collect_i32_scalar_return_alias_facts(
    params: &[ResourceLocal],
    leaf_aliases: &[Place],
    return_projection: &[PlaceProjection],
    scalar_ty: TypeId,
    facts: &mut I32ScalarReturnFacts,
) {
    for scalar_alias in leaf_aliases {
        for (parameter_index, param) in params.iter().enumerate() {
            let Some(parameter_projection) = place_suffix_after_prefix(scalar_alias, &param.place)
            else {
                continue;
            };
            push_unique_i32_scalar_return_alias(
                &mut facts.aliases,
                I32ScalarReturnAlias {
                    return_projection: return_projection.to_vec(),
                    parameter_index,
                    parameter_projection,
                    scalar_ty,
                },
            );
        }
    }
}

fn collect_i32_scalar_return_conditions(
    raw_aliases: &RawCellAddressAliases,
    source: &Place,
    return_projection: &[PlaceProjection],
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    if !matches!(source.root, PlaceRoot::I32Constant(_))
        && !raw_aliases.can_prove_i32_value_condition()
    {
        return;
    }
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
    raw_aliases: &RawCellAddressAliases,
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    if !raw_aliases.can_prove_i32_value_condition() {
        return;
    }
    let candidates =
        i32_scalar_parameter_condition_candidates(params, raw_aliases, condition_context);
    for candidate in candidates {
        if !raw_aliases.can_prove_i32_value_condition_for_value_with_context(
            &candidate.place,
            condition_context,
        ) {
            continue;
        }
        for condition in I32_SCALAR_SUMMARY_CONDITIONS {
            if raw_aliases.i32_condition_is_known_true_with_context(
                &candidate.place,
                condition,
                condition_context,
            ) {
                push_unique_i32_scalar_parameter_condition(
                    &mut facts.parameter_conditions,
                    I32ScalarParameterCondition {
                        parameter_index: candidate.parameter_index,
                        parameter_projection: candidate.parameter_projection.clone(),
                        scalar_ty: candidate.place.ty,
                        condition,
                    },
                );
            }
        }
    }
}

fn i32_scalar_parameter_condition_candidates(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    condition_context: &mut I32ConditionQueryContext,
) -> Vec<I32ScalarParameterConditionCandidate> {
    let mut out = Vec::new();
    for proof_source in raw_aliases.i32_value_condition_candidate_places() {
        for alias in
            raw_aliases.scalar_aliases_for_value_with_context(&proof_source, condition_context)
        {
            for (parameter_index, param) in params.iter().enumerate() {
                let Some(parameter_projection) = place_suffix_after_prefix(&alias, &param.place)
                else {
                    continue;
                };
                push_unique_i32_scalar_parameter_condition_candidate(
                    &mut out,
                    I32ScalarParameterConditionCandidate {
                        parameter_index,
                        parameter_projection,
                        place: alias.clone(),
                    },
                );
            }
        }
    }
    out
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

fn push_unique_i32_scalar_parameter_condition_candidate(
    candidates: &mut Vec<I32ScalarParameterConditionCandidate>,
    candidate: I32ScalarParameterConditionCandidate,
) {
    if candidates.iter().any(|existing| existing == &candidate) {
        return;
    }
    candidates.push(candidate);
}
