extern crate alloc;

use alloc::{vec, vec::Vec};

use super::collection_slot_summary_build_range_step_expr::{
    effect_is_proof_pure, loop_step_expr_effect, LoopStepExprEffect,
};
use super::model::{EffectOp, Place, RawMemoryOp, ResourceCallTarget, ResourceOp};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn loop_body_increment_step(ops: &[ResourceOp], index: &Place) -> Option<usize> {
    let index = index.clone();
    let mut index_aliases = vec![index.clone()];
    let mut index_reference_aliases = Vec::new();
    let mut one_values = Vec::new();
    let mut increment_values = Vec::new();
    let mut step = None;
    for (op_index, op) in ops.iter().enumerate() {
        match op {
            ResourceOp::Expr { kind, output, .. } => {
                match loop_step_expr_effect(kind, output, &index) {
                    LoopStepExprEffect::Marker => {}
                    LoopStepExprEffect::Reject => return None,
                    LoopStepExprEffect::LiteralOne(output) => {
                        clear_place_facts(
                            output,
                            &mut index_aliases,
                            &mut index_reference_aliases,
                            &mut one_values,
                            &mut increment_values,
                        );
                        push_place(&mut one_values, output);
                    }
                    LoopStepExprEffect::Clear(output) => clear_place_facts(
                        output,
                        &mut index_aliases,
                        &mut index_reference_aliases,
                        &mut one_values,
                        &mut increment_values,
                    ),
                }
            }
            ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
                if place_in(&index_aliases, source) {
                    clear_place_facts(
                        output,
                        &mut index_aliases,
                        &mut index_reference_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                    push_place(&mut index_aliases, output);
                } else if place_in(&index_reference_aliases, source) {
                    clear_place_facts(
                        output,
                        &mut index_aliases,
                        &mut index_reference_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                    push_place(&mut index_reference_aliases, output);
                } else {
                    clear_place_facts(
                        output,
                        &mut index_aliases,
                        &mut index_reference_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                }
            }
            ResourceOp::Call {
                output,
                target,
                args,
                effect,
                ..
            } => {
                if !effect_is_proof_pure(effect) {
                    if matches!(
                        effect,
                        EffectOp::UnsafeMemory {
                            operation: RawMemoryOp::Load
                        }
                    ) {
                        clear_place_facts(
                            output,
                            &mut index_aliases,
                            &mut index_reference_aliases,
                            &mut one_values,
                            &mut increment_values,
                        );
                        continue;
                    }
                    if call_may_mutate_index(output, args, &index, &index_reference_aliases) {
                        return None;
                    }
                    // `Drop::drop &loaded` のような effectful witness call は loop index を
                    // 変更しない限り step 証明の対象外です。index への参照が call に渡る場合は
                    // 上の `call_may_mutate_index` で拒否し、ここでは出力側の一時 fact だけを消します。
                    clear_place_facts(
                        output,
                        &mut index_aliases,
                        &mut index_reference_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                    continue;
                }
                if args
                    .iter()
                    .any(|arg| place_in(&index_reference_aliases, arg))
                {
                    return None;
                }
                if i32_adds_one_to_index(target, args, &index_aliases, &one_values) {
                    clear_place_facts(
                        output,
                        &mut index_aliases,
                        &mut index_reference_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                    push_place(&mut increment_values, output);
                } else {
                    clear_place_facts(
                        output,
                        &mut index_aliases,
                        &mut index_reference_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                }
            }
            ResourceOp::Assign { target, value, .. } if *target == index => {
                if step.is_some() || !place_in(&increment_values, value) {
                    return None;
                }
                step = Some(op_index);
                index_aliases.clear();
                index_reference_aliases.clear();
                push_place(&mut index_aliases, target);
            }
            ResourceOp::Assign { target, .. } if place_touches(target, &index) => {
                return None;
            }
            ResourceOp::Assign { target, value, .. } => {
                if place_in(&index_reference_aliases, value) {
                    clear_place_facts(
                        target,
                        &mut index_aliases,
                        &mut index_reference_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                    push_place(&mut index_reference_aliases, target);
                } else {
                    clear_place_facts(
                        target,
                        &mut index_aliases,
                        &mut index_reference_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                }
            }
            ResourceOp::Branch { .. }
            | ResourceOp::Loop { .. }
            | ResourceOp::Match { .. }
            | ResourceOp::IndirectCall { .. } => {
                return None;
            }
            ResourceOp::DeclareLocal {
                place, initializer, ..
            } => {
                if initializer
                    .as_ref()
                    .is_some_and(|initializer| place_in(&index_reference_aliases, initializer))
                {
                    clear_place_facts(
                        place,
                        &mut index_aliases,
                        &mut index_reference_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                    push_place(&mut index_reference_aliases, place);
                } else {
                    clear_place_facts(
                        place,
                        &mut index_aliases,
                        &mut index_reference_aliases,
                        &mut one_values,
                        &mut increment_values,
                    );
                }
            }
            ResourceOp::Drop { place, .. } => clear_place_facts(
                place,
                &mut index_aliases,
                &mut index_reference_aliases,
                &mut one_values,
                &mut increment_values,
            ),
            ResourceOp::Borrow { source, output, .. } => {
                clear_place_facts(
                    output,
                    &mut index_aliases,
                    &mut index_reference_aliases,
                    &mut one_values,
                    &mut increment_values,
                );
                if place_touches(source, &index) || place_in(&index_aliases, source) {
                    push_place(&mut index_reference_aliases, output);
                }
            }
            ResourceOp::Construct { output, inputs, .. } => {
                let contains_index_reference = inputs
                    .iter()
                    .any(|input| place_in(&index_reference_aliases, input));
                clear_place_facts(
                    output,
                    &mut index_aliases,
                    &mut index_reference_aliases,
                    &mut one_values,
                    &mut increment_values,
                );
                if contains_index_reference {
                    push_place(&mut index_reference_aliases, output);
                }
            }
            ResourceOp::FunctionValue { output, .. } => clear_place_facts(
                output,
                &mut index_aliases,
                &mut index_reference_aliases,
                &mut one_values,
                &mut increment_values,
            ),
            ResourceOp::RawMemory {
                operation, output, ..
            } => {
                if !matches!(operation, RawMemoryOp::Load) {
                    return None;
                }
                clear_place_facts(
                    output,
                    &mut index_aliases,
                    &mut index_reference_aliases,
                    &mut one_values,
                    &mut increment_values,
                );
            }
            ResourceOp::RawAddressAlias { target, .. }
            | ResourceOp::RawAddressView { target, .. }
            | ResourceOp::StorageOrigin { target, .. } => {
                if place_touches(target, &index) {
                    return None;
                }
            }
            ResourceOp::CollectionSlotLifecycle { .. }
            | ResourceOp::CollectionStorageRelocate { .. }
            | ResourceOp::CollectionSlotDropTraversal { .. }
            | ResourceOp::CollectionSlotTransformRange { .. } => {
                return None;
            }
            ResourceOp::CallEffect { .. } | ResourceOp::EndScope { .. } => {}
        }
    }
    step
}

fn i32_adds_one_to_index(
    target: &ResourceCallTarget,
    args: &[Place],
    index_aliases: &[Place],
    one_values: &[Place],
) -> bool {
    if super::scalar_primitive::I32ArithmeticPrimitive::from_resource_call_target(target)
        != Some(super::scalar_primitive::I32ArithmeticPrimitive::Add)
    {
        return false;
    }
    let [left, right] = args else {
        return false;
    };
    (place_in(index_aliases, left) && place_in(one_values, right))
        || (place_in(index_aliases, right) && place_in(one_values, left))
}

fn call_may_mutate_index(
    output: &Place,
    args: &[Place],
    index: &Place,
    index_reference_aliases: &[Place],
) -> bool {
    place_touches(output, index)
        || args
            .iter()
            .any(|arg| place_touches(arg, index) || place_in(index_reference_aliases, arg))
}

fn place_touches(left: &Place, right: &Place) -> bool {
    place_suffix_after_prefix(left, right).is_some()
        || place_suffix_after_prefix(right, left).is_some()
}

fn clear_place_facts(
    place: &Place,
    index_aliases: &mut Vec<Place>,
    index_reference_aliases: &mut Vec<Place>,
    one_values: &mut Vec<Place>,
    increment_values: &mut Vec<Place>,
) {
    index_aliases.retain(|existing| existing != place);
    index_reference_aliases.retain(|existing| existing != place);
    one_values.retain(|existing| existing != place);
    increment_values.retain(|existing| existing != place);
}

fn place_in(places: &[Place], place: &Place) -> bool {
    places.iter().any(|existing| existing == place)
}

fn push_place(places: &mut Vec<Place>, place: &Place) {
    if !place_in(places, place) {
        places.push(place.clone());
    }
}
