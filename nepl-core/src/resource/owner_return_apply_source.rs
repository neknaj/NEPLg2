use alloc::vec::Vec;

use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_projection_source::owner_projection_source_returned_by_variant;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_return_apply_place::owner_projection_source_place;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerProjectionSource, OwnerReturnSummary};

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn consume_owner_summary_parameters(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        args: &[Place],
        type_args: &[crate::types::TypeId],
        summary: &OwnerReturnSummary,
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        span: Span,
    ) {
        for arg in summary
            .consumed_parameter_indices
            .iter()
            .filter_map(|index| args.get(*index).map(|arg| (*index, arg)))
        {
            let source = OwnerProjectionSource {
                parameter_index: arg.0,
                suffix: Vec::new(),
                ty: arg.1.ty,
            };
            if owner_projection_source_returned_by_variant(summary, &source) {
                continue;
            }
            self.consume_summary_argument_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg.1,
                &source,
                &summary.consumed_extent_requirements,
                args,
                type_args,
                summary,
                variant_owner_effects,
                span,
            );
        }
        for source in &summary.consumed_parameter_sources {
            if owner_projection_source_returned_by_variant(summary, source) {
                continue;
            }
            let Some(source_place) = owner_projection_source_place(args, source) else {
                continue;
            };
            self.consume_summary_argument_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source_place,
                source,
                &summary.consumed_extent_requirements,
                args,
                type_args,
                summary,
                variant_owner_effects,
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

    pub(super) fn try_copy_parameter_view_return(
        &self,
        owners: &OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        source: &Place,
        output: &Place,
    ) -> bool {
        if !self.place_is_copy_owner_view(owners, raw_aliases, source) {
            return false;
        }
        raw_aliases.copy_alias_if_tracked(source, output);
        raw_views.copy(source, output);
        storage_origins.copy_origin(source, output);
        true
    }
}
