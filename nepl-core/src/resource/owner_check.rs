extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeKind};

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    EffectOp, OwnerState, OwnerStateEntry, Place, RawMemoryOp, ResourceBlock,
    ResourceConditionFact, ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::owner_state::OwnerTable;
use super::place_utils::{
    match_arm_variant_payload_name, match_bind_payload_place, raw_memory_cell_place,
};
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

    fn place_is_never(&self, place: &Place) -> bool {
        matches!(
            self.types.get_ref(self.types.resolve_id(place.ty)),
            TypeKind::Never
        )
    }

    fn apply_branch_condition_fact(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        fact: Option<&ResourceConditionFact>,
        truthy_path: bool,
    ) {
        let Some(fact) = fact else {
            return;
        };
        match fact {
            ResourceConditionFact::EqZero { place } if truthy_path => {
                self.discard_non_owned_raw_address_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    place,
                );
            }
            ResourceConditionFact::NeZero { place } if !truthy_path => {
                self.discard_non_owned_raw_address_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    place,
                );
            }
            ResourceConditionFact::Positive { place } if !truthy_path => {
                self.discard_non_owned_raw_address_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    place,
                );
            }
            ResourceConditionFact::NonPositive { place } if truthy_path => {
                self.discard_non_owned_raw_address_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    place,
                );
            }
            ResourceConditionFact::EqZero { .. }
            | ResourceConditionFact::NeZero { .. }
            | ResourceConditionFact::Positive { .. }
            | ResourceConditionFact::NonPositive { .. } => {}
        }
    }

    fn discard_non_owned_raw_address_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        place: &Place,
    ) {
        let resolved_place =
            super::owner_flow::resolve_owner_alias_place(owners, raw_aliases, place);
        let descendants = owners.descendant_entries(&resolved_place);
        owners.set_state(&resolved_place, OwnerState::NoFreeObligation);
        storage_origins.clear(&resolved_place);
        raw_aliases.clear(place);
        raw_aliases.clear(&resolved_place);
        for entry in descendants {
            owners.set_state(&entry.place, OwnerState::NoFreeObligation);
            storage_origins.clear(&entry.place);
            raw_aliases.clear(&entry.place);
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
            .any(|alias| alias != source && owners.has_transferable_owner(alias))
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
                RawMemoryOp::Load => {
                    if let Some(address) = args.first() {
                        let address = raw_aliases.canonicalize(address);
                        let cell = raw_memory_cell_place(&address, output.ty);
                        self.transfer_owner(
                            owners,
                            raw_aliases,
                            storage_origins,
                            &cell,
                            output,
                            ResourceOwnerOperation::RawMemoryLoadCell,
                            *span,
                        );
                    }
                }
                RawMemoryOp::Store => {
                    if let [address, value, ..] = args.as_slice() {
                        let address = raw_aliases.canonicalize(address);
                        let cell = raw_memory_cell_place(&address, value.ty);
                        self.report_overwritten_owners(
                            owners,
                            storage_origins,
                            &cell,
                            value,
                            *span,
                        );
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
                RawMemoryOp::BulkCopy
                | RawMemoryOp::BulkMove
                | RawMemoryOp::MemorySize
                | RawMemoryOp::MemoryGrow
                | RawMemoryOp::Fill
                | RawMemoryOp::Other { .. } => {}
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
                let mut then_owners = owners.clone();
                let mut else_owners = owners.clone();
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                let mut then_raw_aliases = raw_aliases.clone();
                let mut else_raw_aliases = raw_aliases.clone();
                let mut then_storage_origins = storage_origins.clone();
                let mut else_storage_origins = storage_origins.clone();
                self.apply_branch_condition_fact(
                    &mut then_owners,
                    &mut then_raw_aliases,
                    &mut then_storage_origins,
                    condition_fact.as_ref(),
                    true,
                );
                self.apply_branch_condition_fact(
                    &mut else_owners,
                    &mut else_raw_aliases,
                    &mut else_storage_origins,
                    condition_fact.as_ref(),
                    false,
                );
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
                let mut owner_paths = Vec::new();
                let mut function_alias_paths = Vec::new();
                let mut raw_alias_paths = Vec::new();
                let mut storage_origin_paths = Vec::new();
                if !self.place_is_never(then_value) {
                    self.transfer_owner(
                        &mut then_owners,
                        &mut then_raw_aliases,
                        &mut then_storage_origins,
                        then_value,
                        output,
                        ResourceOwnerOperation::BranchValue,
                        *span,
                    );
                    owner_paths.push(then_owners);
                    function_alias_paths.push(then_function_aliases);
                    raw_alias_paths.push(then_raw_aliases);
                    storage_origin_paths.push(then_storage_origins);
                }
                if !self.place_is_never(else_value) {
                    self.transfer_owner(
                        &mut else_owners,
                        &mut else_raw_aliases,
                        &mut else_storage_origins,
                        else_value,
                        output,
                        ResourceOwnerOperation::BranchValue,
                        *span,
                    );
                    owner_paths.push(else_owners);
                    function_alias_paths.push(else_function_aliases);
                    raw_alias_paths.push(else_raw_aliases);
                    storage_origin_paths.push(else_storage_origins);
                }
                if !owner_paths.is_empty() {
                    *owners = OwnerTable::merge_paths(&owner_paths);
                    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
                    *raw_aliases = RawCellAddressAliases::merge_paths(&raw_alias_paths);
                    *storage_origins = StorageOriginTable::merge_paths(&storage_origin_paths);
                }
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
                output,
                scrutinee,
                arms,
                span,
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
                    if let Some(selected_variant) = match_arm_variant_payload_name(arm) {
                        for inactive_payload in
                            arm_owners.sibling_enum_payload_places(scrutinee, selected_variant)
                        {
                            arm_owners.set_state(&inactive_payload, OwnerState::NoFreeObligation);
                            arm_raw_aliases.clear(&inactive_payload);
                            arm_storage_origins.clear(&inactive_payload);
                        }
                    }
                    if let Some(bind_local) = &arm.bind_local {
                        if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                            self.transfer_owner(
                                &mut arm_owners,
                                &mut arm_raw_aliases,
                                &mut arm_storage_origins,
                                &source,
                                bind_local,
                                ResourceOwnerOperation::MatchValue,
                                *span,
                            );
                            arm_function_aliases.copy_alias(&source, bind_local);
                        } else {
                            arm_raw_aliases.clear(bind_local);
                            arm_storage_origins.clear(bind_local);
                        }
                    }
                    self.check_ops(
                        &mut arm_owners,
                        &mut arm_function_aliases,
                        &mut arm_raw_aliases,
                        &mut arm_storage_origins,
                        &arm.ops,
                    );
                    if !self.place_is_never(&arm.value) {
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
                effect,
                span,
                ..
            } => {
                if !direct_raw_memory_effect(effect) {
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

fn direct_raw_memory_effect(effect: &EffectOp) -> bool {
    matches!(
        effect,
        EffectOp::InternalAlloc | EffectOp::UnsafeMemory { .. }
    )
}
