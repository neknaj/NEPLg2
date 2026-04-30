extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    EffectOp, OwnerState, OwnerStateEntry, Place, RawMemoryOp, ResourceBlock, ResourceFunction,
    ResourceModule, ResourceOp, ResourceTerminator,
};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::place_utils::raw_memory_cell_place;
use super::raw_realloc::PendingRawReallocs;
use super::report::{
    ResourceOwnerCheckDeferred, ResourceOwnerCheckReport, ResourceOwnerDiagnostic,
    ResourceOwnerFunctionCheck, ResourceOwnerOperation,
};
use super::storage_origin::StorageOriginTable;
use super::summary::{compute_owner_return_summaries, OwnerReturnSummary};

pub fn check_resource_owner_obligations(
    module: &ResourceModule,
    types: &TypeCtx,
) -> ResourceOwnerCheckReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceOwnerCheckDeferred::default();
    let summaries = compute_owner_return_summaries(module, types);

    for function in &module.functions {
        let mut engine = ResourceOwnerCheckEngine {
            function: function.name.as_str(),
            types,
            summaries: &summaries,
            diagnostics: Vec::new(),
            deferred: ResourceOwnerCheckDeferred::default(),
        };
        let final_owners = engine.check_function(function);
        merge_owner_deferred(&mut deferred, engine.deferred);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceOwnerFunctionCheck {
            name: function.name.clone(),
            final_owners,
            deferred: engine.deferred,
        });
    }

    ResourceOwnerCheckReport {
        functions,
        diagnostics,
        deferred,
    }
}

pub(super) struct ResourceOwnerCheckEngine<'a> {
    pub(super) function: &'a str,
    pub(super) types: &'a TypeCtx,
    pub(super) summaries: &'a [OwnerReturnSummary],
    pub(super) diagnostics: Vec<ResourceOwnerDiagnostic>,
    pub(super) deferred: ResourceOwnerCheckDeferred,
}

