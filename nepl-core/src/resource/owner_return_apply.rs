use alloc::vec::Vec;

use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::instantiate_owner_extent_summary;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_return_apply_source::owner_projection_source_place;
use super::owner_state::OwnerTable;
use super::owner_summary_record::parameter_return_extent_for_source;
use super::place_utils::place_with_suffix;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerConsumedExtentRequirement, OwnerNonOwningRawViewKind, OwnerProjectionReturnSummary,
    OwnerProjectionSource, OwnerReturnSummary,
};

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn apply_owner_return_summary(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        args: &[Place],
        summary: &OwnerReturnSummary,
        span: Span,
    ) {
        let mut transferred = false;
        for (parameter_index, arg) in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index).map(|arg| (*index, arg)))
        {
            let source = OwnerProjectionSource {
                parameter_index,
                suffix: Vec::new(),
                ty: arg.ty,
            };
            if self.try_copy_non_owning_parameter_return(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg,
                output,
            ) {
                transferred = true;
                break;
            }
            if self.has_returnable_parameter_owner(owners, raw_aliases, raw_views, arg) {
                if !self.summary_return_extent_requirement_holds(
                    owners,
                    raw_aliases,
                    arg,
                    args,
                    &source,
                    &summary.consumed_extent_requirements,
                    span,
                ) {
                    continue;
                }
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
                apply_returned_owner_extent(
                    owners,
                    args,
                    output,
                    &source,
                    &summary.parameter_return_extents,
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
            if self.try_copy_non_owning_parameter_return(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source_place,
                output,
            ) {
                transferred = true;
                break;
            }
            if self.has_returnable_parameter_owner(owners, raw_aliases, raw_views, &source_place) {
                if !self.summary_return_extent_requirement_holds(
                    owners,
                    raw_aliases,
                    &source_place,
                    args,
                    source,
                    &[],
                    span,
                ) {
                    continue;
                }
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
                apply_returned_owner_extent(
                    owners,
                    args,
                    output,
                    source,
                    &summary.parameter_return_extents,
                );
                transferred = true;
            }
        }
        if summary.returns_maybe_owner && !transferred {
            owners.set_state(output, OwnerState::MaybeFreed { storage: None });
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        } else if summary.returns_fresh_owner && !transferred {
            owners.allocate_with_extent(
                output,
                instantiate_owner_extent_summary(args, &summary.returns_fresh_owner_extent),
            );
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        }
        for marker in &summary.non_owning_raw_view_returns {
            let marker_place = place_with_suffix(output, &marker.suffix, marker.ty);
            match marker.kind {
                OwnerNonOwningRawViewKind::AliasView => raw_views.mark_non_owning(&marker_place),
                OwnerNonOwningRawViewKind::ProjectionView => {
                    raw_views.mark_non_owning_projection(&marker_place)
                }
            }
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
        for marker in &summary.storage_origin_markers {
            let marker_place = place_with_suffix(output, &marker.suffix, marker.ty);
            storage_origins.mark_origin(&marker_place, marker.origin);
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
        raw_views: &mut RawAddressViewTable,
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
        for (parameter_index, arg) in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index).map(|arg| (*index, arg)))
        {
            let source = OwnerProjectionSource {
                parameter_index,
                suffix: Vec::new(),
                ty: arg.ty,
            };
            if self.try_copy_non_owning_parameter_return(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                arg,
                output,
            ) {
                transferred = true;
                break;
            }
            if self.has_returnable_parameter_owner(owners, raw_aliases, raw_views, arg) {
                if !self.summary_return_extent_requirement_holds(
                    owners,
                    raw_aliases,
                    arg,
                    args,
                    &source,
                    &[],
                    span,
                ) {
                    continue;
                }
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
                apply_returned_owner_extent(
                    owners,
                    args,
                    output,
                    &source,
                    &summary.parameter_return_extents,
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
            if self.try_copy_non_owning_parameter_return(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source_place,
                output,
            ) {
                transferred = true;
                break;
            }
            if self.has_returnable_parameter_owner(owners, raw_aliases, raw_views, &source_place) {
                if !self.summary_return_extent_requirement_holds(
                    owners,
                    raw_aliases,
                    &source_place,
                    args,
                    source,
                    &[],
                    span,
                ) {
                    continue;
                }
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
                apply_returned_owner_extent(
                    owners,
                    args,
                    output,
                    source,
                    &summary.parameter_return_extents,
                );
                transferred = true;
            }
        }
        if summary.returns_maybe_owner && !transferred {
            owners.set_state(output, OwnerState::MaybeFreed { storage: None });
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        } else if summary.returns_fresh_owner && !transferred {
            owners.allocate_with_extent(
                output,
                instantiate_owner_extent_summary(args, &summary.returns_fresh_owner_extent),
            );
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        }
    }

    fn summary_return_extent_requirement_holds(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
        args: &[Place],
        source: &OwnerProjectionSource,
        requirements: &[OwnerConsumedExtentRequirement],
        span: Span,
    ) -> bool {
        let Some(requirement) = requirements
            .iter()
            .find(|requirement| &requirement.owner == source)
        else {
            return true;
        };
        let extent = instantiate_owner_extent_summary(args, &requirement.extent);
        if self.ensure_owner_extent_matches_summary(
            owners,
            raw_aliases,
            place,
            &extent,
            requirement.operation,
            span,
        ) {
            true
        } else {
            self.push_unavailable(
                requirement.operation,
                place,
                owners.state(place).unwrap_or(OwnerState::NoFreeObligation),
                span,
            );
            false
        }
    }
}

fn apply_returned_owner_extent(
    owners: &mut OwnerTable,
    args: &[Place],
    output: &Place,
    source: &OwnerProjectionSource,
    extents: &[super::summary::OwnerParameterReturnExtent],
) {
    let Some(extent) = parameter_return_extent_for_source(extents, source) else {
        return;
    };
    let extent = instantiate_owner_extent_summary(args, extent);
    if !matches!(extent, super::model::OwnerStorageExtent::Unknown) {
        owners.set_live_extent(output, extent);
    }
}
