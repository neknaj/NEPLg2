extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::{
    apply_direct_call_raw_alias_summary, apply_indirect_call_raw_alias_summary,
    compute_raw_cell_address_return_summaries, RawCellAddressReturnSummary,
};
use super::model::{
    EffectOp, OwnerState, OwnerStateEntry, Place, PlaceProjection, ResourceBlock, ResourceExprKind,
    ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix};
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
    let raw_alias_summaries = compute_raw_cell_address_return_summaries(module, types);
    let summaries = compute_owner_return_summaries(module, types, &raw_alias_summaries);

    for function in &module.functions {
        let mut engine = ResourceOwnerCheckEngine {
            function: function.name.as_str(),
            types,
            raw_alias_summaries: &raw_alias_summaries,
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
    pub(super) raw_alias_summaries: &'a [RawCellAddressReturnSummary],
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

    pub(super) fn initializer_is_non_owning_raw_alias_view(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        allow_projected_alias_view: bool,
        source: &Place,
        target: &Place,
    ) -> bool {
        if self.types.resolve_id(source.ty) != self.types.resolve_id(target.ty)
            || owners.has_transferable_owner(source)
            || owner_state_under_is_not_non_owning(source, owners)
        {
            return false;
        }
        raw_views.contains_under(source)
            || (allow_projected_alias_view
                && initializer_has_projected_raw_alias_view(raw_aliases, source))
    }

    pub(super) fn copy_non_owning_owner_markers(
        &self,
        owners: &mut OwnerTable,
        source: &Place,
        target: &Place,
    ) {
        if owners.state(source) == Some(OwnerState::NoFreeObligation) {
            owners.set_state(target, OwnerState::NoFreeObligation);
        }
        for entry in owners.descendant_entries(source) {
            if entry.state != OwnerState::NoFreeObligation {
                continue;
            }
            if let Some(target_place) = replace_place_prefix(&entry.place, source, target) {
                owners.set_state(&target_place, OwnerState::NoFreeObligation);
            }
        }
    }

    fn reject_reserved_call_arguments(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        variant_owner_effects: &PendingVariantOwnerEffects,
        args: &[Place],
        span: Span,
    ) -> bool {
        let mut rejected = false;
        for arg in args {
            rejected |= variant_owner_effects.reject_reserved_source_use(
                self,
                owners,
                raw_aliases,
                arg,
                ResourceOwnerOperation::CallArgument,
                span,
            );
        }
        rejected
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
                    } else if self.initializer_is_non_owning_raw_alias_view(
                        owners,
                        raw_aliases,
                        raw_views,
                        true,
                        initializer,
                        place,
                    ) {
                        self.copy_non_owning_owner_markers(owners, initializer, place);
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
                let source_is_reserved = variant_owner_effects.reject_reserved_source_use(
                    self,
                    owners,
                    raw_aliases,
                    source,
                    ResourceOwnerOperation::Read,
                    *span,
                );
                if !source_is_reserved {
                    if self.initializer_is_non_owning_raw_alias_view(
                        owners,
                        raw_aliases,
                        raw_views,
                        true,
                        source,
                        output,
                    ) {
                        self.copy_non_owning_owner_markers(owners, source, output);
                        raw_aliases.copy_alias_or_seed(source, output);
                        storage_origins.copy_origin(source, output);
                    } else if !self.types.is_copy(source.ty)
                        && owners.has_tracked_state_under(source)
                    {
                        self.transfer_owner(
                            owners,
                            raw_aliases,
                            storage_origins,
                            source,
                            output,
                            ResourceOwnerOperation::Read,
                            *span,
                        );
                    } else {
                        raw_aliases.copy_alias_or_seed(source, output);
                        storage_origins.copy_origin(source, output);
                    }
                }
                function_aliases.copy_alias(source, output);
                raw_views.copy(source, output);
                pending_reallocs.copy_result(source, output);
                variant_owner_effects.copy_result(source, output);
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                self.report_overwritten_owners(owners, storage_origins, target, value, *span);
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
                } else if self.initializer_is_non_owning_raw_alias_view(
                    owners,
                    raw_aliases,
                    raw_views,
                    true,
                    value,
                    target,
                ) {
                    self.copy_non_owning_owner_markers(owners, value, target);
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
                    self.transfer_owner(
                        owners,
                        raw_aliases,
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
                operation,
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
                    variant_owner_effects,
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
                let raw_alias_summary_applied = !direct_raw_memory_effect(effect)
                    && apply_direct_call_raw_alias_summary(
                        raw_aliases,
                        output,
                        target,
                        args,
                        self.raw_alias_summaries,
                        self.types,
                    );
                if !direct_raw_memory_effect(effect) || checked_mem_ptr_wrapper {
                    if !raw_alias_summary_applied {
                        raw_aliases.clear(output);
                    }
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
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                span,
                ..
            } => {
                if !apply_indirect_call_raw_alias_summary(
                    raw_aliases,
                    function_aliases,
                    output,
                    callee,
                    args,
                    self.raw_alias_summaries,
                    self.types,
                ) {
                    raw_aliases.clear(output);
                }
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
                raw_views.clear(output);
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
            }
            ResourceOp::RawAddressAlias { source, target, .. } => {
                raw_aliases.copy_alias_or_seed(source, target);
                storage_origins.copy_origin(source, target);
                raw_views.copy(source, target);
                pending_reallocs.copy_result(source, target);
                variant_owner_effects.copy_result(source, target);
            }
            ResourceOp::RawAddressView { source, target, .. } => {
                raw_aliases.copy_alias_or_seed(source, target);
                storage_origins.copy_origin(source, target);
                raw_views.mark(target);
                pending_reallocs.clear_result(target);
                variant_owner_effects.clear_result(target);
            }
            ResourceOp::Borrow { source, output, .. } => {
                let deref_output = output
                    .clone()
                    .with_projection(PlaceProjection::Deref, source.ty);
                raw_aliases.copy_alias_or_seed(source, &deref_output);
                storage_origins.copy_origin(source, &deref_output);
                raw_views.copy(source, &deref_output);
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
            }
            ResourceOp::Expr { output, kind, .. } => {
                self.check_expr(raw_aliases, *kind, output);
            }
            ResourceOp::Drop { .. } | ResourceOp::CallEffect { .. } => {}
        }
    }

    fn check_expr(
        &mut self,
        raw_aliases: &mut RawCellAddressAliases,
        kind: ResourceExprKind,
        output: &Place,
    ) {
        match kind {
            ResourceExprKind::LiteralI32(value) => raw_aliases.set_i32_value(output, value),
            ResourceExprKind::LocalRead
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Branch
            | ResourceExprKind::Match
            | ResourceExprKind::Construct => {}
            ResourceExprKind::Literal
            | ResourceExprKind::FunctionValue
            | ResourceExprKind::Loop
            | ResourceExprKind::Block
            | ResourceExprKind::Let
            | ResourceExprKind::Set
            | ResourceExprKind::Borrow
            | ResourceExprKind::Deref
            | ResourceExprKind::Drop => {
                if matches!(kind, ResourceExprKind::Borrow) {
                    raw_aliases.clear_exact(output);
                } else if !(matches!(kind, ResourceExprKind::Deref)
                    && type_preserves_raw_address_alias(self.types, output.ty))
                {
                    raw_aliases.clear(output);
                }
            }
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

fn call_uses_checked_mem_ptr_wrapper(types: &TypeCtx, args: &[Place]) -> bool {
    args.first()
        .map(|arg| is_mem_ptr_type(types, arg.ty))
        .unwrap_or(false)
}

fn is_mem_ptr_type(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { name, .. } => name == "MemPtr",
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(types.get_ref(base), TypeKind::Struct { name, .. } if name == "MemPtr")
        }
        _ => false,
    }
}

fn type_preserves_raw_address_alias(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { name, .. } => name == "MemPtr" || name == "RegionToken",
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(
                types.get_ref(base),
                TypeKind::Struct { name, .. } if name == "MemPtr" || name == "RegionToken"
            )
        }
        _ => false,
    }
}

fn owner_state_under_is_not_non_owning(source: &Place, owners: &OwnerTable) -> bool {
    owners
        .state(source)
        .is_some_and(|state| state != OwnerState::NoFreeObligation)
        || owners
            .descendant_entries(source)
            .iter()
            .any(|entry| entry.state != OwnerState::NoFreeObligation)
}

fn initializer_has_projected_raw_alias_view(
    raw_aliases: &RawCellAddressAliases,
    source: &Place,
) -> bool {
    raw_aliases
        .aliases_for(source)
        .iter()
        .any(|alias| alias != source && !alias.projections.is_empty())
        || raw_aliases.tracked_places().iter().any(|place| {
            place_suffix_after_prefix(place, source).is_some()
                && raw_aliases
                    .aliases_for(place)
                    .iter()
                    .any(|alias| alias != place && !alias.projections.is_empty())
        })
}
