extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummary, CollectionSlotLifecycleSummaryOp,
    CollectionSlotLifecycleSummaryPlace,
};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, ResourceCallTarget};
use super::place_utils::projected_place_with_concrete_type;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_call_collection_slot_lifecycle_summary(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
        span: crate::span::Span,
    ) {
        let ResourceCallTarget::User { name, .. } = target else {
            collection_slots.clear_storage_prefix(output);
            return;
        };
        let Some(summary) = self.collection_slot_summaries.get(name) else {
            collection_slots.clear_storage_prefix(output);
            return;
        };
        self.apply_collection_slot_lifecycle_function_summary(
            collection_slots,
            output,
            args,
            summary,
            span,
        );
    }

    pub(super) fn apply_indirect_call_collection_slot_lifecycle_summary(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        function_aliases: &FunctionAliasTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
        span: crate::span::Span,
    ) {
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            collection_slots.clear_storage_prefix(output);
            return;
        }
        let mut paths = Vec::new();
        for function in functions {
            let mut path = collection_slots.clone();
            if let Some(summary) = self.collection_slot_summaries.get(function) {
                self.apply_collection_slot_lifecycle_function_summary(
                    &mut path, output, args, summary, span,
                );
            } else {
                path.clear_storage_prefix(output);
            }
            paths.push(path);
        }
        *collection_slots = CollectionSlotStateTable::merge_paths(&paths);
    }

    fn apply_collection_slot_lifecycle_function_summary(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        output: &Place,
        args: &[Place],
        summary: &CollectionSlotLifecycleFunctionSummary,
        span: crate::span::Span,
    ) {
        self.apply_collection_slot_lifecycle_summary_ops(
            collection_slots,
            args,
            &summary.ops,
            span,
        );
        self.apply_collection_slot_return_slots(collection_slots, output, &summary.return_slots);
    }

    fn apply_collection_slot_lifecycle_summary_ops(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        args: &[Place],
        ops: &[CollectionSlotLifecycleSummaryOp],
        span: crate::span::Span,
    ) {
        for op in ops {
            match op {
                CollectionSlotLifecycleSummaryOp::Event { target, event } => {
                    if let Some(target) = instantiate_summary_target(self, args, target) {
                        self.apply_collection_slot_lifecycle(
                            collection_slots,
                            &target,
                            *event,
                            span,
                        );
                    }
                }
                CollectionSlotLifecycleSummaryOp::Relocate {
                    old_storage,
                    new_storage,
                } => {
                    let Some(old_storage) = instantiate_summary_target(self, args, old_storage)
                    else {
                        continue;
                    };
                    let Some(new_storage) = instantiate_summary_target(self, args, new_storage)
                    else {
                        continue;
                    };
                    self.apply_collection_storage_relocate(
                        collection_slots,
                        &old_storage,
                        &new_storage,
                        span,
                    );
                }
                CollectionSlotLifecycleSummaryOp::Merge { paths } => {
                    if paths.is_empty() {
                        continue;
                    }
                    let mut merged_paths = Vec::new();
                    for path_ops in paths {
                        let mut path = collection_slots.clone();
                        self.apply_collection_slot_lifecycle_summary_ops(
                            &mut path, args, path_ops, span,
                        );
                        merged_paths.push(path);
                    }
                    *collection_slots = CollectionSlotStateTable::merge_paths(&merged_paths);
                }
                CollectionSlotLifecycleSummaryOp::Loop {
                    condition_ops,
                    body_ops,
                } => {
                    let mut condition_path = collection_slots.clone();
                    self.apply_collection_slot_lifecycle_summary_ops(
                        &mut condition_path,
                        args,
                        condition_ops,
                        span,
                    );
                    let exit_path = condition_path.clone();
                    let mut body_path = condition_path;
                    self.apply_collection_slot_lifecycle_summary_ops(
                        &mut body_path,
                        args,
                        body_ops,
                        span,
                    );
                    *collection_slots =
                        CollectionSlotStateTable::merge_paths(&[exit_path, body_path]);
                }
            }
        }
    }
}

fn instantiate_summary_target(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    target: &CollectionSlotLifecycleSummaryPlace,
) -> Option<Place> {
    let arg = args.get(target.parameter_index)?;
    Some(projected_place_with_concrete_type(
        engine.types,
        arg,
        &target.suffix,
        target.ty,
    ))
}
