extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_i32_condition_context::I32ConditionQueryContext;
use super::model::{I32ValueCondition, Place, PlaceProjection, ResourceLocal};
use super::owner_summary_i32_condition_leaf::I32LeafProjectionCache;
use super::place_utils::{place_suffix_after_prefix, projected_place_with_concrete_type};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct I32ScalarReturnFacts {
    pub(super) aliases: Vec<I32ScalarReturnAlias>,
    pub(super) offsets: Vec<I32ScalarReturnOffset>,
    pub(super) constants: Vec<I32ScalarReturnConstant>,
    pub(super) return_conditions: Vec<I32ScalarReturnCondition>,
    pub(super) parameter_conditions: Vec<I32ScalarParameterCondition>,
}

impl I32ScalarReturnFacts {
    pub(super) fn is_empty(&self) -> bool {
        self.aliases.is_empty()
            && self.offsets.is_empty()
            && self.constants.is_empty()
            && self.return_conditions.is_empty()
            && self.parameter_conditions.is_empty()
    }

    pub(super) fn extend(&mut self, other: I32ScalarReturnFacts) {
        for alias in other.aliases {
            push_unique_i32_scalar_return_alias(&mut self.aliases, alias);
        }
        for offset in other.offsets {
            push_unique_i32_scalar_return_offset(&mut self.offsets, offset);
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
    let mut facts = I32ScalarReturnFacts::default();
    let mut condition_context = I32ConditionQueryContext::default();
    for leaf in leaf_cache.leaf_places_for_conditions(types, value) {
        let mut return_projection = target_suffix.to_vec();
        return_projection.extend(leaf.suffix);
        collect_i32_scalar_return_leaf_facts(
            params,
            raw_aliases,
            &leaf.place,
            &return_projection,
            &mut condition_context,
            &mut facts,
        );
    }
    collect_i32_scalar_parameter_conditions(
        params,
        types,
        raw_aliases,
        leaf_cache,
        &mut condition_context,
        &mut facts,
    );
    facts
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

fn collect_i32_scalar_return_leaf_facts(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    leaf: &Place,
    return_projection: &[PlaceProjection],
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    if let Some(value) = raw_aliases.i32_value(leaf) {
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
    for scalar_alias in raw_aliases.scalar_aliases_for_value(leaf) {
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
    collect_i32_scalar_parameter_conditions_for_source(
        params,
        raw_aliases,
        leaf,
        leaf.ty,
        condition_context,
        facts,
    );
    collect_i32_scalar_return_offset_facts(
        params,
        raw_aliases,
        leaf,
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
    collect_i32_scalar_parameter_conditions_for_source(
        params,
        raw_aliases,
        source,
        scalar_ty,
        condition_context,
        facts,
    );
    for scalar_alias in raw_aliases.scalar_aliases_for_value(source) {
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
    for (base, source_offset) in raw_aliases.i32_offset_sources(source) {
        let Some(offset) = source_offset.checked_add(additional_offset) else {
            continue;
        };
        collect_i32_scalar_parameter_conditions_for_source(
            params,
            raw_aliases,
            &base,
            scalar_ty,
            condition_context,
            facts,
        );
        for scalar_alias in raw_aliases.scalar_aliases_for_value(&base) {
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

fn collect_i32_scalar_parameter_conditions_for_source(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    source: &Place,
    scalar_ty: TypeId,
    condition_context: &mut I32ConditionQueryContext,
    facts: &mut I32ScalarReturnFacts,
) {
    for condition in I32_SCALAR_SUMMARY_CONDITIONS {
        if raw_aliases.i32_condition_is_known_true_with_context(
            source,
            condition,
            condition_context,
        ) {
            collect_i32_scalar_parameter_condition_fact(
                params,
                raw_aliases,
                source,
                scalar_ty,
                condition,
                facts,
            );
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
    for (parameter_index, param) in params.iter().enumerate() {
        for leaf in leaf_cache.leaf_places_for_conditions(types, &param.place) {
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
    facts: &mut I32ScalarReturnFacts,
) {
    for scalar_alias in raw_aliases.scalar_aliases_for_value(source) {
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

fn push_unique_i32_scalar_parameter_condition(
    conditions: &mut Vec<I32ScalarParameterCondition>,
    condition: I32ScalarParameterCondition,
) {
    if conditions.iter().any(|existing| existing == &condition) {
        return;
    }
    conditions.push(condition);
}
