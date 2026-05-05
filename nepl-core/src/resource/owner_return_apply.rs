use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::place_utils::place_with_suffix;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerProjectionReturnSummary, OwnerProjectionSource, OwnerReturnSummary};

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn apply_owner_return_summary(
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
                    raw_views,
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
                    raw_views,
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
        self.consume_owner_summary_parameters(
            owners,
            raw_aliases,
            raw_views,
            storage_origins,
            args,
            summary,
            span,
        );
    }

    pub(super) fn consume_call_argument_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        arg: &Place,
        span: Span,
    ) {
        if !self.place_is_non_owning_raw_address_view(owners, raw_aliases, raw_views, arg)
            && self.has_transferable_owner(owners, raw_aliases, arg)
        {
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

    pub(super) fn has_transferable_owner(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> bool {
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        owners.has_transferable_owner(&resolved_place)
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
        if self.has_transferable_owner(owners, raw_aliases, output) {
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
                    raw_views,
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
                    raw_views,
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
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        args: &[Place],
        summary: &OwnerReturnSummary,
        span: Span,
    ) {
        for arg in summary
            .consumed_parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
        {
            self.consume_call_argument_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg,
                span,
            );
        }
        for source in &summary.consumed_parameter_sources {
            let Some(source_place) = owner_projection_source_place(args, source) else {
                continue;
            };
            self.consume_call_argument_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source_place,
                span,
            );
        }
    }

    fn has_returnable_parameter_owner(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        place: &Place,
    ) -> bool {
        !self.place_is_non_owning_raw_address_view(owners, raw_aliases, raw_views, place)
            && self.has_transferable_owner(owners, raw_aliases, place)
    }
}

fn owner_projection_source_place(args: &[Place], source: &OwnerProjectionSource) -> Option<Place> {
    let arg = args.get(source.parameter_index)?;
    Some(place_with_suffix(arg, &source.suffix, source.ty))
}
