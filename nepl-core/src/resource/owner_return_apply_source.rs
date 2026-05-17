use alloc::vec::Vec;

use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use crate::types::TypeId;

use super::model::{Place, PlaceProjection};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::instantiate_owner_extent_summary;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_summary_leaf::owner_leaf_places;
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::place_with_suffix;
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerConsumedExtentRequirement, OwnerProjectionSource, OwnerReturnSummary};

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn consume_owner_summary_parameters(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        args: &[Place],
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
            self.consume_summary_argument_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg.1,
                &source,
                &summary.consumed_extent_requirements,
                args,
                variant_owner_effects,
                span,
            );
        }
        for source in &summary.consumed_parameter_sources {
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

    fn consume_summary_argument_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        arg: &Place,
        source: &OwnerProjectionSource,
        requirements: &[OwnerConsumedExtentRequirement],
        args: &[Place],
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        span: Span,
    ) {
        variant_owner_effects.materialize_return_owner_for_target(
            self,
            owners,
            raw_aliases,
            raw_views,
            storage_origins,
            arg,
            span,
        );
        if self.place_is_copy_owner_view(owners, raw_aliases, arg) {
            return;
        }
        let requirement = requirements
            .iter()
            .find(|requirement| &requirement.owner == source);
        if let Some(requirement) = requirement {
            let extent = instantiate_owner_extent_summary(args, &requirement.extent);
            self.consume_call_argument_owner_with_extent(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg,
                &extent,
                requirement.operation,
                span,
            );
        } else {
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

    pub(super) fn place_is_copy_owner_view(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> bool {
        self.types.is_copy(place.ty)
            && !self.has_transferable_owner(owners, raw_aliases, place)
            && owner_leaf_places(self.types, place)
                .iter()
                .any(|leaf| leaf.place == *place)
    }
}

pub(super) fn owner_projection_source_place(
    args: &[Place],
    source: &OwnerProjectionSource,
) -> Option<Place> {
    let arg = args.get(source.parameter_index)?;
    Some(owner_projection_source_place_for_arg(arg, source))
}

pub(super) fn owner_projection_source_place_for_arg(
    arg: &Place,
    source: &OwnerProjectionSource,
) -> Place {
    summary_projection_place(arg, &source.suffix, source.ty)
}

pub(super) fn summary_projection_place(
    base: &Place,
    suffix: &[PlaceProjection],
    ty: TypeId,
) -> Place {
    let ty = if suffix.is_empty() { base.ty } else { ty };
    place_with_suffix(base, suffix, ty)
}
