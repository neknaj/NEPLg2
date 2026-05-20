use alloc::vec::Vec;

use super::model::{Place, ResourceOp};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_summary_raw_alias_branch::collect_branch_raw_owner_aliases;
use super::owner_summary_raw_transfer::{
    push_transferred_raw_owner_view_aliases, push_transferred_value_aliases,
    push_transferred_value_aliases_from,
};
use super::owner_summary_raw_use_return::push_direct_call_returned_raw_owner_aliases;
use super::place_utils::{construct_aggregate_field_place, match_bind_payload_place};
use super::summary::OwnerReturnSummaryIndex;

pub(super) fn collect_raw_owner_aliases_with_views(
    ops: &[ResourceOp],
    aliases: &mut Vec<Place>,
    raw_views: &mut RawAddressViewTable,
    summaries: &OwnerReturnSummaryIndex<'_>,
) {
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
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } => collect_branch_raw_owner_aliases(
                aliases, raw_views, output, then_ops, then_value, else_ops, else_value, summaries,
            ),
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                collect_raw_owner_aliases_with_views(condition_ops, aliases, raw_views, summaries);
                collect_raw_owner_aliases_with_views(body_ops, aliases, raw_views, summaries);
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
                    collect_raw_owner_aliases_with_views(
                        &arm.ops,
                        &mut arm_aliases,
                        &mut arm_raw_views,
                        summaries,
                    );
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
            | ResourceOp::RawMemory { .. }
            | ResourceOp::StorageOrigin { .. }
            | ResourceOp::CollectionSlotLifecycle { .. } => {}
            ResourceOp::Call {
                output,
                target,
                args,
                ..
            } => push_direct_call_returned_raw_owner_aliases(
                output, target, args, aliases, raw_views, summaries,
            ),
        }
    }
}
