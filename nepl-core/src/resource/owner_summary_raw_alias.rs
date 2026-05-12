use alloc::vec::Vec;

use super::model::{Place, ResourceOp};
use super::place_utils::{
    construct_aggregate_field_place, place_suffix_after_prefix, push_unique_place,
};

pub(super) fn collect_raw_owner_aliases(ops: &[ResourceOp], aliases: &mut Vec<Place>) {
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
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                ..
            } => {
                for (index, input) in inputs.iter().enumerate() {
                    if place_matches_any_alias(input, aliases) {
                        push_unique_place(
                            aliases,
                            &construct_aggregate_field_place(output, kind, index, input),
                        );
                    }
                }
            }
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } => {
                let mut then_aliases = aliases.clone();
                collect_raw_owner_aliases(then_ops, &mut then_aliases);
                let mut else_aliases = aliases.clone();
                collect_raw_owner_aliases(else_ops, &mut else_aliases);
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
                collect_raw_owner_aliases(condition_ops, aliases);
                collect_raw_owner_aliases(body_ops, aliases);
            }
            ResourceOp::Match { output, arms, .. } => {
                let mut output_alias = false;
                for arm in arms {
                    let mut arm_aliases = aliases.clone();
                    collect_raw_owner_aliases(&arm.ops, &mut arm_aliases);
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
            | ResourceOp::RawMemory { .. }
            | ResourceOp::StorageOrigin { .. } => {}
        }
    }
}

pub(super) fn place_matches_any_alias(place: &Place, aliases: &[Place]) -> bool {
    aliases.iter().any(|alias| {
        place == alias
            || place_suffix_after_prefix(place, alias).is_some()
            || place_suffix_after_prefix(alias, place).is_some()
    })
}
