use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::place_utils::place_with_suffix;
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerProjectionSource, OwnerReturnSummary};

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn consume_owner_summary_parameters(
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

    pub(super) fn has_returnable_parameter_owner(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        place: &Place,
    ) -> bool {
        !self.place_is_non_owning_raw_address_view(owners, raw_aliases, raw_views, place)
            && self.has_transferable_owner(owners, raw_aliases, place)
    }

    pub(super) fn try_copy_non_owning_parameter_return(
        &self,
        owners: &OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        source: &Place,
        output: &Place,
    ) -> bool {
        if !self.place_is_non_owning_raw_address_view(owners, raw_aliases, raw_views, source) {
            return false;
        }
        raw_aliases.copy_alias_if_tracked(source, output);
        storage_origins.copy_origin(source, output);
        if raw_views.contains_non_owning(source) {
            raw_views.copy_non_owning(source, output);
        } else {
            raw_views.mark_non_owning(output);
        }
        true
    }
}

pub(super) fn owner_projection_source_place(
    args: &[Place],
    source: &OwnerProjectionSource,
) -> Option<Place> {
    let arg = args.get(source.parameter_index)?;
    Some(place_with_suffix(arg, &source.suffix, source.ty))
}
