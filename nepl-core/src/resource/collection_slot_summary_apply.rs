extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummary, CollectionSlotLifecycleReturnTransfer,
    CollectionSlotLifecycleSummaryOp, CollectionSlotLifecycleSummaryPlace,
};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceCallTarget};
use super::place_utils::projected_place_with_concrete_type;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_call_collection_slot_lifecycle_summary(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
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
            raw_aliases,
            output,
            args,
            summary,
            span,
        );
    }

    pub(super) fn apply_indirect_call_collection_slot_lifecycle_summary(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
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
                    &mut path,
                    raw_aliases,
                    output,
                    args,
                    summary,
                    span,
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
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        summary: &CollectionSlotLifecycleFunctionSummary,
        span: crate::span::Span,
    ) {
        self.apply_collection_slot_lifecycle_summary_ops(
            collection_slots,
            raw_aliases,
            args,
            &summary.ops,
            span,
        );
        collection_slots.clear_storage_prefix(output);
        self.apply_collection_slot_return_transfers(
            collection_slots,
            raw_aliases,
            args,
            output,
            &summary.return_transfers,
            span,
        );
        self.apply_collection_slot_return_slots(collection_slots, output, &summary.return_slots);
    }

    fn apply_collection_slot_lifecycle_summary_ops(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        args: &[Place],
        ops: &[CollectionSlotLifecycleSummaryOp],
        span: crate::span::Span,
    ) {
        for op in ops {
            match op {
                CollectionSlotLifecycleSummaryOp::Event { target, event } => {
                    if let Some(target) = instantiate_summary_target(self, args, target) {
                        self.apply_collection_slot_lifecycle_with_aliases(
                            collection_slots,
                            raw_aliases,
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
                    self.apply_collection_storage_relocate_with_aliases(
                        collection_slots,
                        raw_aliases,
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
                            &mut path,
                            raw_aliases,
                            args,
                            path_ops,
                            span,
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
                        raw_aliases,
                        args,
                        condition_ops,
                        span,
                    );
                    let exit_path = condition_path.clone();
                    let mut body_path = condition_path;
                    self.apply_collection_slot_lifecycle_summary_ops(
                        &mut body_path,
                        raw_aliases,
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

    fn apply_collection_slot_return_transfers(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        args: &[Place],
        output: &Place,
        transfers: &[CollectionSlotLifecycleReturnTransfer],
        span: crate::span::Span,
    ) {
        for transfer in transfers {
            let Some(source) = instantiate_summary_target(self, args, &transfer.source) else {
                continue;
            };
            let source = raw_aliases.canonicalize(&source);
            let target = super::place_utils::place_with_suffix(
                output,
                &transfer.target_suffix,
                transfer.target_ty,
            );
            let target = raw_aliases.canonicalize(&target);
            self.transfer_slot_state(collection_slots, &source, &target, span);
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