impl ResourceOwnerCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<OwnerStateEntry> {
        let mut owners = OwnerTable::default();
        let mut function_aliases = FunctionAliasTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut raw_views = RawAddressViewTable::default();
        let mut storage_origins = StorageOriginTable::default();
        let mut pending_reallocs = PendingRawReallocs::default();
        for block in &function.blocks {
            self.check_block(
                &mut owners,
                &mut function_aliases,
                &mut raw_aliases,
                &mut raw_views,
                &mut storage_origins,
                &mut pending_reallocs,
                block,
            );
        }
        self.push_live_owner_diagnostics(&owners, function.span);
        owners.into_entries()
    }

    fn check_block(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        block: &ResourceBlock,
    ) {
        self.check_ops(
            owners,
            function_aliases,
            raw_aliases,
            raw_views,
            storage_origins,
            pending_reallocs,
            &block.ops,
        );
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if let Some(value) = value {
                    self.move_owner_out(
                        owners,
                        raw_aliases,
                        storage_origins,
                        value,
                        ResourceOwnerOperation::ReturnValue,
                        *span,
                    );
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    pub(super) fn check_ops(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(
                owners,
                function_aliases,
                raw_aliases,
                raw_views,
                storage_origins,
                pending_reallocs,
                op,
            );
        }
    }

    fn initializer_is_non_owning_raw_alias_view(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        source: &Place,
        target: &Place,
    ) -> bool {
        if self.types.resolve_id(source.ty) != self.types.i32()
            || self.types.resolve_id(target.ty) != self.types.i32()
            || owners.has_transferable_owner(source)
            || owners.has_tracked_state_under(source)
        {
            return false;
        }
        raw_aliases
            .aliases_for(source)
            .iter()
            .any(|alias| alias != source)
    }

    fn check_op(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        op: &ResourceOp,
    ) {
        match op {
            ResourceOp::DeclareLocal {
                place,
                initializer,
                span,
                ..
            } => {
                if let Some(initializer) = initializer {
                    if self.initializer_is_non_owning_raw_alias_view(
                        owners,
                        raw_aliases,
                        initializer,
                        place,
                    ) {
                        raw_aliases.copy_alias_or_seed(initializer, place);
                        storage_origins.copy_origin(initializer, place);
                    } else {
                        self.transfer_owner(
                            owners,
                            raw_aliases,
                            storage_origins,
                            initializer,
                            place,
                            ResourceOwnerOperation::DeclareInitializer,
                            *span,
                        );
                    }
                    function_aliases.copy_alias(initializer, place);
                    raw_views.copy(initializer, place);
                    pending_reallocs.copy_result(initializer, place);
                }
            }
            ResourceOp::Read {
                source,
                output,
                span: _,
            } => {
                raw_aliases.copy_alias_or_seed(source, output);
                storage_origins.copy_origin(source, output);
                function_aliases.copy_alias(source, output);
                raw_views.copy(source, output);
                pending_reallocs.copy_result(source, output);
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                self.report_overwritten_owners(owners, storage_origins, target, value, *span);
                if self.initializer_is_non_owning_raw_alias_view(owners, raw_aliases, value, target)
                {
                    raw_aliases.copy_alias_or_seed(value, target);
                    storage_origins.copy_origin(value, target);
                } else {
                    self.transfer_owner(
                        owners,
                        raw_aliases,
                        storage_origins,
                        value,
                        target,
                        ResourceOwnerOperation::AssignValue,
                        *span,
                    );
                }
                function_aliases.copy_alias(value, target);
                raw_views.copy(value, target);
                pending_reallocs.copy_result(value, target);
            }
            ResourceOp::Move {
                source,
                output,
                span,
            } => {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    source,
                    output,
                    ResourceOwnerOperation::Move,
                    *span,
                );
                function_aliases.copy_alias(source, output);
                raw_views.copy(source, output);
                raw_views.clear(source);
                pending_reallocs.copy_result(source, output);
            }
            ResourceOp::RawMemory {
                operation,
                output,
                args,
                span,
            } => match operation {
                RawMemoryOp::Alloc => {
                    pending_reallocs.clear_result(output);
                    owners.allocate(output);
                    raw_aliases.mark(output);
                    raw_views.clear(output);
                    storage_origins.mark_owned(output);
                }
                RawMemoryOp::Dealloc => {
                    pending_reallocs.clear_result(output);
                    if let Some(ptr) = args.first() {
                        self.release_owner(
                            owners,
                            raw_aliases,
                            storage_origins,
                            ptr,
                            ResourceOwnerOperation::Dealloc,
                            *span,
                        );
                    }
                }
                RawMemoryOp::Realloc => {
                    pending_reallocs.clear_result(output);
                    if let Some(ptr) = args.first() {
                        if self.ensure_owner_available(
                            owners,
                            raw_aliases,
                            storage_origins,
                            ptr,
                            ResourceOwnerOperation::ReallocInput,
                            *span,
                        ) {
                            owners.set_state(output, OwnerState::MaybeFreed { storage: None });
                            raw_aliases.mark(output);
                            raw_views.clear(output);
                            pending_reallocs.mark(ptr, output);
                        }
                    }
                }
                RawMemoryOp::Load => {
                    pending_reallocs.clear_result(output);
                    if let Some(address) = args.first() {
                        let address = raw_aliases.canonicalize_owner_cell_address(address);
                        let cell = raw_memory_cell_place(&address, output.ty);
                        if self.raw_memory_load_is_non_owning_raw_address_view(
                            owners,
                            raw_aliases,
                            &cell,
                            output.ty,
                        ) {
                            raw_aliases.copy_alias_or_seed(&cell, output);
                            storage_origins.copy_origin(&cell, output);
                            raw_views.mark(output);
                        } else {
                            self.transfer_owner(
                                owners,
                                raw_aliases,
                                storage_origins,
                                &cell,
                                output,
                                ResourceOwnerOperation::RawMemoryLoadCell,
                                *span,
                            );
                            raw_views.clear(output);
                        }
                    }
                }
                RawMemoryOp::Store => {
                    pending_reallocs.clear_result(output);
                    if let [address, value, ..] = args.as_slice() {
                        let address = raw_aliases.canonicalize_owner_cell_address(address);
                        let cell = raw_memory_cell_place(&address, value.ty);
                        self.report_overwritten_owners(
                            owners,
                            storage_origins,
                            &cell,
                            value,
                            *span,
                        );
                        if self.raw_store_value_is_non_owning_raw_address_view(
                            owners,
                            raw_aliases,
                            raw_views,
                            value,
                        ) {
                            raw_aliases.copy_alias_or_seed(value, &cell);
                            storage_origins.copy_origin(value, &cell);
                        } else {
                            self.transfer_owner(
                                owners,
                                raw_aliases,
                                storage_origins,
                                value,
                                &cell,
                                ResourceOwnerOperation::RawMemoryStoreValue,
                                *span,
                            );
                        }
                    }
                }
                RawMemoryOp::BulkCopy
                | RawMemoryOp::BulkMove
                | RawMemoryOp::MemorySize
                | RawMemoryOp::MemoryGrow
                | RawMemoryOp::Fill
                | RawMemoryOp::Other { .. } => {
                    pending_reallocs.clear_result(output);
                }
            },
            ResourceOp::Branch {
                output,
                condition_fact,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span,
                ..
            } => {
                self.check_branch(
                    owners,
                    function_aliases,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    pending_reallocs,
                    output,
                    condition_fact.as_ref(),
                    then_ops,
                    then_value,
                    else_ops,
                    else_value,
                    *span,
                );
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                self.check_loop(
                    owners,
                    function_aliases,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    pending_reallocs,
                    condition_ops,
                    body_ops,
                );
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
            } => {
                self.check_match(
                    owners,
                    function_aliases,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    pending_reallocs,
                    output,
                    scrutinee,
                    arms,
                    *span,
                );
            }
            ResourceOp::FunctionValue { output, name, .. } => {
                function_aliases.set_alias(output, name.clone());
                raw_views.clear(output);
                pending_reallocs.clear_result(output);
            }
            ResourceOp::Call {
                output,
                target,
                args,
                effect,
                span,
                ..
            } => {
                if !direct_raw_memory_effect(effect) {
                    raw_views.clear(output);
                    pending_reallocs.clear_result(output);
                    self.apply_call_return_owner(
                        owners,
                        raw_aliases,
                        storage_origins,
                        output,
                        target,
                        args,
                        *span,
                    );
                }
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                span,
                ..
            } => {
                raw_views.clear(output);
                pending_reallocs.clear_result(output);
                self.apply_indirect_call_return_owner(
                    owners,
                    function_aliases,
                    raw_aliases,
                    storage_origins,
                    output,
                    callee,
                    args,
                    *span,
                );
            }
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                span,
            } => {
                self.construct_owner_fields(
                    owners,
                    raw_aliases,
                    storage_origins,
                    output,
                    kind,
                    inputs,
                    *span,
                );
                construct_function_alias_fields(function_aliases, output, kind, inputs);
                raw_views.clear(output);
                pending_reallocs.clear_result(output);
            }
            ResourceOp::RawAddressAlias { source, target, .. } => {
                raw_aliases.copy_alias_or_seed(source, target);
                storage_origins.copy_origin(source, target);
                raw_views.copy(source, target);
                pending_reallocs.copy_result(source, target);
            }
            ResourceOp::RawAddressView { source, target, .. } => {
                raw_aliases.copy_alias_or_seed(source, target);
                storage_origins.copy_origin(source, target);
                raw_views.mark(target);
                pending_reallocs.clear_result(target);
            }
            ResourceOp::Borrow { output, .. } => {
                pending_reallocs.clear_result(output);
            }
            ResourceOp::Expr { .. } | ResourceOp::Drop { .. } | ResourceOp::CallEffect { .. } => {}
        }
    }
}

fn merge_owner_deferred(
    target: &mut ResourceOwnerCheckDeferred,
    source: ResourceOwnerCheckDeferred,
) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}

fn direct_raw_memory_effect(effect: &EffectOp) -> bool {
    matches!(
        effect,
        EffectOp::InternalAlloc | EffectOp::UnsafeMemory { .. }
    )
}
