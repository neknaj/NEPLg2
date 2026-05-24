use alloc::vec::Vec;

use super::model::{Place, RawMemoryOp, ResourceOp};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_summary_raw_transfer::{
    place_matches_any_alias, push_transferred_raw_owner_view_aliases,
    push_transferred_value_aliases, push_transferred_value_aliases_from,
};
use super::owner_summary_raw_use_branch::branch_uses_raw_owner_alias;
use super::owner_summary_raw_use_call::direct_call_consumes_raw_owner_alias;
use super::owner_summary_raw_use_return::push_direct_call_returned_raw_owner_aliases;
use super::place_utils::{construct_aggregate_field_place, match_bind_payload_place};
use super::summary::OwnerReturnSummaryIndex;

pub(super) fn ops_use_raw_owner_alias_with_views(
    ops: &[ResourceOp],
    aliases: &mut Vec<Place>,
    raw_views: &mut RawAddressViewTable,
    summaries: &OwnerReturnSummaryIndex<'_>,
) -> bool {
    for op in ops {
        match op {
            ResourceOp::Read { source, output, .. }
            | ResourceOp::Move { source, output, .. }
            | ResourceOp::RawAddressAlias {
                source,
                target: output,
                ..
            } => {
                push_transferred_value_aliases(aliases, raw_views, source, output);
            }
            ResourceOp::RawAddressView {
                source,
                target: output,
                kind,
                ..
            } => {
                push_transferred_raw_owner_view_aliases(aliases, raw_views, source, output, *kind);
            }
            ResourceOp::Assign { target, value, .. } => {
                push_transferred_value_aliases(aliases, raw_views, value, target);
            }
            ResourceOp::DeclareLocal {
                place,
                initializer: Some(initializer),
                ..
            } => {
                push_transferred_value_aliases(aliases, raw_views, initializer, place);
            }
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                ..
            } => {
                for (index, input) in inputs.iter().enumerate() {
                    let field = construct_aggregate_field_place(output, kind, index, input);
                    push_transferred_value_aliases(aliases, raw_views, input, &field);
                }
            }
            ResourceOp::RawMemory {
                operation, args, ..
            } => {
                if raw_memory_uses_alias(*operation, args, aliases, raw_views) {
                    return true;
                }
            }
            ResourceOp::Call {
                output,
                target,
                args,
                ..
            } => {
                if direct_call_consumes_raw_owner_alias(target, args, aliases, summaries) {
                    return true;
                }
                push_direct_call_returned_raw_owner_aliases(
                    output, target, args, aliases, raw_views, summaries,
                );
            }
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } => {
                if branch_uses_raw_owner_alias(
                    aliases, raw_views, output, then_ops, then_value, else_ops, else_value,
                    summaries,
                ) {
                    return true;
                }
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut loop_aliases = aliases.clone();
                let mut loop_raw_views = raw_views.clone();
                if ops_use_raw_owner_alias_with_views(
                    condition_ops,
                    &mut loop_aliases,
                    &mut loop_raw_views,
                    summaries,
                ) || ops_use_raw_owner_alias_with_views(
                    body_ops,
                    &mut loop_aliases,
                    &mut loop_raw_views,
                    summaries,
                ) {
                    return true;
                }
                *raw_views = RawAddressViewTable::merge_paths(&[raw_views.clone(), loop_raw_views]);
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                ..
            } => {
                let mut arm_output_raw_views = Vec::new();
                for arm in arms {
                    let mut arm_aliases = aliases.clone();
                    let mut arm_raw_views = raw_views.clone();
                    if let Some(bind_local) = &arm.bind_local {
                        if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                            push_transferred_value_aliases(
                                &mut arm_aliases,
                                &mut arm_raw_views,
                                &source,
                                bind_local,
                            );
                        }
                    }
                    if ops_use_raw_owner_alias_with_views(
                        &arm.ops,
                        &mut arm_aliases,
                        &mut arm_raw_views,
                        summaries,
                    ) {
                        return true;
                    }
                    let mut output_raw_views = raw_views.clone();
                    push_transferred_value_aliases_from(
                        aliases,
                        &mut output_raw_views,
                        &arm.value,
                        output,
                        &arm_aliases,
                        &arm_raw_views,
                    );
                    arm_output_raw_views.push(output_raw_views);
                }
                if !arm_output_raw_views.is_empty() {
                    *raw_views = RawAddressViewTable::merge_paths(&arm_output_raw_views);
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
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::StorageOrigin { .. }
            | ResourceOp::CollectionSlotLifecycle { .. }
            | ResourceOp::CollectionStorageRelocate { .. }
            | ResourceOp::CollectionSlotDropTraversal { .. }
            | ResourceOp::CollectionSlotTransformRange { .. } => {}
        }
    }
    false
}

fn raw_memory_uses_alias(
    operation: RawMemoryOp,
    args: &[Place],
    aliases: &[Place],
    raw_views: &RawAddressViewTable,
) -> bool {
    match operation {
        RawMemoryOp::Dealloc | RawMemoryOp::Realloc => args
            .first()
            .is_some_and(|arg| place_matches_any_alias(arg, aliases)),
        RawMemoryOp::Store => args.get(1).is_some_and(|arg| {
            place_matches_any_alias(arg, aliases) && !raw_views.contains_non_owning(arg)
        }),
        _ => false,
    }
}
