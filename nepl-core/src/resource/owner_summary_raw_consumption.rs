use alloc::vec;
use alloc::vec::Vec;

use super::model::{Place, RawMemoryOp, ResourceFunction, ResourceOp};
use super::place_utils::{place_suffix_after_prefix, push_unique_place};

pub(super) fn function_has_raw_owner_consumption(function: &ResourceFunction) -> bool {
    function.blocks.iter().any(|block| {
        function.params.iter().any(|param| {
            let mut aliases = vec![param.place.clone()];
            ops_use_raw_owner_alias(&block.ops, &mut aliases)
        })
    })
}

fn ops_use_raw_owner_alias(ops: &[ResourceOp], aliases: &mut Vec<Place>) -> bool {
    for op in ops {
        match op {
            ResourceOp::Read { source, output, .. }
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
                if place_matches_any_alias(source, aliases) {
                    push_unique_place(aliases, output);
                }
            }
            ResourceOp::Assign { target, value, .. } => {
                if place_matches_any_alias(value, aliases) {
                    push_unique_place(aliases, target);
                }
            }
            ResourceOp::RawMemory {
                operation, args, ..
            } => match operation {
                RawMemoryOp::Dealloc | RawMemoryOp::Realloc => {
                    if args
                        .first()
                        .is_some_and(|arg| place_matches_any_alias(arg, aliases))
                    {
                        return true;
                    }
                }
                _ => {}
            },
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } => {
                let mut then_aliases = aliases.clone();
                if ops_use_raw_owner_alias(then_ops, &mut then_aliases) {
                    return true;
                }
                let mut else_aliases = aliases.clone();
                if ops_use_raw_owner_alias(else_ops, &mut else_aliases) {
                    return true;
                }
                if place_matches_any_alias(then_value, &then_aliases)
                    || place_matches_any_alias(else_value, &else_aliases)
                {
                    push_unique_place(aliases, output);
                }
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut loop_aliases = aliases.clone();
                if ops_use_raw_owner_alias(condition_ops, &mut loop_aliases)
                    || ops_use_raw_owner_alias(body_ops, &mut loop_aliases)
                {
                    return true;
                }
            }
            ResourceOp::Match { output, arms, .. } => {
                let mut output_alias = false;
                for arm in arms {
                    let mut arm_aliases = aliases.clone();
                    if ops_use_raw_owner_alias(&arm.ops, &mut arm_aliases) {
                        return true;
                    }
                    output_alias |= place_matches_any_alias(&arm.value, &arm_aliases);
                }
                if output_alias {
                    push_unique_place(aliases, output);
                }
            }
            ResourceOp::Expr { .. }
            | ResourceOp::DeclareLocal { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::EndScope { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::Call { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
    false
}

fn place_matches_any_alias(place: &Place, aliases: &[Place]) -> bool {
    aliases.iter().any(|alias| {
        place == alias
            || place_suffix_after_prefix(place, alias).is_some()
            || place_suffix_after_prefix(alias, place).is_some()
    })
}
