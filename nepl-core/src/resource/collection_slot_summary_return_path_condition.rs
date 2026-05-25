extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_return_model::{
    CollectionSlotLifecyclePathPrecondition, CollectionSlotLifecyclePathPreconditionOperand,
};
use super::collection_slot_summary_target::{
    instantiate_summary_target_with_aliases, summary_place_for_params_with_aliases_and_types,
};
use super::condition_fact::simple_condition_value_constraint;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_relation_op::relation_negation;
use super::model::{Place, ResourceConditionFact, ResourceLocal};

pub(super) fn collect_return_path_preconditions(
    out: &mut Vec<CollectionSlotLifecyclePathPrecondition>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    state: &CollectionSlotSummaryBuildState,
    fact: &ResourceConditionFact,
    truthy_path: bool,
) {
    if let Some((place, condition)) = simple_condition_value_constraint(fact, truthy_path) {
        if let Some(operand) = path_precondition_operand_for_place(engine, params, state, place) {
            push_unique_precondition(
                out,
                CollectionSlotLifecyclePathPrecondition::I32Condition { operand, condition },
            );
        }
        return;
    }
    match (fact, truthy_path) {
        (ResourceConditionFact::I32Relation { left, op, right }, truthy_path) => {
            let Some(left) = path_precondition_operand_for_place(engine, params, state, left)
            else {
                return;
            };
            let Some(right) = path_precondition_operand_for_place(engine, params, state, right)
            else {
                return;
            };
            push_unique_precondition(
                out,
                CollectionSlotLifecyclePathPrecondition::I32Relation {
                    left,
                    op: if truthy_path {
                        *op
                    } else {
                        relation_negation(*op)
                    },
                    right,
                },
            );
        }
        (ResourceConditionFact::All(facts), true) | (ResourceConditionFact::Any(facts), false) => {
            for fact in facts {
                collect_return_path_preconditions(out, engine, params, state, fact, truthy_path);
            }
        }
        (ResourceConditionFact::All(_), false)
        | (ResourceConditionFact::Any(_), true)
        | (ResourceConditionFact::EqZero { .. }, _)
        | (ResourceConditionFact::NeZero { .. }, _)
        | (ResourceConditionFact::Positive { .. }, _)
        | (ResourceConditionFact::NonPositive { .. }, _)
        | (ResourceConditionFact::Negative { .. }, _)
        | (ResourceConditionFact::NonNegative { .. }, _) => {}
    }
}

pub(super) fn translate_return_path_preconditions_for_call(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    preconditions: &[CollectionSlotLifecyclePathPrecondition],
) -> Vec<CollectionSlotLifecyclePathPrecondition> {
    let mut out = Vec::new();
    for precondition in preconditions {
        match precondition {
            CollectionSlotLifecyclePathPrecondition::I32Condition { operand, condition } => {
                let Some(operand) =
                    translate_precondition_operand(engine, args, params, raw_aliases, operand)
                else {
                    continue;
                };
                push_unique_precondition(
                    &mut out,
                    CollectionSlotLifecyclePathPrecondition::I32Condition {
                        operand,
                        condition: *condition,
                    },
                );
            }
            CollectionSlotLifecyclePathPrecondition::I32Relation { left, op, right } => {
                let Some(left) =
                    translate_precondition_operand(engine, args, params, raw_aliases, left)
                else {
                    continue;
                };
                let Some(right) =
                    translate_precondition_operand(engine, args, params, raw_aliases, right)
                else {
                    continue;
                };
                push_unique_precondition(
                    &mut out,
                    CollectionSlotLifecyclePathPrecondition::I32Relation {
                        left,
                        op: *op,
                        right,
                    },
                );
            }
        }
    }
    out
}

pub(super) fn return_path_preconditions_match_callsite(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    preconditions: &[CollectionSlotLifecyclePathPrecondition],
) -> bool {
    preconditions
        .iter()
        .all(|precondition| precondition_matches_callsite(engine, args, raw_aliases, precondition))
}

fn path_precondition_operand_for_place(
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    state: &CollectionSlotSummaryBuildState,
    place: &Place,
) -> Option<CollectionSlotLifecyclePathPreconditionOperand> {
    let place = state.raw_aliases.canonicalize_scalar(place);
    if let Some(summary) = summary_place_for_params_with_aliases_and_types(
        params,
        Some(engine.types),
        &state.raw_aliases,
        &place,
    ) {
        return Some(CollectionSlotLifecyclePathPreconditionOperand::Place(
            summary,
        ));
    }
    state.raw_aliases.i32_value(&place).map(|value| {
        CollectionSlotLifecyclePathPreconditionOperand::KnownI32 {
            value,
            ty: place.ty,
        }
    })
}

fn translate_precondition_operand(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    operand: &CollectionSlotLifecyclePathPreconditionOperand,
) -> Option<CollectionSlotLifecyclePathPreconditionOperand> {
    match operand {
        CollectionSlotLifecyclePathPreconditionOperand::KnownI32 { value, ty } => {
            Some(CollectionSlotLifecyclePathPreconditionOperand::KnownI32 {
                value: *value,
                ty: *ty,
            })
        }
        CollectionSlotLifecyclePathPreconditionOperand::Place(place) => {
            let actual = instantiate_summary_target_with_aliases(engine, args, raw_aliases, place)?;
            let actual = raw_aliases.canonicalize_scalar(&actual);
            if let Some(summary) = summary_place_for_params_with_aliases_and_types(
                params,
                Some(engine.types),
                raw_aliases,
                &actual,
            ) {
                return Some(CollectionSlotLifecyclePathPreconditionOperand::Place(
                    summary,
                ));
            }
            raw_aliases.i32_value(&actual).map(|value| {
                CollectionSlotLifecyclePathPreconditionOperand::KnownI32 {
                    value,
                    ty: actual.ty,
                }
            })
        }
    }
}

fn precondition_matches_callsite(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    precondition: &CollectionSlotLifecyclePathPrecondition,
) -> bool {
    match precondition {
        CollectionSlotLifecyclePathPrecondition::I32Condition { operand, condition } => {
            let Some(operand) =
                instantiate_precondition_operand(engine, args, raw_aliases, operand)
            else {
                return true;
            };
            let truth = raw_aliases.i32_condition_truth(&operand, *condition);
            truth != Some(false)
        }
        CollectionSlotLifecyclePathPrecondition::I32Relation { left, op, right } => {
            let Some(left) = instantiate_precondition_operand(engine, args, raw_aliases, left)
            else {
                return true;
            };
            let Some(right) = instantiate_precondition_operand(engine, args, raw_aliases, right)
            else {
                return true;
            };
            let truth = raw_aliases.i32_relation_truth(&left, *op, &right);
            truth != Some(false)
        }
    }
}

fn instantiate_precondition_operand(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    operand: &CollectionSlotLifecyclePathPreconditionOperand,
) -> Option<Place> {
    match operand {
        CollectionSlotLifecyclePathPreconditionOperand::KnownI32 { value, ty } => {
            Some(Place::i32_constant(*value, *ty))
        }
        CollectionSlotLifecyclePathPreconditionOperand::Place(place) => {
            instantiate_summary_target_with_aliases(engine, args, raw_aliases, place)
        }
    }
}

fn push_unique_precondition(
    out: &mut Vec<CollectionSlotLifecyclePathPrecondition>,
    precondition: CollectionSlotLifecyclePathPrecondition,
) {
    if !out.iter().any(|existing| existing == &precondition) {
        out.push(precondition);
    }
}
