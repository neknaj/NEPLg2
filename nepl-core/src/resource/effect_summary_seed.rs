use alloc::vec::Vec;

use super::model::{Place, ResourceFunction, ResourceOp, ResourceTerminator};
use super::place_utils::{place_suffix_after_prefix, push_unique_place};

pub(super) fn parameter_summary_seed_places(
    function: &ResourceFunction,
    parameter: &Place,
) -> Vec<Place> {
    let mut places = Vec::new();
    push_unique_place(&mut places, parameter);
    for block in &function.blocks {
        collect_parameter_descendant_places(&block.ops, parameter, &mut places);
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            push_parameter_descendant_place(&mut places, parameter, value);
        }
    }
    places.sort();
    places
}

fn collect_parameter_descendant_places(
    ops: &[ResourceOp],
    parameter: &Place,
    places: &mut Vec<Place>,
) {
    for op in ops {
        match op {
            ResourceOp::Expr { output, .. }
            | ResourceOp::FunctionValue { output, .. }
            | ResourceOp::Construct { output, .. } => {
                push_parameter_descendant_place(places, parameter, output);
            }
            ResourceOp::DeclareLocal {
                place, initializer, ..
            } => {
                push_parameter_descendant_place(places, parameter, place);
                if let Some(initializer) = initializer {
                    push_parameter_descendant_place(places, parameter, initializer);
                }
            }
            ResourceOp::Read { source, output, .. }
            | ResourceOp::Assign {
                target: output,
                value: source,
                ..
            }
            | ResourceOp::Borrow { source, output, .. }
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
                push_parameter_descendant_place(places, parameter, source);
                push_parameter_descendant_place(places, parameter, output);
            }
            ResourceOp::Drop { place, .. } => {
                push_parameter_descendant_place(places, parameter, place);
            }
            ResourceOp::EndScope { locals, result, .. } => {
                for local in locals {
                    push_parameter_descendant_place(places, parameter, local);
                }
                if let Some(result) = result {
                    push_parameter_descendant_place(places, parameter, result);
                }
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                ..
            } => {
                push_parameter_descendant_place(places, parameter, output);
                push_parameter_descendant_place(places, parameter, callee);
                for arg in args {
                    push_parameter_descendant_place(places, parameter, arg);
                }
            }
            ResourceOp::Call { output, args, .. } | ResourceOp::RawMemory { output, args, .. } => {
                push_parameter_descendant_place(places, parameter, output);
                for arg in args {
                    push_parameter_descendant_place(places, parameter, arg);
                }
            }
            ResourceOp::StorageOrigin { target, .. } => {
                push_parameter_descendant_place(places, parameter, target);
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
                push_parameter_descendant_place(places, parameter, output);
                push_parameter_descendant_place(places, parameter, condition);
                collect_parameter_descendant_places(then_ops, parameter, places);
                push_parameter_descendant_place(places, parameter, then_value);
                collect_parameter_descendant_places(else_ops, parameter, places);
                push_parameter_descendant_place(places, parameter, else_value);
            }
            ResourceOp::Loop {
                condition_ops,
                condition,
                body_ops,
                ..
            } => {
                collect_parameter_descendant_places(condition_ops, parameter, places);
                push_parameter_descendant_place(places, parameter, condition);
                collect_parameter_descendant_places(body_ops, parameter, places);
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                ..
            } => {
                push_parameter_descendant_place(places, parameter, output);
                push_parameter_descendant_place(places, parameter, scrutinee);
                for arm in arms {
                    if let Some(bind_local) = &arm.bind_local {
                        push_parameter_descendant_place(places, parameter, bind_local);
                    }
                    collect_parameter_descendant_places(&arm.ops, parameter, places);
                    push_parameter_descendant_place(places, parameter, &arm.value);
                }
            }
            ResourceOp::CallEffect { .. } => {}
        }
    }
}

fn push_parameter_descendant_place(places: &mut Vec<Place>, parameter: &Place, place: &Place) {
    if place_suffix_after_prefix(place, parameter).is_some() {
        push_unique_place(places, place);
    }
}
