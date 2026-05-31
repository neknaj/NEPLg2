use crate::span::Span;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceCallTarget};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_address::{raw_address_return_ownership, RawAddressReturnOwnership};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn apply_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
        apply_unconditional_summary: bool,
        span: Span,
    ) {
        let ResourceCallTarget::User { name, type_args } = target else {
            return;
        };
        match raw_address_return_ownership(name) {
            Some(RawAddressReturnOwnership::NonOwningAddressView) => return,
            None => {}
        }
        let Some(summary) = self.summaries.get(name) else {
            return;
        };
        variant_owner_effects.apply_resolved_parameter_variants(
            self,
            owners,
            raw_aliases,
            raw_views,
            storage_origins,
            args,
            &summary.resolved_parameter_variants,
            span,
        );
        if apply_unconditional_summary {
            self.apply_owner_return_summary(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                variant_owner_effects,
                output,
                args,
                type_args,
                summary,
                span,
            );
        } else if !self.apply_owner_memory_span_requirements(
            owners,
            raw_aliases,
            raw_views,
            args,
            &summary.memory_span_requirements,
            span,
        ) {
            return;
        }
        variant_owner_effects.record_call(
            self.types,
            raw_aliases,
            output,
            args,
            type_args,
            summary,
        );
    }

    pub(super) fn apply_indirect_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        output: &Place,
        callee: &Place,
        args: &[Place],
        span: Span,
    ) {
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            self.apply_unknown_indirect_call_return_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                output,
                args,
                span,
            );
            variant_owner_effects.clear_result(output);
            return;
        }
        for function in functions {
            if let Some(summary) = self.summaries.get(function.symbol()) {
                variant_owner_effects.apply_resolved_parameter_variants(
                    self,
                    owners,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    args,
                    &summary.resolved_parameter_variants,
                    span,
                );
                self.apply_owner_return_summary(
                    owners,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    variant_owner_effects,
                    output,
                    args,
                    &[],
                    summary,
                    span,
                );
                variant_owner_effects.record_call(
                    self.types,
                    raw_aliases,
                    output,
                    args,
                    &[],
                    summary,
                );
                if self.has_transferable_owner(owners, raw_aliases, output) {
                    return;
                }
            }
        }
    }
}
