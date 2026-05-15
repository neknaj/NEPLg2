extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::i32_call_facts::record_direct_call_i32_facts;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    BorrowKind, EffectOp, OwnerStateEntry, Place, ResourceBlock, ResourceFunction, ResourceOp,
    ResourceTerminator,
};
use super::owner_check_utils::{direct_raw_memory_effect, raw_owner_alias_moves_into_wrapper};
use super::owner_extent::PendingOwnerExtentRequirement;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::{
    call_uses_checked_mem_ptr_wrapper, reference_target_place,
    structural_i32_projection_preserves_raw_address, type_can_seed_raw_address_alias,
};
use super::raw_realloc::PendingRawReallocs;
use super::report::{ResourceOwnerCheckDeferred, ResourceOwnerDiagnostic, ResourceOwnerOperation};
use super::storage_origin::StorageOriginTable;
use super::summary::OwnerReturnSummaryIndex;

pub(super) struct ResourceOwnerCheckEngine<'a> {
    pub(super) function: &'a str,
    pub(super) types: &'a TypeCtx,
    pub(super) summaries: &'a OwnerReturnSummaryIndex<'a>,
    pub(super) diagnostics: Vec<ResourceOwnerDiagnostic>,
    pub(super) deferred: ResourceOwnerCheckDeferred,
    pub(super) owner_extent_requirements: Vec<PendingOwnerExtentRequirement>,
}

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn check_function(&mut self, function: &ResourceFunction) -> Vec<OwnerStateEntry> {
        let mut owners = OwnerTable::default();
        let mut function_aliases = FunctionAliasTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut raw_views = RawAddressViewTable::default();
        let mut storage_origins = StorageOriginTable::default();
        let mut pending_reallocs = PendingRawReallocs::default();
        let mut variant_owner_effects = PendingVariantOwnerEffects::default();
        for block in &function.blocks {
            self.check_block(
                &mut owners,
                &mut function_aliases,
                &mut raw_aliases,
                &mut raw_views,
                &mut storage_origins,
                &mut pending_reallocs,
                &mut variant_owner_effects,
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
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        block: &ResourceBlock,
    ) {
        self.check_ops(
            owners,
            function_aliases,
            raw_aliases,
            raw_views,
            storage_origins,
            pending_reallocs,
            variant_owner_effects,
            &block.ops,
        );
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if let Some(value) = value {
                    variant_owner_effects.materialize_result_owner_effects(
                        self,
                        owners,
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        value,
                        *span,
                    );
                    if !variant_owner_effects.reject_reserved_source_use(
                        self,
                        owners,
                        raw_aliases,
                        value,
                        ResourceOwnerOperation::ReturnValue,
                        *span,
                    ) {
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
        variant_owner_effects: &mut PendingVariantOwnerEffects,
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
                variant_owner_effects,
                op,
            );
        }
    }

    fn check_op(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_owner_effects: &mut PendingVariantOwnerEffects,
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
                    if variant_owner_effects.reject_reserved_source_use(
                        self,
                        owners,
                        raw_aliases,
                        initializer,
                        ResourceOwnerOperation::DeclareInitializer,
                        *span,
                    ) {
                        raw_aliases.clear(place);
                        raw_views.clear(place);
                        storage_origins.clear(place);
                    } else {
                        raw_aliases.copy_scalar_facts_if_tracked(initializer, place);
                        self.copy_non_owning_raw_address_view_aliases(
                            raw_aliases,
                            raw_views,
                            storage_origins,
                            initializer,
                            place,
                        );
                        if self.initializer_is_non_owning_raw_alias_view(
                            owners,
                            raw_aliases,
                            initializer,
                            place,
                        ) {
                            raw_aliases.copy_alias_if_tracked(initializer, place);
                            storage_origins.copy_origin(initializer, place);
                        } else {
                            self.transfer_owner(
                                owners,
                                raw_aliases,
                                raw_views,
                                storage_origins,
                                initializer,
                                place,
                                ResourceOwnerOperation::DeclareInitializer,
                                *span,
                            );
                        }
                    }
                    function_aliases.copy_alias(initializer, place);
                    raw_views.copy(initializer, place);
                    pending_reallocs.copy_result(initializer, place);
                    variant_owner_effects.copy_result(initializer, place);
                } else {
                    variant_owner_effects.clear_result(place);
                }
            }
            ResourceOp::Read {
                source,
                output,
                span,
            } => {
                let source_is_copy = self.types.is_copy(source.ty);
                variant_owner_effects.reject_reserved_source_use(
                    self,
                    owners,
                    raw_aliases,
                    source,
                    ResourceOwnerOperation::Read,
                    *span,
                );
                if !source_is_copy {
                    raw_aliases.copy_scalar_facts_if_tracked(source, output);
                    self.transfer_owner(
                        owners,
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        source,
                        output,
                        ResourceOwnerOperation::Read,
                        *span,
                    );
                } else {
                    if structural_i32_projection_preserves_raw_address(self.types, source, output) {
                        raw_aliases.copy_explicit_raw_address_alias(source, output);
                    } else {
                        self.copy_non_owning_raw_address_view_aliases(
                            raw_aliases,
                            raw_views,
                            storage_origins,
                            source,
                            output,
                        );
                        raw_aliases.copy_alias_if_tracked(source, output);
                    }
                    storage_origins.copy_origin(source, output);
                }
                function_aliases.copy_alias(source, output);
                raw_views.copy(source, output);
                if !source_is_copy {
                    raw_views.clear(source);
                }
                pending_reallocs.copy_result(source, output);
                variant_owner_effects.copy_result(source, output);
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                self.report_overwritten_owners(
                    owners,
                    raw_aliases,
                    storage_origins,
                    target,
                    value,
                    *span,
                );
                if variant_owner_effects.reject_reserved_source_use(
                    self,
                    owners,
                    raw_aliases,
                    value,
                    ResourceOwnerOperation::AssignValue,
                    *span,
                ) {
                    raw_aliases.clear(target);
                    raw_views.clear(target);
                    storage_origins.clear(target);
                } else {
                    raw_aliases.copy_scalar_facts_if_tracked(value, target);
                    self.copy_non_owning_raw_address_view_aliases(
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        value,
                        target,
                    );
                    if self.initializer_is_non_owning_raw_alias_view(
                        owners,
                        raw_aliases,
                        value,
                        target,
                    ) {
                        raw_aliases.copy_alias_if_tracked(value, target);
                        storage_origins.copy_origin(value, target);
                    } else {
                        self.transfer_owner(
                            owners,
                            raw_aliases,
                            raw_views,
                            storage_origins,
                            value,
                            target,
                            ResourceOwnerOperation::AssignValue,
                            *span,
                        );
                    }
                }
                function_aliases.copy_alias(value, target);
                raw_views.copy(value, target);
                pending_reallocs.copy_result(value, target);
                variant_owner_effects.copy_result(value, target);
            }
            ResourceOp::Move {
                source,
                output,
                span,
            } => {
                if !variant_owner_effects.reject_reserved_source_use(
                    self,
                    owners,
                    raw_aliases,
                    source,
                    ResourceOwnerOperation::Move,
                    *span,
                ) {
                    raw_aliases.copy_scalar_facts_if_tracked(source, output);
                    self.transfer_owner(
                        owners,
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        source,
                        output,
                        ResourceOwnerOperation::Move,
                        *span,
                    );
                }
                function_aliases.copy_alias(source, output);
                raw_views.copy(source, output);
                raw_views.clear(source);
                pending_reallocs.copy_result(source, output);
                variant_owner_effects.copy_result(source, output);
            }
            ResourceOp::RawMemory {
                operation,
                output,
                args,
                span,
            } => self.check_raw_memory_op(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                pending_reallocs,
                variant_owner_effects,
                *operation,
                output,
                args,
                *span,
            ),
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
                    variant_owner_effects,
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
                condition_fact,
                body_ops,
                span,
                ..
            } => {
                self.check_loop(
                    owners,
                    function_aliases,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    pending_reallocs,
                    variant_owner_effects,
                    condition_ops,
                    condition_fact.as_ref(),
                    body_ops,
                    *span,
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
                    variant_owner_effects,
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
                variant_owner_effects.clear_result(output);
            }
            ResourceOp::Call {
                output,
                target,
                args,
                effect,
                span,
                ..
            } => {
                let checked_mem_ptr_wrapper = matches!(effect, EffectOp::UnsafeMemory { .. })
                    && call_uses_checked_mem_ptr_wrapper(self.types, args);
                if !direct_raw_memory_effect(effect) || checked_mem_ptr_wrapper {
                    raw_views.clear(output);
                    pending_reallocs.clear_result(output);
                    variant_owner_effects.clear_result(output);
                    if !self.reject_reserved_call_arguments(
                        owners,
                        raw_aliases,
                        variant_owner_effects,
                        args,
                        *span,
                    ) {
                        self.apply_call_return_owner(
                            owners,
                            raw_aliases,
                            raw_views,
                            storage_origins,
                            variant_owner_effects,
                            output,
                            target,
                            args,
                            !checked_mem_ptr_wrapper,
                            *span,
                        );
                    }
                }
                record_direct_call_i32_facts(raw_aliases, target, output, args);
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
                variant_owner_effects.clear_result(output);
                if !self.reject_reserved_call_arguments(
                    owners,
                    raw_aliases,
                    variant_owner_effects,
                    args,
                    *span,
                ) {
                    self.apply_indirect_call_return_owner(
                        owners,
                        function_aliases,
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        variant_owner_effects,
                        output,
                        callee,
                        args,
                        *span,
                    );
                }
            }
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                span,
            } => {
                raw_views.clear(output);
                if !self.reject_reserved_call_arguments(
                    owners,
                    raw_aliases,
                    variant_owner_effects,
                    inputs,
                    *span,
                ) {
                    self.construct_owner_fields(
                        owners,
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        output,
                        kind,
                        inputs,
                        *span,
                    );
                }
                construct_function_alias_fields(function_aliases, output, kind, inputs);
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
            }
            ResourceOp::RawAddressAlias {
                source,
                target,
                span,
            } => {
                let moves_into_owner_wrapper = raw_owner_alias_moves_into_wrapper(source, target);
                let source_transfers_owner = moves_into_owner_wrapper
                    && self.has_transferable_owner(owners, raw_aliases, source);
                let target_already_owns = moves_into_owner_wrapper
                    && self.has_transferable_owner(owners, raw_aliases, target);
                if source_transfers_owner {
                    self.transfer_owner(
                        owners,
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        source,
                        target,
                        ResourceOwnerOperation::Move,
                        *span,
                    );
                }
                if source_transfers_owner || target_already_owns {
                    raw_aliases.copy_explicit_raw_address_alias_preserving_target(source, target);
                } else {
                    raw_aliases.copy_explicit_raw_address_alias(source, target);
                }
                storage_origins.copy_origin(source, target);
                raw_views.copy(source, target);
                pending_reallocs.copy_result(source, target);
                variant_owner_effects.copy_result(source, target);
            }
            ResourceOp::RawAddressView {
                source,
                target,
                kind,
                ..
            } => {
                self.apply_raw_address_view(
                    owners,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    source,
                    target,
                    *kind,
                );
                pending_reallocs.clear_result(target);
                variant_owner_effects.clear_result(target);
            }
            ResourceOp::StorageOrigin { target, origin, .. } => {
                storage_origins.mark_origin(target, *origin);
            }
            ResourceOp::Borrow {
                source,
                output,
                kind,
                ..
            } => {
                let target = reference_target_place(output, source.ty);
                match kind {
                    BorrowKind::Shared => {
                        raw_aliases.clear(&target);
                        storage_origins.clear(&target);
                        raw_views.copy_non_owning(source, &target);
                    }
                    BorrowKind::Unique => {
                        raw_aliases.copy_alias_if_tracked(source, &target);
                        storage_origins.copy_origin(source, &target);
                        raw_views.copy(source, &target);
                    }
                }
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
            }
            ResourceOp::Expr { output, kind, .. } => {
                self.check_expr(raw_aliases, *kind, output);
            }
            ResourceOp::Drop { place, span } => {
                self.drop_owner_obligation(
                    owners,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    pending_reallocs,
                    place,
                    *span,
                );
                variant_owner_effects.clear_result(place);
            }
            ResourceOp::EndScope {
                locals,
                result,
                span,
            } => {
                self.auto_drop_scope_owner_obligations(
                    owners,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    pending_reallocs,
                    locals,
                    result.as_ref(),
                    *span,
                );
            }
            ResourceOp::CallEffect { .. } => {}
        }
    }

    fn copy_non_owning_raw_address_view_aliases(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        source: &Place,
        target: &Place,
    ) {
        if !type_can_seed_raw_address_alias(self.types, source.ty)
            && !type_can_seed_raw_address_alias(self.types, target.ty)
        {
            return;
        }
        if !raw_views.contains_non_owning_under(source) {
            return;
        }
        raw_aliases.copy_alias_if_tracked(source, target);
        storage_origins.copy_origin(source, target);
    }
}
