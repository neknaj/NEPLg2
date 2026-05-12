use alloc::vec::Vec;

use super::model::{Place, RawMemoryOp, ResourceOp};
use super::owner_summary_raw_transfer::{
    place_matches_any_alias, push_transferred_aliases, push_transferred_aliases_from,
    push_transferred_raw_owner_view_aliases,
};
use super::place_utils::{construct_aggregate_field_place, match_bind_payload_place};

pub(super) fn ops_use_raw_owner_alias(ops: &[ResourceOp], aliases: &mut Vec<Place>) -> bool {
    for op in ops {
        match op {
            ResourceOp::Read { source, output, .. }
            | ResourceOp::Move { source, output, .. }
            | ResourceOp::RawAddressAlias {
                source,
                target: output,
                ..
            } => {
                push_transferred_aliases(aliases, source, output);
            }
            ResourceOp::RawAddressView {
                source,
                target: output,
                kind,
                ..
            } => {
                push_transferred_raw_owner_view_aliases(aliases, source, output, *kind);
            }
            ResourceOp::Assign { target, value, .. } => {
                push_transferred_aliases(aliases, value, target);
            }
            ResourceOp::DeclareLocal {
                place,
                initializer: Some(initializer),
                ..
            } => {
                push_transferred_aliases(aliases, initializer, place);
            }
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                ..
            } => {
                for (index, input) in inputs.iter().enumerate() {
                    let field = construct_aggregate_field_place(output, kind, index, input);
                    push_transferred_aliases(aliases, input, &field);
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
                RawMemoryOp::Store => {
                    if args
                        .get(1)
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
                push_transferred_aliases_from(aliases, then_value, output, &then_aliases);
                push_transferred_aliases_from(aliases, else_value, output, &else_aliases);
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
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                ..
            } => {
                for arm in arms {
                    let mut arm_aliases = aliases.clone();
                    if let Some(bind_local) = &arm.bind_local {
                        if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                            push_transferred_aliases(&mut arm_aliases, &source, bind_local);
                        }
                    }
                    if ops_use_raw_owner_alias(&arm.ops, &mut arm_aliases) {
                        return true;
                    }
                    push_transferred_aliases_from(aliases, &arm.value, output, &arm_aliases);
                }
            }
            ResourceOp::Expr { .. }
            | ResourceOp::DeclareLocal {
                initializer: None, ..
            }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::EndScope { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::Call { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::StorageOrigin { .. } => {}
        }
    }
    false
}
