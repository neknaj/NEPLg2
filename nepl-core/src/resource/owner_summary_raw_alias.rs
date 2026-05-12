use alloc::vec::Vec;

use super::model::{Place, ResourceOp};
use super::owner_summary_raw_transfer::{push_transferred_aliases, push_transferred_aliases_from};
use super::place_utils::{construct_aggregate_field_place, match_bind_payload_place};

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
                push_transferred_aliases(aliases, source, output);
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
                push_transferred_aliases_from(aliases, then_value, output, &then_aliases);
                push_transferred_aliases_from(aliases, else_value, output, &else_aliases);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                collect_raw_owner_aliases(condition_ops, aliases);
                collect_raw_owner_aliases(body_ops, aliases);
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
                    collect_raw_owner_aliases(&arm.ops, &mut arm_aliases);
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
            | ResourceOp::RawMemory { .. }
            | ResourceOp::StorageOrigin { .. } => {}
        }
    }
}
