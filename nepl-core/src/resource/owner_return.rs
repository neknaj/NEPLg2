use crate::span::Span;
use alloc::vec::Vec;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, ResourceCallTarget};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_address::{raw_address_return_ownership, RawAddressReturnOwnership};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::{place_with_suffix, places_overlap};
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerProjectionReturnSummary, OwnerProjectionSource, OwnerReturnSummary};

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
        if apply_unconditional_summary {
            variant_owner_effects.materialize_call_argument_variant_returns(
                self,
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                args,
                summary,
                span,
            );
            variant_owner_effects.record_call(
                owners,
                raw_aliases,
                raw_views,
                output,
                args,
                summary,
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
        } else {
            variant_owner_effects.record_call(
                owners,
                raw_aliases,
                raw_views,
                output,
                args,
                summary,
            );
        }
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
                variant_owner_effects.materialize_call_argument_variant_returns(
                    self,
                    owners,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    args,
                    summary,
                    span,
                );
                variant_owner_effects.record_call(
                    owners,
                    raw_aliases,
                    raw_views,
                    output,
                    args,
                    summary,
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
            if self.has_returnable_parameter_owner(owners, raw_aliases, raw_views, arg) {
                self.transfer_owner(
                    owners,
                    raw_aliases,
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
            self.consume_call_argument_owner(owners, raw_aliases, storage_origins, arg, span);
        }
    }

    fn apply_owner_return_summary(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        args: &[Place],
        summary: &OwnerReturnSummary,
        span: Span,
    ) {
        let mut transferred = false;
        for arg in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
        {
            if self.has_returnable_parameter_owner(owners, raw_aliases, raw_views, arg) {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    arg,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                transferred = true;
                break;
            }
        }
        for source in &summary.parameter_sources {
            if transferred {
                break;
            }
            let Some(source_place) = owner_projection_source_place(args, source) else {
                continue;
            };
            if self.has_returnable_parameter_owner(owners, raw_aliases, raw_views, &source_place) {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    &source_place,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                transferred = true;
            }
        }
        if summary.returns_maybe_owner && !transferred {
            owners.set_state(output, OwnerState::MaybeFreed { storage: None });
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        } else if summary.returns_fresh_owner && !transferred {
            owners.allocate(output);
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        }
        for projection in &summary.projection_returns {
            let output_projection = place_with_suffix(output, &projection.suffix, projection.ty);
            self.apply_owner_projection_return_summary(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &output_projection,
                args,
                projection,
                span,
            );
        }
        for marker in &summary.projection_markers {
            let marker_place = place_with_suffix(output, &marker.suffix, marker.ty);
            if owners.state(&marker_place).is_none() {
                owners.set_state(&marker_place, OwnerState::NoFreeObligation);
            }
        }
        let protected_return_places = projection_return_places(output, summary);
        self.consume_owner_summary_parameters(
            owners,
            raw_aliases,
            storage_origins,
            args,
            summary,
            &protected_return_places,
            span,
        );
    }

    fn apply_owner_projection_return_summary(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        args: &[Place],
        summary: &OwnerProjectionReturnSummary,
        span: Span,
    ) {
        let mut transferred = false;
        if owners.state(output).is_none() {
            owners.set_state(output, OwnerState::NoFreeObligation);
        }
        if owners.has_transferable_owner(output) {
            return;
        }
        for arg in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
        {
            if self.has_returnable_parameter_owner(owners, raw_aliases, raw_views, arg) {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    arg,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                transferred = true;
                break;
            }
        }
        for source in &summary.parameter_sources {
            if transferred {
                break;
            }
            let Some(source_place) = owner_projection_source_place(args, source) else {
                continue;
            };
            if self.has_returnable_parameter_owner(owners, raw_aliases, raw_views, &source_place) {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    &source_place,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                transferred = true;
            }
        }
        if summary.returns_maybe_owner && !transferred {
            owners.set_state(output, OwnerState::MaybeFreed { storage: None });
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        } else if summary.returns_fresh_owner && !transferred {
            owners.allocate(output);
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        }
    }

    fn consume_owner_summary_parameters(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        args: &[Place],
        summary: &OwnerReturnSummary,
        protected_return_places: &[Place],
        span: Span,
    ) {
        for arg in summary
            .consumed_parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
        {
            self.consume_call_argument_owner_protecting_returns(
                owners,
                raw_aliases,
                storage_origins,
                arg,
                protected_return_places,
                span,
            );
        }
        for source in &summary.consumed_parameter_sources {
            let Some(source_place) = owner_projection_source_place(args, source) else {
                continue;
            };
            self.consume_call_argument_owner_protecting_returns(
                owners,
                raw_aliases,
                storage_origins,
                &source_place,
                protected_return_places,
                span,
            );
        }
    }

    fn consume_call_argument_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        arg: &Place,
        span: Span,
    ) {
        if self.has_transferable_owner(owners, raw_aliases, arg) {
            self.move_owner_out(
                owners,
                raw_aliases,
                storage_origins,
                arg,
                ResourceOwnerOperation::CallArgument,
                span,
            );
        }
    }

    fn consume_call_argument_owner_protecting_returns(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        arg: &Place,
        protected_return_places: &[Place],
        span: Span,
    ) {
        let resolved = resolve_owner_alias_place(owners, raw_aliases, arg);
        if place_or_alias_overlaps(raw_aliases, arg, protected_return_places)
            || place_or_alias_overlaps(raw_aliases, &resolved, protected_return_places)
        {
            self.move_owner_out_exact_protecting_aliases(
                owners,
                raw_aliases,
                storage_origins,
                arg,
                ResourceOwnerOperation::CallArgument,
                span,
                protected_return_places,
            );
            return;
        }
        self.consume_call_argument_owner(owners, raw_aliases, storage_origins, arg, span);
    }

    fn has_transferable_owner(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> bool {
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        owners.has_transferable_owner(&resolved_place)
    }

    fn has_returnable_parameter_owner(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        place: &Place,
    ) -> bool {
        !self.place_is_non_owning_raw_address_view(owners, raw_views, place)
            && self.has_transferable_owner(owners, raw_aliases, place)
    }

    fn place_is_non_owning_raw_address_view(
        &self,
        owners: &OwnerTable,
        raw_views: &RawAddressViewTable,
        place: &Place,
    ) -> bool {
        self.types.resolve_id(place.ty) == self.types.i32()
            && raw_views.contains(place)
            && !owners.has_transferable_owner(place)
            && !owners.has_tracked_state_under(place)
    }
}

fn owner_projection_source_place(args: &[Place], source: &OwnerProjectionSource) -> Option<Place> {
    let arg = args.get(source.parameter_index)?;
    Some(place_with_suffix(arg, &source.suffix, source.ty))
}

fn projection_return_places(output: &Place, summary: &OwnerReturnSummary) -> Vec<Place> {
    let mut places = Vec::new();
    if summary.returns_fresh_owner
        || summary.returns_maybe_owner
        || !summary.parameter_indices.is_empty()
        || !summary.parameter_sources.is_empty()
    {
        push_unique_place(&mut places, output);
    }
    for projection in &summary.projection_returns {
        push_unique_place(
            &mut places,
            &place_with_suffix(output, &projection.suffix, projection.ty),
        );
    }
    for variant_projection in &summary.variant_projection_returns {
        push_unique_place(
            &mut places,
            &place_with_suffix(output, &variant_projection.suffix, variant_projection.ty),
        );
    }
    places
}

fn place_or_alias_overlaps(
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
    protected_places: &[Place],
) -> bool {
    raw_aliases.aliases_for(place).iter().any(|alias| {
        protected_places
            .iter()
            .any(|protected| places_overlap(alias, protected))
    })
}

fn push_unique_place(places: &mut Vec<Place>, place: &Place) {
    if !places.iter().any(|existing| existing == place) {
        places.push(place.clone());
    }
}
