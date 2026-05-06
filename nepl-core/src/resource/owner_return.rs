use crate::span::Span;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceCallTarget};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_address::{raw_address_return_ownership, RawAddressReturnOwnership};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::report::ResourceOwnerOperation;
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
        let ResourceCallTarget::User { name, .. } = target else {
            return;
        };
        match raw_address_return_ownership(name) {
            Some(RawAddressReturnOwnership::NonOwningAddressView) => return,
            None => {}
        }
        let Some(summary) = self
            .summaries
            .iter()
            .find(|summary| summary.function == name.as_str())
        else {
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
                output,
                args,
                summary,
                span,
            );
        }
        variant_owner_effects.record_call(raw_aliases, output, args, summary);
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
            if let Some(summary) = self
                .summaries
                .iter()
                .find(|summary| summary.function == function.as_str())
            {
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
                    output,
                    args,
                    summary,
                    span,
                );
                variant_owner_effects.record_call(raw_aliases, output, args, summary);
                if self.has_transferable_owner(owners, raw_aliases, output) {
                    return;
                }
            }
        }
    }

    fn apply_unknown_indirect_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        let mut returned_index = None;
        for (index, arg) in args
            .iter()
            .enumerate()
            .filter(|(_, arg)| arg.ty == output.ty)
        {
            if !self.place_is_non_owning_raw_address_view(owners, raw_aliases, raw_views, arg)
                && self.has_transferable_owner(owners, raw_aliases, arg)
            {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    arg,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                returned_index = Some(index);
                break;
            }
        }
        for (index, arg) in args.iter().enumerate() {
            if returned_index == Some(index) {
                continue;
            }
            self.consume_call_argument_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg,
                span,
            );
        }
    }
}
