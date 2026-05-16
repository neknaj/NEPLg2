use alloc::vec::Vec;

use super::effect_summary_seed_alias::ParameterSeedAliases;
use super::model::{Place, ResourceOp};
use super::place_utils::push_unique_place;

pub(super) fn collect_parameter_descendant_places(
    ops: &[ResourceOp],
    parameter: &Place,
    aliases: &mut ParameterSeedAliases,
    places: &mut Vec<Place>,
) {
    for op in ops {
        match op {
            ResourceOp::Expr { output, .. } | ResourceOp::FunctionValue { output, .. } => {
                push_parameter_descendant_place(places, parameter, aliases, output);
            }
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                ..
            } => {
                push_parameter_descendant_place(places, parameter, aliases, output);
                for input in inputs {
                    push_parameter_descendant_place(places, parameter, aliases, input);
                }
                aliases.record_construct(parameter, output, kind, inputs);
            }
            ResourceOp::DeclareLocal {
                place, initializer, ..
            } => {
                push_parameter_descendant_place(places, parameter, aliases, place);
                if let Some(initializer) = initializer {
                    push_parameter_descendant_place(places, parameter, aliases, initializer);
                    aliases.record_copy(parameter, initializer, place);
                }
            }
            ResourceOp::Read { source, output, .. }
            | ResourceOp::Assign {
                target: output,
                value: source,
                ..
            }
            | ResourceOp::Move { source, output, .. }
            | ResourceOp::RawAddressAlias {
                source,
                target: output,
                ..
            }
            | ResourceOp::RawAddressView {
                source,
                target: output,
                ..
            } => {
                push_parameter_descendant_place(places, parameter, aliases, source);
                push_parameter_descendant_place(places, parameter, aliases, output);
                aliases.record_copy(parameter, source, output);
            }
            ResourceOp::Borrow { source, output, .. } => {
                push_parameter_descendant_place(places, parameter, aliases, source);
                push_parameter_descendant_place(places, parameter, aliases, output);
                aliases.record_borrow(parameter, source, output);
            }
            ResourceOp::Drop { place, .. } => {
                push_parameter_descendant_place(places, parameter, aliases, place);
            }
            ResourceOp::EndScope { locals, result, .. } => {
                for local in locals {
                    push_parameter_descendant_place(places, parameter, aliases, local);
                }
                if let Some(result) = result {
                    push_parameter_descendant_place(places, parameter, aliases, result);
                }
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                ..
            } => {
                push_parameter_descendant_place(places, parameter, aliases, output);
                push_parameter_descendant_place(places, parameter, aliases, callee);
                for arg in args {
                    push_parameter_descendant_place(places, parameter, aliases, arg);
                }
            }
            ResourceOp::Call { output, args, .. } | ResourceOp::RawMemory { output, args, .. } => {
                push_parameter_descendant_place(places, parameter, aliases, output);
                for arg in args {
                    push_parameter_descendant_place(places, parameter, aliases, arg);
                }
            }
            ResourceOp::StorageOrigin { target, .. } => {
                push_parameter_descendant_place(places, parameter, aliases, target);
            }
            ResourceOp::Branch {
                output,
                condition,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } => {
                push_parameter_descendant_place(places, parameter, aliases, output);
                push_parameter_descendant_place(places, parameter, aliases, condition);
                let mut then_aliases = aliases.clone();
                collect_parameter_descendant_places(then_ops, parameter, &mut then_aliases, places);
                push_parameter_descendant_place(places, parameter, &then_aliases, then_value);
                let mut else_aliases = aliases.clone();
                collect_parameter_descendant_places(else_ops, parameter, &mut else_aliases, places);
                push_parameter_descendant_place(places, parameter, &else_aliases, else_value);
            }
            ResourceOp::Loop {
                condition_ops,
                condition,
                body_ops,
                ..
            } => {
                let mut condition_aliases = aliases.clone();
                collect_parameter_descendant_places(
                    condition_ops,
                    parameter,
                    &mut condition_aliases,
                    places,
                );
                push_parameter_descendant_place(places, parameter, &condition_aliases, condition);
                collect_parameter_descendant_places(body_ops, parameter, aliases, places);
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                ..
            } => {
                push_parameter_descendant_place(places, parameter, aliases, output);
                push_parameter_descendant_place(places, parameter, aliases, scrutinee);
                for arm in arms {
                    let mut arm_aliases = aliases.clone();
                    if let Some(bind_local) = &arm.bind_local {
                        push_parameter_descendant_place(
                            places,
                            parameter,
                            &arm_aliases,
                            bind_local,
                        );
                    }
                    collect_parameter_descendant_places(
                        &arm.ops,
                        parameter,
                        &mut arm_aliases,
                        places,
                    );
                    push_parameter_descendant_place(places, parameter, &arm_aliases, &arm.value);
                }
            }
            ResourceOp::CallEffect { .. } => {}
        }
    }
}

fn push_parameter_descendant_place(
    places: &mut Vec<Place>,
    parameter: &Place,
    aliases: &ParameterSeedAliases,
    place: &Place,
) {
    if let Some(seed) = aliases.derived_place(parameter, place) {
        push_unique_place(places, &seed);
    }
}
