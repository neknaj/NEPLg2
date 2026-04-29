extern crate alloc;

use alloc::vec::Vec;

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    OwnerStateEntry, RawMemoryOp, ResourceBlock, ResourceFunction, ResourceModule, ResourceOp,
    ResourceTerminator,
};
use super::owner_state::OwnerTable;
use super::report::{
    ResourceOwnerCheckDeferred, ResourceOwnerCheckReport, ResourceOwnerDiagnostic,
    ResourceOwnerFunctionCheck, ResourceOwnerOperation,
};
use super::storage_origin::StorageOriginTable;
use super::summary::{compute_owner_return_summaries, OwnerReturnSummary};

pub fn check_resource_owner_obligations(module: &ResourceModule) -> ResourceOwnerCheckReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceOwnerCheckDeferred::default();
    let summaries = compute_owner_return_summaries(module);

    for function in &module.functions {
        let mut engine = ResourceOwnerCheckEngine {
            function: function.name.as_str(),
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
    pub(super) summaries: &'a [OwnerReturnSummary],
    pub(super) diagnostics: Vec<ResourceOwnerDiagnostic>,
    pub(super) deferred: ResourceOwnerCheckDeferred,
}

impl ResourceOwnerCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<OwnerStateEntry> {
        let mut owners = OwnerTable::default();
        let mut function_aliases = FunctionAliasTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut storage_origins = StorageOriginTable::default();
        for block in &function.blocks {
            self.check_block(
                &mut owners,
                &mut function_aliases,
                &mut raw_aliases,
                &mut storage_origins,
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
        storage_origins: &mut StorageOriginTable,
        block: &ResourceBlock,
    ) {
        self.check_ops(
            owners,
            function_aliases,
            raw_aliases,
            storage_origins,
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
        storage_origins: &mut StorageOriginTable,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(owners, function_aliases, raw_aliases, storage_origins, op);
        }
    }

    fn check_op(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
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
                    self.transfer_owner(
                        owners,
                        raw_aliases,
                        storage_origins,
                        initializer,
                        place,
                        ResourceOwnerOperation::DeclareInitializer,
                        *span,
                    );
                    function_aliases.copy_alias(initializer, place);
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
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                self.report_overwritten_owners(owners, storage_origins, target, value, *span);
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    value,
                    target,
                    ResourceOwnerOperation::AssignValue,
                    *span,
                );
                function_aliases.copy_alias(value, target);
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
            }
            ResourceOp::RawMemory {
                operation,
                output,
                args,
                span,
            } => match operation {
                RawMemoryOp::Alloc => {
                    owners.allocate(output);
                    raw_aliases.mark(output);
                    storage_origins.mark_owned(output);
                }
                RawMemoryOp::Dealloc => {
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
                    if let Some(ptr) = args.first() {
                        if self.release_owner(
                            owners,
                            raw_aliases,
                            storage_origins,
                            ptr,
                            ResourceOwnerOperation::ReallocInput,
                            *span,
                        ) {
                            owners.allocate(output);
                            raw_aliases.mark(output);
                            storage_origins.mark_owned(output);
                        }
                    }
                }
                RawMemoryOp::Load
                | RawMemoryOp::Store
                | RawMemoryOp::BulkCopy
                | RawMemoryOp::BulkMove
                | RawMemoryOp::MemorySize
                | RawMemoryOp::MemoryGrow
                | RawMemoryOp::Fill
                | RawMemoryOp::Other { .. } => {}
            },
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span,
                ..
            } => {
                let mut then_owners = owners.clone();
                let mut else_owners = owners.clone();
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                let mut then_raw_aliases = raw_aliases.clone();
                let mut else_raw_aliases = raw_aliases.clone();
                let mut then_storage_origins = storage_origins.clone();
                let mut else_storage_origins = storage_origins.clone();
                self.check_ops(
                    &mut then_owners,
                    &mut then_function_aliases,
                    &mut then_raw_aliases,
                    &mut then_storage_origins,
                    then_ops,
                );
                self.check_ops(
                    &mut else_owners,
                    &mut else_function_aliases,
                    &mut else_raw_aliases,
                    &mut else_storage_origins,
                    else_ops,
                );
                self.transfer_owner(
                    &mut then_owners,
                    &mut then_raw_aliases,
                    &mut then_storage_origins,
                    then_value,
                    output,
                    ResourceOwnerOperation::BranchValue,
                    *span,
                );
                self.transfer_owner(
                    &mut else_owners,
                    &mut else_raw_aliases,
                    &mut else_storage_origins,
                    else_value,
                    output,
                    ResourceOwnerOperation::BranchValue,
                    *span,
                );
                *owners = OwnerTable::merge_paths(&[then_owners, else_owners]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    then_function_aliases,
                    else_function_aliases,
                ]);
                *raw_aliases =
                    RawCellAddressAliases::merge_paths(&[then_raw_aliases, else_raw_aliases]);
                *storage_origins =
                    StorageOriginTable::merge_paths(&[then_storage_origins, else_storage_origins]);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut condition_owners = owners.clone();
                let mut condition_function_aliases = function_aliases.clone();
                let mut condition_raw_aliases = raw_aliases.clone();
                let mut condition_storage_origins = storage_origins.clone();
                self.check_ops(
                    &mut condition_owners,
                    &mut condition_function_aliases,
                    &mut condition_raw_aliases,
                    &mut condition_storage_origins,
                    condition_ops,
                );
                let mut body_owners = condition_owners.clone();
                let mut body_function_aliases = condition_function_aliases.clone();
                let mut body_raw_aliases = condition_raw_aliases.clone();
                let mut body_storage_origins = condition_storage_origins.clone();
                self.check_ops(
                    &mut body_owners,
                    &mut body_function_aliases,
                    &mut body_raw_aliases,
                    &mut body_storage_origins,
                    body_ops,
                );
                *owners = OwnerTable::merge_paths(&[condition_owners, body_owners]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    condition_function_aliases,
                    body_function_aliases,
                ]);
                *raw_aliases =
                    RawCellAddressAliases::merge_paths(&[condition_raw_aliases, body_raw_aliases]);
                *storage_origins = StorageOriginTable::merge_paths(&[
                    condition_storage_origins,
                    body_storage_origins,
                ]);
            }
            ResourceOp::Match {
                output, arms, span, ..
            } => {
                let mut arm_paths = Vec::new();
                let mut function_alias_paths = Vec::new();
                let mut raw_alias_paths = Vec::new();
                let mut storage_origin_paths = Vec::new();
                for arm in arms {
                    let mut arm_owners = owners.clone();
                    let mut arm_function_aliases = function_aliases.clone();
                    let mut arm_raw_aliases = raw_aliases.clone();
                    let mut arm_storage_origins = storage_origins.clone();
                    self.check_ops(
                        &mut arm_owners,
                        &mut arm_function_aliases,
                        &mut arm_raw_aliases,
                        &mut arm_storage_origins,
                        &arm.ops,
                    );
                    self.transfer_owner(
                        &mut arm_owners,
                        &mut arm_raw_aliases,
                        &mut arm_storage_origins,
                        &arm.value,
                        output,
                        ResourceOwnerOperation::MatchValue,
                        *span,
                    );
                    arm_paths.push(arm_owners);
                    function_alias_paths.push(arm_function_aliases);
                    raw_alias_paths.push(arm_raw_aliases);
                    storage_origin_paths.push(arm_storage_origins);
                }
                if !arm_paths.is_empty() {
                    *owners = OwnerTable::merge_paths(&arm_paths);
                    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
                    *raw_aliases = RawCellAddressAliases::merge_paths(&raw_alias_paths);
                    *storage_origins = StorageOriginTable::merge_paths(&storage_origin_paths);
                }
            }
            ResourceOp::FunctionValue { output, name, .. } => {
                function_aliases.set_alias(output, name.clone());
            }
            ResourceOp::Call {
                output,
                target,
                args,
                span,
                ..
            } => self.apply_call_return_owner(
                owners,
                raw_aliases,
                storage_origins,
                output,
                target,
                args,
                *span,
            ),
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                span,
                ..
            } => self.apply_indirect_call_return_owner(
                owners,
                function_aliases,
                raw_aliases,
                storage_origins,
                output,
                callee,
                args,
                *span,
            ),
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
            }
            ResourceOp::RawAddressAlias { source, target, .. } => {
                raw_aliases.copy_alias_or_seed(source, target);
                storage_origins.copy_origin(source, target);
            }
            ResourceOp::Expr { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::CallEffect { .. } => {}
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
